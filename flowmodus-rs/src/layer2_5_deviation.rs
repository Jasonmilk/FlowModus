//! Layer 2.5: deterministic baseline — declaration deviation.
//!
//! Deviation is a pure-math percentage between declared and actual values.
//! It is computed automatically, published automatically, and NEVER rated
//! (whitepaper §5.6). Data source per ADR-0100 D2: parasitic telemetry from
//! real traffic + user-explicit on-demand probing. NO timing heartbeat.

/// Deviation math: (actual - claimed) / claimed, as a signed percentage.
pub fn deviation_percent(claimed: f64, actual: f64) -> f64 {
    if claimed == 0.0 {
        0.0
    } else {
        (actual - claimed) / claimed * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deviation_positive_when_actual_exceeds_claimed() {
        assert!((deviation_percent(100.0, 110.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn deviation_zero_when_claimed_is_zero() {
        assert_eq!(deviation_percent(0.0, 5.0), 0.0);
    }
}
