//! Telemetry — parasitic collector (whitepaper §4, iron law 0: zero timing
//! probes, zero heartbeat). Feeds on REAL traffic only: request/response
//! outcomes recorded after the fact; nothing is ever probed.
//!
//! Python v1.7 persisted samples to SQLite; rs keeps an in-memory append-only
//! sample log + derived aggregates (extreme minimalism — persistence is a
//! sidecar concern, deferred; samples are already fully structured via the
//! immutable metrics contract).

use crate::pb::{ClaimDeviation, TelemetrySample};

/// Classify an HTTP outcome into "success" / "retryable" / "terminal"
/// (Python v1.7 `classify_http_error` semantics, verbatim).
pub fn classify_http_error(status_code: i32, connect_error: bool) -> &'static str {
    if (200..300).contains(&status_code) {
        return "success";
    }
    if matches!(status_code, 429 | 500 | 502 | 503 | 504) {
        return "retryable";
    }
    if connect_error {
        return "retryable";
    }
    if matches!(status_code, 401 | 403 | 400) {
        return "terminal";
    }
    // unknown errors are retryable once
    "retryable"
}

/// Cache-hit detection from response headers (Python `_check_cache_hit`).
pub fn check_cache_hit(x_cache: Option<&str>, anthropic_cache: Option<&str>) -> bool {
    x_cache == Some("HIT") || anthropic_cache == Some("hit")
}

/// Parasitic telemetry collector: appends samples, derives aggregates.
#[derive(Clone, Debug, Default)]
pub struct Collector {
    samples: Vec<TelemetrySample>,
}

impl Collector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a real traffic outcome (pure state append).
    #[allow(clippy::too_many_arguments)]
    pub fn collect_from_response(
        &mut self,
        supplier_id: &str,
        model_id: &str,
        status_code: i32,
        ttft_ms: f32,
        total_duration_ms: f32,
        x_cache: Option<&str>,
        anthropic_cache: Option<&str>,
        billed_tokens: i32,
        connect_error: bool,
        sample_time_unix: i64,
    ) -> TelemetrySample {
        let sample = TelemetrySample {
            supplier_id: supplier_id.to_string(),
            model_id: model_id.to_string(),
            sample_time_unix,
            ttft_ms,
            total_duration_ms,
            kv_cache_hit: check_cache_hit(x_cache, anthropic_cache),
            billed_tokens,
            status_code,
            health_state: classify_http_error(status_code, connect_error).to_string(),
        };
        self.samples.push(sample.clone());
        sample
    }

    pub fn samples(&self) -> &[TelemetrySample] {
        &self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Aggregate: per-supplier health state counts (derived, pure).
    pub fn health_counts(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, usize>> {
        let mut out: std::collections::HashMap<String, std::collections::HashMap<String, usize>> =
            std::collections::HashMap::new();
        for s in &self.samples {
            out.entry(s.supplier_id.clone())
                .or_default()
                .entry(s.health_state.clone())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        out
    }

    /// Aggregate: per-supplier cache hit rate (derived, pure).
    pub fn cache_hit_rate(&self, supplier_id: &str) -> Option<f64> {
        let all: Vec<&TelemetrySample> = self
            .samples
            .iter()
            .filter(|s| s.supplier_id == supplier_id)
            .collect();
        if all.is_empty() {
            return None;
        }
        let hits = all.iter().filter(|s| s.kv_cache_hit).count();
        Some(hits as f64 / all.len() as f64)
    }

    /// Aggregate: per-supplier mean deviation samples (deviation records are
    /// written by the settlement flow; here exposed as a snapshot shape
    /// matching Python's DeviationSnapshot consumer contract).
    pub fn snapshot(&self) -> Vec<ClaimDeviation> {
        Vec::new() // populated by settlement (layer2.5) in the live loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_status_codes() {
        assert_eq!(classify_http_error(200, false), "success");
        assert_eq!(classify_http_error(299, false), "success");
        assert_eq!(classify_http_error(429, false), "retryable");
        assert_eq!(classify_http_error(503, false), "retryable");
        assert_eq!(classify_http_error(401, false), "terminal");
        assert_eq!(classify_http_error(403, false), "terminal");
        assert_eq!(classify_http_error(400, false), "terminal");
        assert_eq!(classify_http_error(404, false), "retryable");
        assert_eq!(classify_http_error(500, true), "retryable");
    }

    #[test]
    fn connect_error_retryable() {
        assert_eq!(classify_http_error(0, true), "retryable");
    }

    #[test]
    fn cache_hit_headers() {
        assert!(check_cache_hit(Some("HIT"), None));
        assert!(check_cache_hit(None, Some("hit")));
        assert!(!check_cache_hit(Some("MISS"), None));
        assert!(!check_cache_hit(None, None));
    }

    #[test]
    fn collector_appends_and_aggregates() {
        let mut c = Collector::new();
        c.collect_from_response("s1", "m1", 200, 12.5, 300.0, Some("HIT"), None, 100, false, 1);
        c.collect_from_response("s1", "m1", 429, 0.0, 50.0, None, None, 0, false, 2);
        c.collect_from_response("s1", "m1", 200, 10.0, 250.0, Some("HIT"), None, 90, false, 3);
        assert_eq!(c.len(), 3);
        let counts = c.health_counts();
        assert_eq!(counts["s1"]["success"], 2);
        assert_eq!(counts["s1"]["retryable"], 1);
        assert_eq!(c.cache_hit_rate("s1"), Some(2.0 / 3.0));
        assert_eq!(c.cache_hit_rate("nobody"), None);
    }

    #[test]
    fn sample_fields_carried() {
        let mut c = Collector::new();
        let s = c.collect_from_response("s9", "m9", 401, 5.0, 100.0, None, None, 0, false, 42);
        assert_eq!(s.supplier_id, "s9");
        assert_eq!(s.sample_time_unix, 42);
        assert_eq!(s.health_state, "terminal");
        assert!(!s.kv_cache_hit);
    }
}
