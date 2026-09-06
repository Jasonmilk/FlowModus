//! Telemetry: collector + deviation.
//!
//! Collector is 100% parasitic on real traffic (DNA iron law 0 / ADR-0100 D2):
//! it records what real requests already produced — no polling, no heartbeat.
//! Deviation computes declaration-vs-actual percentages (see layer2_5_deviation).

/// Telemetry collector stub — parasitic sampling lands in R-2.
pub struct Collector;

impl Collector {
    /// Stub: no-op record.
    pub fn record_stub() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_records_silently() {
        Collector::record_stub();
    }
}
