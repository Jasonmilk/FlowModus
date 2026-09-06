//! Health tracker — failure cooldown + recovery (one-api operational
//! semantics, ADR-0100 D3). 100% parasitic: fed ONLY by real traffic results
//! (success/failure of actual requests); zero probing, zero heartbeat
//! (iron law 0). Layer 4 consumes the resulting states + timestamps.
//!
//! Semantics:
//! - success -> HEALTHY, consecutive-failure counter reset
//! - failure  -> DEGRADED after `degrade_after_failures` (default 1)
//! - repeated failure -> TERMINAL after `terminal_after_failures` (default 5,
//!   one-api auto-disable); Layer 4 skips TERMINAL suppliers entirely
//! - recovery: Layer 4's `should_rehabilitate` probabilistically re-admits a
//!   DEGRADED supplier after the cooldown window (user-configured)

use crate::config::HealthConfig;
use std::collections::HashMap;

/// Health state label, matching the strings Layer 4 filters on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Terminal,
}

impl HealthState {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthState::Healthy => "HEALTHY",
            HealthState::Degraded => "DEGRADED",
            HealthState::Terminal => "TERMINAL",
        }
    }
}

/// Parasitic health tracker. `current_minute` is wall-clock minutes
/// (`unix_secs / 60`), the same scale Layer 4 uses for cooldown math.
#[derive(Clone, Debug)]
pub struct HealthTracker {
    states: HashMap<String, HealthState>,
    /// Degradation timestamp in SECONDS (minute * 60), Layer 4's scale.
    timestamps: HashMap<String, i64>,
    failures: HashMap<String, u32>,
    config: HealthConfig,
}

impl HealthTracker {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            states: HashMap::new(),
            timestamps: HashMap::new(),
            failures: HashMap::new(),
            config,
        }
    }

    /// Record a real traffic outcome (pure state transition, parasitic).
    pub fn record(&mut self, supplier_id: &str, ok: bool, current_minute: i64) {
        let failures = self.failures.entry(supplier_id.to_string()).or_insert(0);
        if ok {
            *failures = 0;
            self.states.insert(supplier_id.to_string(), HealthState::Healthy);
            return;
        }
        *failures += 1;
        let n = *failures;
        let state = if n >= self.config.terminal_after_failures {
            HealthState::Terminal
        } else if n >= self.config.degrade_after_failures {
            HealthState::Degraded
        } else {
            HealthState::Healthy
        };
        self.states.insert(supplier_id.to_string(), state);
        if state == HealthState::Degraded {
            self.timestamps
                .insert(supplier_id.to_string(), current_minute * 60);
        }
    }

    pub fn state(&self, supplier_id: &str) -> HealthState {
        self.states.get(supplier_id).copied().unwrap_or(HealthState::Healthy)
    }

    /// Snapshot for Layer 4 (supplier_id -> "HEALTHY"/"DEGRADED"/"TERMINAL").
    pub fn states_map(&self) -> HashMap<String, String> {
        self.states
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().to_string()))
            .collect()
    }

    /// Snapshot for Layer 4 (supplier_id -> degradation minute*60).
    pub fn timestamps_map(&self) -> HashMap<String, i64> {
        self.timestamps.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_keeps_healthy() {
        let mut t = HealthTracker::new(HealthConfig::default());
        t.record("s1", true, 100);
        assert_eq!(t.state("s1"), HealthState::Healthy);
        assert!(t.timestamps_map().is_empty());
    }

    #[test]
    fn first_failure_degrades() {
        let mut t = HealthTracker::new(HealthConfig::default());
        t.record("s1", false, 100);
        assert_eq!(t.state("s1"), HealthState::Degraded);
        assert_eq!(t.timestamps_map().get("s1").copied(), Some(100 * 60));
    }

    #[test]
    fn repeated_failures_terminal() {
        let mut t = HealthTracker::new(HealthConfig::default());
        for _ in 0..5 {
            t.record("s1", false, 100);
        }
        assert_eq!(t.state("s1"), HealthState::Terminal);
    }

    #[test]
    fn recovery_resets_on_success() {
        let mut t = HealthTracker::new(HealthConfig::default());
        t.record("s1", false, 100);
        t.record("s1", false, 101);
        t.record("s1", true, 102);
        assert_eq!(t.state("s1"), HealthState::Healthy);
        // one more failure starts the degrade count again (failures reset)
        t.record("s1", false, 103);
        assert_eq!(t.state("s1"), HealthState::Degraded);
    }

    #[test]
    fn unknown_supplier_healthy() {
        let t = HealthTracker::new(HealthConfig::default());
        assert_eq!(t.state("nobody"), HealthState::Healthy);
    }
}
