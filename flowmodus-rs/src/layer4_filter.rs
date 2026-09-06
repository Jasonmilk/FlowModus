//! Layer 4: user preferences & hard boundaries — the hard filter.
//!
//! Behavior aligned with Python v1.7 `layer4_filter.py`: budget / verified /
//! deviation-tolerance constraints, per-supplier bias cost caps, then a
//! cascading priority-group gate with degradation/rehabilitation semantics.
//!
//! Determinism note (ADR-0100 D4): Python v1.7 seeded rehabilitation with the
//! builtin `hash()` (stable within a process, RANDOM across processes). rs
//! derives the seed from sha256(instance_id + supplier_id + minute) — same
//! inputs give bit-identical outcomes on every run, everywhere.

use crate::config::{BiasConfig, PriorityGroup};
use crate::layer2_5_deviation::DeviationSnapshot;
use crate::pb::{CostEstimate, UserConstraints};
use sha2::{Digest, Sha256};

/// Deterministic seed: sha256(instance|supplier|minute) -> u64.
fn rehab_seed(instance_id: &str, supplier_id: &str, current_minute: i64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(instance_id.as_bytes());
    hasher.update(b"|");
    hasher.update(supplier_id.as_bytes());
    hasher.update(b"|");
    hasher.update(current_minute.to_string().as_bytes());
    let digest = hasher.finalize();
    u64::from_be_bytes(digest[..8].try_into().expect("8 bytes"))
}

/// Whether to attempt rehabilitation of a degraded supplier (pure).
/// Python semantics: cooldown gate, then modulus-space probability check.
pub fn should_rehabilitate(
    supplier_id: &str,
    instance_id: &str,
    time_since_degraded: i64,
    current_minute: i64,
    rehabilitation_probability: f64,
    rehabilitation_cooldown_seconds: i64,
) -> bool {
    if time_since_degraded < rehabilitation_cooldown_seconds {
        return false;
    }
    let seed = rehab_seed(instance_id, supplier_id, current_minute);
    let modulus = if rehabilitation_probability > 0.0 {
        (1.0 / rehabilitation_probability) as u64
    } else {
        1
    };
    seed.is_multiple_of(modulus)
}

/// Cascading priority-group filter (pure). Groups sorted by priority desc;
/// the first group with surviving candidates wins; else original candidates.
pub fn apply_priority_filter(
    candidates: Vec<CostEstimate>,
    priority_groups: &[PriorityGroup],
    health_states: &std::collections::HashMap<String, String>,
    health_timestamps: &std::collections::HashMap<String, i64>,
    instance_id: &str,
    current_minute: i64,
    bias_config: &BiasConfig,
) -> Vec<CostEstimate> {
    if priority_groups.is_empty() {
        return candidates;
    }
    let mut sorted: Vec<&PriorityGroup> = priority_groups.iter().collect();
    sorted.sort_by_key(|g| std::cmp::Reverse(g.priority));

    for group in sorted {
        let mut group_candidates = Vec::new();
        for candidate in &candidates {
            if !group.models.iter().any(|m| m == &candidate.model_id) {
                continue;
            }
            let state = health_states
                .get(&candidate.supplier_id)
                .map(|s| s.as_str())
                .unwrap_or("HEALTHY");
            if state == "TERMINAL" {
                continue;
            }
            if state == "DEGRADED" {
                let since = (current_minute * 60)
                    - health_timestamps.get(&candidate.supplier_id).copied().unwrap_or(0);
                let since = if since < 0 { 0 } else { since };
                if !should_rehabilitate(
                    &candidate.supplier_id,
                    instance_id,
                    since,
                    current_minute,
                    bias_config.rehabilitation_probability,
                    bias_config.rehabilitation_cooldown_seconds,
                ) {
                    continue;
                }
            }
            group_candidates.push(candidate.clone());
        }
        if !group_candidates.is_empty() {
            return group_candidates;
        }
    }
    candidates
}

/// Apply all hard filters + priority cascade (pure).
#[allow(clippy::too_many_arguments)]
pub fn apply_hard_filters(
    candidates: Vec<CostEstimate>,
    constraints: &UserConstraints,
    deviation_snapshot: &DeviationSnapshot,
    bias_config: &BiasConfig,
    health_states: &std::collections::HashMap<String, String>,
    health_timestamps: &std::collections::HashMap<String, i64>,
    instance_id: &str,
    current_minute: i64,
) -> Vec<CostEstimate> {
    let mut filtered = Vec::new();

    for candidate in candidates {
        if constraints.max_cost_per_request_usd > 0.0
            && candidate.estimated_cost_usd > constraints.max_cost_per_request_usd {
                continue;
            }
        // `require_verified_supplier` is honored upstream (registry snapshot);
        // Python v1.7 also passes it through without extra action here.
        let deviation = deviation_snapshot.get_deviation(&candidate.supplier_id, "billing_accuracy");
        if let Some(d) = deviation {
            if constraints.max_claim_deviation_tolerance > 0.0
                && d.deviation_percent.abs() > constraints.max_claim_deviation_tolerance
            {
                continue;
            }
        }
        if let Some(bias) = bias_config.supplier_biases.get(&candidate.supplier_id) {
            if let Some(cap) = bias.max_cost_per_request {
                if candidate.estimated_cost_usd > cap as f32 {
                    continue;
                }
            }
        }
        filtered.push(candidate);
    }

    apply_priority_filter(
        filtered,
        &bias_config.priority_groups,
        health_states,
        health_timestamps,
        instance_id,
        current_minute,
        bias_config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SupplierBias;
    use crate::pb::ClaimDeviation;

    fn est(supplier: &str, model: &str, cost: f32) -> CostEstimate {
        CostEstimate {
            supplier_id: supplier.into(),
            model_id: model.into(),
            estimated_cost_usd: cost,
            ..Default::default()
        }
    }

    fn constraints(max_cost: f32, tolerance: f32) -> UserConstraints {
        UserConstraints {
            max_cost_per_request_usd: max_cost,
            max_claim_deviation_tolerance: tolerance,
            ..Default::default()
        }
    }

    #[test]
    fn budget_cuts_over_budget() {
        let candidates = vec![
            est("s1", "m1", 0.01),
            est("s2", "m2", 0.02),
            est("s3", "m3", 0.005),
        ];
        let c = constraints(0.015, 10.0);
        let snap = DeviationSnapshot::default();
        let bias = BiasConfig::default();
        let out = apply_hard_filters(
            candidates,
            &c,
            &snap,
            &bias,
            &Default::default(),
            &Default::default(),
            "inst",
            42,
        );
        let ids: Vec<&str> = out.iter().map(|c| c.supplier_id.as_str()).collect();
        assert_eq!(ids, vec!["s1", "s3"]); // s2 at 0.02 cut
    }

    #[test]
    fn deviation_tolerance_cuts() {
        let candidates = vec![est("s1", "m1", 0.01), est("s2", "m2", 0.01)];
        let c = constraints(0.0, 10.0);
        let d = ClaimDeviation {
            supplier_id: "s2".into(),
            metric_name: "billing_accuracy".into(),
            deviation_percent: 25.0,
            ..Default::default()
        };
        let mut inner = std::collections::HashMap::new();
        inner.insert("billing_accuracy".to_string(), d);
        let mut outer = std::collections::HashMap::new();
        outer.insert("s2".to_string(), inner);
        let snap = DeviationSnapshot::new(outer, Default::default());
        let bias = BiasConfig::default();
        let out = apply_hard_filters(
            candidates,
            &c,
            &snap,
            &bias,
            &Default::default(),
            &Default::default(),
            "inst",
            42,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].supplier_id, "s1");
    }

    #[test]
    fn supplier_bias_cost_cap() {
        let candidates = vec![est("s1", "m1", 0.01), est("s1", "m2", 0.05)];
        let c = constraints(0.0, 0.0);
        let snap = DeviationSnapshot::default();
        let mut biases = std::collections::HashMap::new();
        biases.insert(
            "s1".to_string(),
            SupplierBias {
                bias_score: 0.0,
                max_cost_per_request: Some(0.02),
            },
        );
        let bias = BiasConfig {
            supplier_biases: biases,
            ..Default::default()
        };
        let out = apply_hard_filters(
            candidates,
            &c,
            &snap,
            &bias,
            &Default::default(),
            &Default::default(),
            "inst",
            42,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].model_id, "m1");
    }

    #[test]
    fn rehabilitation_deterministic_same_input() {
        // Same inputs 100x -> same decision (Python determinism test shape).
        let decisions: Vec<bool> = (0..100)
            .map(|_| {
                should_rehabilitate(
                    "s1",
                    "inst-1",
                    400, // > cooldown 300
                    42,
                    0.001,
                    300,
                )
            })
            .collect();
        assert!(decisions.iter().all(|d| *d == decisions[0]));
    }

    #[test]
    fn rehabilitation_cooldown_gate() {
        assert!(!should_rehabilitate("s1", "inst-1", 100, 42, 0.001, 300));
        // At exactly cooldown it may proceed (seed-dependent).
        let _ = should_rehabilitate("s1", "inst-1", 300, 42, 0.001, 300);
    }

    #[test]
    fn priority_cascade_prefers_higher_group() {
        let candidates = vec![est("s1", "m1", 0.01), est("s2", "m2", 0.02)];
        let groups = vec![
            PriorityGroup {
                name: "low".into(),
                priority: 1,
                models: vec!["m2".into()],
            },
            PriorityGroup {
                name: "high".into(),
                priority: 10,
                models: vec!["m1".into()],
            },
        ];
        let bias = BiasConfig {
            priority_groups: groups,
            ..Default::default()
        };
        let out = apply_priority_filter(
            candidates,
            &bias.priority_groups,
            &Default::default(),
            &Default::default(),
            "inst",
            42,
            &bias,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].model_id, "m1"); // high group wins
    }

    #[test]
    fn terminal_supplier_skipped() {
        let candidates = vec![est("s1", "m1", 0.01), est("s2", "m2", 0.01)];
        let groups = vec![PriorityGroup {
            name: "all".into(),
            priority: 5,
            models: vec!["m1".into(), "m2".into()],
        }];
        let bias = BiasConfig {
            priority_groups: groups,
            ..Default::default()
        };
        let mut states = std::collections::HashMap::new();
        states.insert("s1".to_string(), "TERMINAL".to_string());
        let out = apply_priority_filter(
            candidates,
            &bias.priority_groups,
            &states,
            &Default::default(),
            "inst",
            42,
            &bias,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].supplier_id, "s2");
    }
}
