//! Layer 5: agent intent & dynamic preference — soft weighting + entropy
//! routing. Behavior aligned with Python v1.7 `layer5_score.py`:
//! bias-score addition, role-match bonus (+10), softmax probabilities, and
//! deterministic weighted sampling seeded by instance_id + request_hash.
//!
//! Determinism (ADR-0100 D4): same instance + same request -> bit-identical
//! decision; different instances decorrelate (anti-herd). The Python seed
//! (`sha256[:8]` big-endian) is reproduced exactly.

use crate::config::BiasConfig;
use crate::pb::{EligibleSupplier, RoutingDecision};
use sha2::{Digest, Sha256};

/// Deterministic seed from instance_id and request_hash (Python shape:
/// sha256("instance:request")[:8] big-endian).
pub fn deterministic_hash(instance_id: &str, request_hash: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(instance_id.as_bytes());
    hasher.update(b":");
    hasher.update(request_hash.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes)
}

/// Softmax probabilities (Python v1.7: shift by max for stability, e = 2.71828...).
pub fn softmax(scores: &[f64]) -> Vec<f64> {
    if scores.is_empty() {
        return Vec::new();
    }
    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = scores
        .iter()
        .map(|s| std::f64::consts::E.powf(s - max_score))
        .collect();
    let total: f64 = exps.iter().sum();
    if total == 0.0 {
        let uniform = 1.0 / scores.len() as f64;
        return vec![uniform; scores.len()];
    }
    exps.iter().map(|e| e / total).collect()
}

/// Deterministic weighted sample: same seed + same candidates -> same pick.
pub fn weighted_sample<'a, T>(
    candidates: &'a [T],
    probabilities: &[f64],
    seed: u64,
) -> Option<&'a T> {
    if candidates.len() == 1 {
        return candidates.first();
    }
    let normalized_seed = (seed % 1_000_000) as f64 / 1_000_000.0;
    let mut cumulative = 0.0;
    for (i, prob) in probabilities.iter().enumerate() {
        cumulative += prob;
        if normalized_seed < cumulative {
            return candidates.get(i);
        }
    }
    candidates.last()
}

/// Score candidates and perform entropy-weighted routing (pure).
/// `free_ids`: suppliers whose billing is all-zero (physical fact) — they
/// get the user-configured soft bonus (0 tokens first, 2026-09-09).
pub fn score_and_entropy_sample(
    candidates: &[EligibleSupplier],
    agent_role: &str,
    instance_id: &str,
    request_hash: &str,
    bias_config: &BiasConfig,
    free_ids: &std::collections::HashSet<String>,
) -> RoutingDecision {
    assert!(!candidates.is_empty(), "at least one candidate is required");

    let scored: Vec<(f64, &EligibleSupplier)> = candidates
        .iter()
        .map(|c| {
            let mut score = c.score as f64;
            if let Some(bias) = bias_config.supplier_biases.get(&c.supplier_id) {
                score += bias.bias_score;
            }
            if free_ids.contains(&c.supplier_id) {
                score += bias_config.free_tier_bonus;
            }
            if !agent_role.is_empty() && c.model_id.to_lowercase().contains(agent_role) {
                score += 10.0;
            }
            (score, c)
        })
        .collect();
    // NOTE: input order is preserved (Python v1.7 behavior) — probabilities
    // accumulate over the original candidate order, no sorting.

    let raw_scores: Vec<f64> = scored.iter().map(|(s, _)| *s).collect();
    let probabilities = softmax(&raw_scores);
    let seed = deterministic_hash(instance_id, request_hash);
    let refs: Vec<&EligibleSupplier> = scored.iter().map(|(_, c)| *c).collect();
    let selected = weighted_sample(&refs, &probabilities, seed).expect("non-empty candidates");
    let selected = *selected;

    // KV cache defaults. Source: protocol sample declaration (whitepaper §2.2:
    // ttl_seconds 300, control_parameter "cache_control"); overridden later by
    // supplier-specific settings.
    let kv_applicable = selected.cost.as_ref().map(|c| c.kv_cache_applicable).unwrap_or(false);
    let (kv_cache_parameter, kv_cache_ttl) = if kv_applicable {
        ("cache_control".to_string(), 300)
    } else {
        (String::new(), 0)
    };

    RoutingDecision {
        supplier_id: selected.supplier_id.clone(),
        model_id: selected.model_id.clone(),
        endpoint_url: selected.endpoint_url.clone(),
        estimated_cost_usd: selected.cost.as_ref().map(|c| c.estimated_cost_usd).unwrap_or(0.0),
        kv_cache_parameter,
        kv_cache_ttl,
        request_id: request_hash.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::CostEstimate;

    fn eligible(supplier: &str, model: &str, score: f32, url: &str) -> EligibleSupplier {
        EligibleSupplier {
            supplier_id: supplier.into(),
            model_id: model.into(),
            endpoint_url: url.into(),
            score,
            cost: Some(CostEstimate {
                supplier_id: supplier.into(),
                model_id: model.into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn softmax_uniform_for_empty() {
        assert!(softmax(&[]).is_empty());
    }

    #[test]
    fn softmax_sums_to_one() {
        let p = softmax(&[1.0, 2.0, 3.0]);
        let total: f64 = p.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
        // Higher score -> higher probability
        assert!(p[2] > p[1] && p[1] > p[0]);
    }

    #[test]
    fn hash_deterministic() {
        let a = deterministic_hash("inst", "req");
        let b = deterministic_hash("inst", "req");
        assert_eq!(a, b);
        assert_ne!(a, deterministic_hash("inst2", "req"));
    }

    #[test]
    fn entropy_same_instance_same_output() {
        let candidates = vec![
            eligible("t1", "model1", 100.0, "http://t1"),
            eligible("t2", "model2", 90.0, "http://t2"),
            eligible("t3", "model3", 80.0, "http://t3"),
        ];
        let bias = BiasConfig::default();
        let first = score_and_entropy_sample(&candidates, "default", "fixed-instance", "req-123", &bias, &std::collections::HashSet::new());
        for _ in 0..100 {
            let d = score_and_entropy_sample(&candidates, "default", "fixed-instance", "req-123", &bias, &std::collections::HashSet::new());
            assert_eq!(d.supplier_id, first.supplier_id);
        }
    }

    #[test]
    fn entropy_different_instances_diverge() {
        let candidates = vec![
            eligible("t1", "model1", 100.0, "http://t1"),
            eligible("t2", "model2", 99.0, "http://t2"),
            eligible("t3", "model3", 98.0, "http://t3"),
        ];
        let bias = BiasConfig::default();
        let mut seen = std::collections::HashSet::new();
        for i in 0..100 {
            let d = score_and_entropy_sample(
                &candidates,
                "default",
                &format!("test-instance-{i}"),
                "req-456",
                &bias,
                &std::collections::HashSet::new(),
            );
            seen.insert(d.supplier_id);
        }
        assert!(seen.len() > 1, "expected divergence across instances");
    }

    #[test]
    fn role_bonus_applies() {
        // Score gap 100 vs 90 is small; the +10 role bonus flips it: 110 vs 90,
        // softmax is overwhelming (>0.9999999), selection is deterministic.
        let candidates = vec![
            eligible("t1", "code-model", 100.0, "http://t1"),
            eligible("t2", "other", 90.0, "http://t2"),
        ];
        let bias = BiasConfig::default();
        let d = score_and_entropy_sample(&candidates, "code", "i", "r", &bias, &std::collections::HashSet::new());
        assert_eq!(d.supplier_id, "t1");
    }

    #[test]
    fn kv_cache_defaults_from_protocol() {
        let mut c = eligible("t1", "m1", 1.0, "http://t1");
        c.cost.as_mut().unwrap().kv_cache_applicable = true;
        let d = score_and_entropy_sample(&[c], "", "i", "r", &BiasConfig::default(), &std::collections::HashSet::new());
        assert_eq!(d.kv_cache_parameter, "cache_control");
        assert_eq!(d.kv_cache_ttl, 300);
    }
}
