//! Layer 5: agent intent & dynamic preference — soft weighting + entropy routing.
//!
//! A deterministic multi-dimensional dot-product scores the surviving candidates;
//! declaration deviation acts as a weight-correction factor (non-overriding,
//! non-punishing, pure math). Entropy routing (ADR-0100 D4) decorrelates via
//! instance-id: same instance + same input = bit-identical output; different
//! instances spread to avoid herd traffic.

/// Soft-weight scoring stub — full dot-product lands in R-2.
pub struct Scorer;

impl Scorer {
    /// Stub: identity score.
    pub fn score_stub(x: f64) -> f64 {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_is_identity() {
        assert_eq!(Scorer::score_stub(3.0), 3.0);
    }
}
