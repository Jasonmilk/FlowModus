//! Control plane: canonicalizer, verifier, anti-corruption.
//!
//! - Canonicalizer: deterministic byte normalization (UTF-8 NFC, key-sorted
//!   JSON, no BOM) before signing/verification (whitepaper §2.3).
//! - Verifier: Ed25519 signature verification against hardcoded committee keys.
//! - Anti-corruption: defense-in-depth against injection (whitepaper §3.3).

/// Canonicalizer stub — NFC normalization lands in R-2.
pub struct Canonicalizer;

impl Canonicalizer {
    /// Stub: pass-through.
    pub fn canonicalize_stub(input: &str) -> String {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_passes_through() {
        assert_eq!(Canonicalizer::canonicalize_stub("abc"), "abc");
    }
}
