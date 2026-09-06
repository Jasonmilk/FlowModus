//! Layer 1: protocol & measure specification — STE (Standard Token Equivalent).
//!
//! STE collapses heterogeneous token/billing definitions (per-token, per-time,
//! per-image-frame) into one comparable scale. Per whitepaper §5.3 L1, STE is a
//! comparison aid for cross-supplier quotes; it never enters audit logic.
//! Full conversion lands in R-2; this module only declares the measure contract.

/// Measure contract marker for the STE scale (1 STE = baseline tokenizer unit).
pub struct SteScale;

impl SteScale {
    /// The canonical baseline: 1 STE is defined by the protocol's reference
    /// tokenizer. Exact tokenizer integration lands in R-2.
    pub fn baseline_unit() -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_is_one() {
        assert_eq!(SteScale::baseline_unit(), 1);
    }
}
