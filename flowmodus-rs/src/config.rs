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

/// Full user bias configuration.
#[derive(Clone, Debug)]
pub struct BiasConfig {
    pub supplier_biases: HashMap<String, SupplierBias>,
    pub priority_groups: Vec<PriorityGroup>,
    /// Rehabilitation probability threshold.
    /// Source: Python v1.7 `BiasConfig` default (0.001 = 0.1%), user-overridable.
    pub rehabilitation_probability: f64,
    /// Minimum seconds before a degraded supplier can be rehabilitated.
    /// Source: Python v1.7 `BiasConfig` default (300 = 5 min), user-overridable.
    pub rehabilitation_cooldown_seconds: i64,
}

impl Default for BiasConfig {
    fn default() -> Self {
        Self {
            supplier_biases: HashMap::new(),
            priority_groups: Vec::new(),
            rehabilitation_probability: 0.001,
            rehabilitation_cooldown_seconds: 300,
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
