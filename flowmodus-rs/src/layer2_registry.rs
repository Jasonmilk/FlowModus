//! Layer 2: supplier raw model registry — Ed25519-signed declarations.
//!
//! The registry records supplier-published raw prices, capabilities and
//! compliance claims verbatim, without any subjective evaluation (whitepaper
//! §5.3 L2). It is the trust base of the whole system. Signature verification
//! with ed25519-dalek lands in R-2.

/// Registry verification stub — full Ed25519 flow lands in R-2.
pub struct Registry;

impl Registry {
    /// Verify a signature array against the hardcoded committee public keys.
    /// Stub: returns `true` only for empty input (no-op placeholder).
    pub fn verify_stub(_signatures: &[String]) -> bool {
        // DNA iron law 0 / ADR-0100 D2: verification logic lands in R-2 with
        // the ed25519-dalek dependency. This stub is replaced, not extended.
        _signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_signatures_verify_ok() {
        assert!(Registry::verify_stub(&[]));
    }
}
