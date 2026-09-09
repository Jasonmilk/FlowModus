//! User bias configuration (migrated from Python v1.7 `config/bias.py`).
//!
//! Zero-hardcode: the rehabilitation defaults below carry their source —
//! Python v1.7 dataclass defaults, user-overridable, documented as such.
//! Judgement about suppliers always belongs to the user; these are local
//! knobs, never protocol judgments.

use std::collections::HashMap;

/// User-defined bias for a specific supplier.
#[derive(Clone, Debug, Default)]
pub struct SupplierBias {
    pub bias_score: f64,
    pub max_cost_per_request: Option<f64>,
}

/// User-defined priority group for channel cascading (Layer 4).
#[derive(Clone, Debug)]
pub struct PriorityGroup {
    pub name: String,
    pub priority: i32,
    pub models: Vec<String>,
}

/// A single endpoint within a user-defined routing group (Group call mode).
#[derive(Clone, Debug)]
pub struct GroupEndpoint {
    pub id: String,
    pub priority: i32,
    pub weight: u32,
}

/// User-defined routing group with prioritized, weighted endpoints.
#[derive(Clone, Debug, Default)]
pub struct RoutingGroup {
    pub description: String,
    pub endpoints: Vec<GroupEndpoint>,
}

/// Health thresholds for the failure-cooldown tracker.
/// Sources: one-api operational semantics (fail -> cooldown -> auto-disable
/// after repeated failures), user-overridable. Zero-hardcode documented.
#[derive(Clone, Debug)]
pub struct HealthConfig {
    /// Failures before a supplier is marked DEGRADED (default 1: first failure
    /// degrades — matches Layer 4's expectation).
    pub degrade_after_failures: u32,
    /// Failures before a supplier is marked TERMINAL (one-api auto-disable).
    pub terminal_after_failures: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            degrade_after_failures: 1,
            terminal_after_failures: 5,
        }
    }
}

/// Full user bias configuration.
#[derive(Clone, Debug)]
pub struct BiasConfig {
    pub supplier_biases: HashMap<String, SupplierBias>,
    pub priority_groups: Vec<PriorityGroup>,
    pub groups: HashMap<String, RoutingGroup>,
    /// Rehabilitation probability threshold.
    /// Source: Python v1.7 `BiasConfig` default (0.001 = 0.1%), user-overridable.
    pub rehabilitation_probability: f64,
    /// Minimum seconds before a degraded supplier can be rehabilitated.
    /// Source: Python v1.7 `BiasConfig` default (300 = 5 min), user-overridable.
    pub rehabilitation_cooldown_seconds: i64,
    /// Soft priority bonus for free-tier suppliers (billing all-zero =
    /// physical fact). Source: Helix survival philosophy — 0 tokens first >
    /// fewer tokens/small LLM > more tokens/big LLM. Soft: free wins when
    /// available and healthy; paid still wins when free cannot satisfy.
    /// User-overridable (度量衡: judgement belongs to the user).
    pub free_tier_bonus: f64,
}

impl Default for BiasConfig {
    fn default() -> Self {
        Self {
            supplier_biases: HashMap::new(),
            priority_groups: Vec::new(),
            groups: HashMap::new(),
            rehabilitation_probability: 0.001,
            rehabilitation_cooldown_seconds: 300,
            free_tier_bonus: 5.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_carry_python_semantics() {
        let c = BiasConfig::default();
        assert!((c.rehabilitation_probability - 0.001).abs() < 1e-12);
        assert_eq!(c.rehabilitation_cooldown_seconds, 300);
        assert!(c.supplier_biases.is_empty());
        assert!(c.priority_groups.is_empty());
    }
}
