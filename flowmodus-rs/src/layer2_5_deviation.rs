//! Layer 2.5: deterministic baseline — declaration deviation (measure/weight
//! calibration). Behavior aligned with Python v1.7 `telemetry/deviation.py`.
//!
//! Data source per ADR-0100 D2: parasitic telemetry from real traffic +
//! user-explicit on-demand probing. NO timing heartbeat (iron law 0).
//! Deviation is pure math, published automatically, NEVER rated.

use crate::pb::ClaimDeviation;
use std::collections::HashMap;

/// Deviation percentage between declared and actual values.
/// Python v1.7 semantics: claimed == 0 -> 0.0 if actual == 0, else |actual|*100.
pub fn deviation_percent(claimed: f64, actual: f64) -> f64 {
    if claimed == 0.0 {
        if actual == 0.0 {
            0.0
        } else {
            actual.abs() * 100.0
        }
    } else {
        (actual - claimed) / claimed * 100.0
    }
}

/// Weighted-average settlement deviation over a sliding window.
/// Newer samples carry higher weight (weight = index + 1). Python semantics.
pub fn settlement_deviation(window: &[f64]) -> f64 {
    if window.is_empty() {
        return 0.0;
    }
    let mut total_weight = 0.0;
    let mut weighted_sum = 0.0;
    for (i, deviation) in window.iter().enumerate() {
        let weight = (i + 1) as f64;
        weighted_sum += deviation * weight;
        total_weight += weight;
    }
    weighted_sum / total_weight
}

/// Immutable snapshot of deviation data, injected into the pipeline by the
/// control plane. Python v1.7 kept one duck-typed dict serving both lookups;
/// rs makes the shapes explicit (type-safe, behavior-equivalent):
/// - `deviations`: supplier_id -> (metric_name -> ClaimDeviation), for Layer 4
/// - `hit_rates`:  model_id -> kv-cache hit rate, for Layer 3 cost savings
#[derive(Clone, Debug, Default)]
pub struct DeviationSnapshot {
    deviations: HashMap<String, HashMap<String, ClaimDeviation>>,
    hit_rates: HashMap<String, f64>,
}

impl DeviationSnapshot {
    pub fn new(
        deviations: HashMap<String, HashMap<String, ClaimDeviation>>,
        hit_rates: HashMap<String, f64>,
    ) -> Self {
        Self {
            deviations,
            hit_rates,
        }
    }

    pub fn with_hit_rates(hit_rates: HashMap<String, f64>) -> Self {
        Self {
            deviations: HashMap::new(),
            hit_rates,
        }
    }

    /// O(1) lookup of a supplier's claim deviation for a metric.
    pub fn get_deviation(&self, supplier_id: &str, metric_name: &str) -> Option<&ClaimDeviation> {
        self.deviations
            .get(supplier_id)
            .and_then(|m| m.get(metric_name))
    }

    /// O(1) lookup of a model's kv-cache hit rate (default 0.0).
    pub fn kv_cache_hit_rate(&self, model_id: &str) -> f64 {
        self.hit_rates.get(model_id).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deviation_basic() {
        assert!((deviation_percent(100.0, 110.0) - 10.0).abs() < 1e-9);
        assert!((deviation_percent(100.0, 90.0) + 10.0).abs() < 1e-9);
    }

    #[test]
    fn deviation_zero_claimed() {
        assert_eq!(deviation_percent(0.0, 0.0), 0.0);
        // Python: claimed == 0, actual != 0 -> |actual| * 100
        assert!((deviation_percent(0.0, 5.0) - 500.0).abs() < 1e-9);
        assert!((deviation_percent(0.0, -3.0) - 300.0).abs() < 1e-9);
    }

    #[test]
    fn settlement_weighted() {
        assert_eq!(settlement_deviation(&[]), 0.0);
        // window [1, 2]: weights 1,2 -> (1*1 + 2*2)/3 = 5/3
        assert!((settlement_deviation(&[1.0, 2.0]) - 5.0 / 3.0).abs() < 1e-9);
        // single sample: weight 1 -> value itself
        assert_eq!(settlement_deviation(&[7.5]), 7.5);
    }

    #[test]
    fn snapshot_defaults_zero() {
        let snap = DeviationSnapshot::default();
        assert_eq!(snap.kv_cache_hit_rate("missing"), 0.0);
        assert!(snap.get_deviation("missing", "billing_accuracy").is_none());
    }

    #[test]
    fn snapshot_lookup() {
        let mut m = HashMap::new();
        m.insert("m1".to_string(), 0.8);
        let snap = DeviationSnapshot::with_hit_rates(m);
        assert!((snap.kv_cache_hit_rate("m1") - 0.8).abs() < 1e-9);
    }

    #[test]
    fn snapshot_deviation_lookup() {
        let d = ClaimDeviation {
            supplier_id: "s1".into(),
            metric_name: "billing_accuracy".into(),
            deviation_percent: 3.5,
            ..Default::default()
        };
        let mut inner = HashMap::new();
        inner.insert("billing_accuracy".to_string(), d);
        let mut outer = HashMap::new();
        outer.insert("s1".to_string(), inner);
        let snap = DeviationSnapshot::new(outer, HashMap::new());
        let got = snap.get_deviation("s1", "billing_accuracy").expect("dev");
        assert!((got.deviation_percent - 3.5).abs() < 1e-9);
    }
}
