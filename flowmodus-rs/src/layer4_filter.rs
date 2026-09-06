//! Layer 4: user preferences & hard boundaries — the hard filter.
//!
//! Local config constraints (budget / region / residency / user rules) ruthlessly
//! cut every supplier that does not qualify; filtered suppliers never reach
//! Layer 5 (whitepaper §5.3 L4). Failure cooldown + recovery (one-api-derived
//! semantics, ADR-0100 D3) also gates here — deterministically, from parasitic
//! telemetry, never from probing.

/// Hard filter stub — constraint evaluation lands in R-2.
pub struct HardFilter;

impl HardFilter {
    /// Decision: keep (`true`) or cut (`false`). Stub keeps everything.
    pub fn keep_stub() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_keeps_all() {
        assert!(HardFilter::keep_stub());
    }
}
