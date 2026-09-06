//! Layer 1: protocol & measure specification — STE (Standard Token Equivalent).
//!
//! STE collapses heterogeneous token/billing definitions into one comparable
//! scale (whitepaper §5.3 L1). Behavior aligned with Python v1.7:
//! `estimate_token_count` heuristic, `calculate_ste`, `parse_cache_breakpoints`,
//! `normalize_request`.

use crate::pb;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Estimate token count from raw text (simple heuristic, no external
/// tokenizer). Approximate; final billing relies on the supplier's actual
/// usage response. Python v1.7: ascii/4 + non-ascii/1.5, floor, min 1.
pub fn estimate_token_count(text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let ascii = text.chars().filter(|c| (*c as u32) < 128).count();
    let non_ascii = text.chars().count() - ascii;
    let est = ascii as f64 / 4.0 + non_ascii as f64 / 1.5;
    est.max(1.0) as i32
}

/// STE = token_count * compression_ratio (pure).
pub fn calculate_ste(token_count: i32, compression_ratio: f64) -> f64 {
    token_count as f64 * compression_ratio
}

/// Extract cache breakpoints from extra headers (`x-flowmodus-cache-breakpoints`).
pub fn parse_cache_breakpoints(headers: &HashMap<String, String>) -> Vec<i32> {
    let raw = match headers.get("x-flowmodus-cache-breakpoints") {
        Some(v) => v,
        None => return Vec::new(),
    };
    if raw.trim().is_empty() {
        return Vec::new();
    }
    raw.split(',')
        .filter_map(|v| {
            let t = v.trim();
            if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit()) {
                None
            } else {
                t.parse::<i32>().ok()
            }
        })
        .collect()
}

/// Normalize a raw user request into a standard normalized request (pure).
/// `tokenizer_ratios` supplies the "default" compression ratio (Python shape).
pub fn normalize_request(
    raw: &pb::RawRequest,
    tokenizer_ratios: &HashMap<String, f64>,
) -> pb::NormalizedRequest {
    let mut hasher = Sha256::new();
    hasher.update(raw.prompt.as_bytes());
    let prompt_hash = format!("{:x}", hasher.finalize());

    let estimated_input_tokens = estimate_token_count(&raw.prompt);
    let default_ratio = tokenizer_ratios.get("default").copied().unwrap_or(1.0);
    let ste_input = calculate_ste(estimated_input_tokens, default_ratio);
    let ste_output_estimated = calculate_ste(raw.max_output_tokens, default_ratio);

    pb::NormalizedRequest {
        prompt_hash,
        estimated_input_tokens,
        max_output_tokens: raw.max_output_tokens,
        agent_role: raw.agent_role.clone(),
        cognitive_mode: raw.cognitive_mode.clone(),
        ste_input: ste_input as f32,
        ste_output_estimated: ste_output_estimated as f32,
        cache_breakpoints: parse_cache_breakpoints(&raw.extra_headers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn ste_baseline() {
        assert_eq!(calculate_ste(100, 1.0), 100.0);
        assert_eq!(calculate_ste(100, 1.5), 150.0);
        assert_eq!(calculate_ste(100, 0.5), 50.0);
        assert_eq!(calculate_ste(0, 1.0), 0.0);
        assert_eq!(calculate_ste(100, 0.0), 0.0);
        assert_eq!(calculate_ste(100, -1.0), -100.0);
    }

    #[test]
    fn ste_determinism() {
        let r: Vec<f64> = (0..100).map(|_| calculate_ste(42, 1.337)).collect();
        assert!(r.iter().all(|v| *v == r[0]));
        assert!((r[0] - 42.0 * 1.337).abs() < 1e-9);
    }

    #[test]
    fn token_estimate_ascii_and_mixed() {
        // Python: ascii/4 + non-ascii/1.5
        assert_eq!(estimate_token_count(""), 0);
        assert_eq!(estimate_token_count("abcd"), 1); // 4/4 = 1
        assert_eq!(estimate_token_count("中文"), 1); // 2/1.5=1.33 -> floor 1, max(1,1)=1 (Python)
        // ascii 8 -> 2, non-ascii 0 -> 0, est 2.0
        assert_eq!(estimate_token_count("abcdefgh"), 2);
        // 1 ascii + 2 non-ascii: 1/4 + 2/1.5 = 0.25 + 1.333 = 1.583 -> 1
        assert_eq!(estimate_token_count("a中文"), 1);
    }

    #[test]
    fn breakpoints_parsing() {
        assert_eq!(parse_cache_breakpoints(&headers(&[])), vec![]);
        assert_eq!(
            parse_cache_breakpoints(&headers(&[("x-flowmodus-cache-breakpoints", "10,20,30")])),
            vec![10, 20, 30]
        );
        assert_eq!(
            parse_cache_breakpoints(&headers(&[("x-flowmodus-cache-breakpoints", " 5 , 15 ")]))[..],
            vec![5, 15][..]
        );
        // non-digit entries are dropped by Python's isdigit guard
        assert_eq!(
            parse_cache_breakpoints(&headers(&[("x-flowmodus-cache-breakpoints", "10,abc,20")])),
            vec![10, 20]
        );
        // negative numbers rejected (isdigit false)
        assert_eq!(
            parse_cache_breakpoints(&headers(&[("x-flowmodus-cache-breakpoints", "-5,10")])),
            vec![10]
        );
    }

    #[test]
    fn normalize_request_shape() {
        let raw = pb::RawRequest {
            prompt: "hello world".into(),
            agent_role: "code-generation".into(),
            cognitive_mode: "partner".into(),
            max_output_tokens: 512,
            extra_headers: headers(&[("x-flowmodus-cache-breakpoints", "100,200")]),
        };
        let mut ratios = HashMap::new();
        ratios.insert("default".to_string(), 1.0);
        let n = normalize_request(&raw, &ratios);
        assert_eq!(n.estimated_input_tokens, 2); // "hello world" = 11 ascii / 4 = 2.75 -> 2
        assert_eq!(n.ste_input, 2.0);
        assert_eq!(n.ste_output_estimated, 512.0);
        assert_eq!(n.agent_role, "code-generation");
        assert_eq!(n.cache_breakpoints, vec![100, 200]);
        assert_eq!(n.prompt_hash.len(), 64); // sha256 hex
    }

    #[test]
    fn normalize_deterministic() {
        let raw = pb::RawRequest {
            prompt: "the quick brown fox".into(),
            agent_role: String::new(),
            cognitive_mode: String::new(),
            max_output_tokens: 64,
            extra_headers: HashMap::new(),
        };
        let mut ratios = HashMap::new();
        ratios.insert("default".to_string(), 1.0);
        let a = normalize_request(&raw, &ratios);
        let b = normalize_request(&raw, &ratios);
        assert_eq!(a.prompt_hash, b.prompt_hash);
    }
}
