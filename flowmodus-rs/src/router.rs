//! Call-mode router — Manual / Group / Auto dispatch (migrated from Python
//! v1.7 `lifecycle.resolve_routing_decision`, with Group realized for real:
//! Python left `_group_decision` as a stub delegating to auto).
//!
//! - Manual: direct dispatch to a named model, zero pipeline overhead
//! - Group:  user routing group — priority-desc, then deterministic
//!   weight sampling among top-priority endpoints (ADR-0100 D4)
//! - Auto:   the full five-layer deterministic pipeline
//!
//! All decisions are pure: given the same inputs, the same decision is
//! produced every time, on every process.

use crate::config::{BiasConfig, GroupEndpoint};
use crate::health::HealthTracker;
use crate::layer1_normalize::normalize_request;
use crate::layer2_registry::get_eligible_suppliers;
use crate::layer2_5_deviation::DeviationSnapshot;
use crate::layer3_cost::estimate_all;
use crate::layer4_filter::apply_hard_filters;
use crate::layer5_score::{deterministic_hash, score_and_entropy_sample};
use crate::pb::{self, RawRequest, RoutingDecision, SupplierDeclaration, UserConstraints};
use std::collections::HashMap;

/// The three call modes (Python v1.7 `resolve_routing_decision` dispatch).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallMode {
    Manual(String),
    Group(String),
    Auto,
}

impl CallMode {
    /// Parse the `model` request parameter.
    /// Manual: non-empty, not "group:*", not "auto".
    /// Group:  starts with "group:".
    /// Auto:   anything else (empty or "auto").
    pub fn parse(model_param: &str) -> CallMode {
        if !model_param.is_empty() && !model_param.starts_with("group:") && model_param != "auto" {
            CallMode::Manual(model_param.to_string())
        } else if let Some(name) = model_param.strip_prefix("group:") {
            CallMode::Group(name.to_string())
        } else {
            CallMode::Auto
        }
    }
}

/// Resolve a model's supplier + endpoint from the registry (Python helpers).
fn resolve_model<'a>(
    registry: &'a [SupplierDeclaration],
    model_id: &str,
) -> Option<(&'a SupplierDeclaration, &'a pb::ModelDeclaration)> {
    registry.iter().find_map(|supplier| {
        supplier
            .models
            .iter()
            .find(|m| m.model_id == model_id)
            .map(|m| (supplier, m))
    })
}

/// First declared endpoint URL, trailing slash stripped (Python behavior).
fn endpoint_url(supplier: &SupplierDeclaration) -> String {
    supplier
        .endpoints
        .first()
        .map(|e| e.base_url.trim_end_matches('/').to_string())
        .unwrap_or_default()
}

/// Deterministic weighted pick among top-priority endpoints.
/// Same instance_id + same group -> same pick; different instances decorrelate.
fn pick_weighted<'a>(
    endpoints: &'a [GroupEndpoint],
    instance_id: &str,
    group_name: &str,
) -> &'a GroupEndpoint {
    let max_priority = endpoints
        .iter()
        .map(|e| e.priority)
        .max()
        .expect("non-empty endpoints");
    let top: Vec<&GroupEndpoint> = endpoints
        .iter()
        .filter(|e| e.priority == max_priority)
        .collect();
    if top.len() == 1 {
        return top[0];
    }
    let total_weight: u64 = top.iter().map(|e| e.weight as u64).sum();
    let seed = deterministic_hash(instance_id, &format!("group:{group_name}"));
    let threshold = seed % total_weight;
    let mut cumulative = 0u64;
    for e in &top {
        cumulative += e.weight as u64;
        if threshold < cumulative {
            return e;
        }
    }
    top.last().expect("non-empty top")
}

/// Router with all pipeline inputs injected (pure decision maker).
pub struct Router<'a> {
    registry: &'a [SupplierDeclaration],
    tokenizer_ratios: &'a HashMap<String, f64>,
    deviation: DeviationSnapshot,
    bias: &'a BiasConfig,
    constraints: UserConstraints,
    health: &'a HealthTracker,
    instance_id: &'a str,
    current_minute: i64,
}

impl<'a> Router<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry: &'a [SupplierDeclaration],
        tokenizer_ratios: &'a HashMap<String, f64>,
        deviation: DeviationSnapshot,
        bias: &'a BiasConfig,
        constraints: UserConstraints,
        health: &'a HealthTracker,
        instance_id: &'a str,
        current_minute: i64,
    ) -> Self {
        Self {
            registry,
            tokenizer_ratios,
            deviation,
            bias,
            constraints,
            health,
            instance_id,
            current_minute,
        }
    }

    /// Resolve a routing decision for a raw request (pure).
    pub fn resolve(
        &self,
        request: &RawRequest,
        model_param: &str,
    ) -> Result<RoutingDecision, String> {
        match CallMode::parse(model_param) {
            CallMode::Manual(model_id) => self.manual(&model_id),
            CallMode::Group(name) => match self.group(&name) {
                Some(d) => Ok(d),
                // unknown/empty group falls back to Auto (Python stub shape)
                None => Ok(self.auto(request)),
            },
            CallMode::Auto => Ok(self.auto(request)),
        }
    }

    /// Manual: direct dispatch to a named model, zero pipeline overhead.
    pub fn manual(&self, model_id: &str) -> Result<RoutingDecision, String> {
        let (supplier, _) =
            resolve_model(self.registry, model_id).ok_or_else(|| format!("Model '{model_id}' not found in registry"))?;
        Ok(RoutingDecision {
            supplier_id: supplier.supplier_id.clone(),
            model_id: model_id.to_string(),
            endpoint_url: endpoint_url(supplier),
            estimated_cost_usd: 0.0,
            request_id: model_id.to_string(),
            ..Default::default()
        })
    }

    /// Group: user routing group — priority-desc, then deterministic weight
    /// sampling. Unknown/empty group falls back to Auto (Python stub shape).
    pub fn group(&self, name: &str) -> Option<RoutingDecision> {
        let group = self.bias.groups.get(name)?;
        if group.endpoints.is_empty() {
            return None;
        }
        let picked = pick_weighted(&group.endpoints, self.instance_id, name);
        let (supplier, model) = resolve_model(self.registry, &picked.id)?;
        Some(RoutingDecision {
            supplier_id: supplier.supplier_id.clone(),
            model_id: model.model_id.clone(),
            endpoint_url: endpoint_url(supplier),
            estimated_cost_usd: 0.0,
            request_id: format!("group:{name}"),
            ..Default::default()
        })
    }

    /// Auto: the full five-layer deterministic pipeline
    /// (Python v1.7 `_auto_decision` shape, incl. score = 0.0 candidates).
    pub fn auto(&self, request: &RawRequest) -> RoutingDecision {
        let normalized = normalize_request(request, self.tokenizer_ratios);
        let eligible = get_eligible_suppliers(self.registry, &normalized);
        let estimates = estimate_all(&normalized, &eligible, &self.deviation);
        let filtered = apply_hard_filters(
            estimates,
            &self.constraints,
            &self.deviation,
            self.bias,
            &self.health.states_map(),
            &self.health.timestamps_map(),
            self.instance_id,
            self.current_minute,
        );
        let eligible_suppliers: Vec<pb::EligibleSupplier> = filtered
            .into_iter()
            .map(|c| pb::EligibleSupplier {
                supplier_id: c.supplier_id.clone(),
                model_id: c.model_id.clone(),
                endpoint_url: endpoint_url_by_ids(self.registry, &c.supplier_id, &c.model_id),
                score: 0.0, // Python v1.7 create_eligible_supplier semantics
                cost: Some(c),
                ..Default::default()
            })
            .collect();
        let free_ids: std::collections::HashSet<String> = self
            .registry
            .iter()
            .filter(|s| crate::registry::is_free_supplier(s))
            .map(|s| s.supplier_id.clone())
            .collect();
        score_and_entropy_sample(
            &eligible_suppliers,
            &normalized.agent_role,
            self.instance_id,
            &normalized.prompt_hash,
            self.bias,
            &free_ids,
        )
    }
}

/// Endpoint URL for a (supplier, model) pair (Python `get_endpoint_url`).
fn endpoint_url_by_ids(registry: &[SupplierDeclaration], supplier_id: &str, _model_id: &str) -> String {
    registry
        .iter()
        .find(|s| s.supplier_id == supplier_id)
        .map(endpoint_url)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Vec<SupplierDeclaration> {
        let model = |id: &str, role: &str| pb::ModelDeclaration {
            model_id: id.into(),
            agent_roles: vec![role.into()],
            billing: Some(pb::BillingDeclaration {
                token_input: 0.1,
                token_output: 0.4,
                ..Default::default()
            }),
            capabilities: Some(pb::CapabilitiesDeclaration {
                context_window: 65536,
                ..Default::default()
            }),
            kv_cache: Some(pb::KvCacheDeclaration {
                supported: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        let endpoint = |url: &str| pb::EndpointDeclaration {
            base_url: url.into(),
            ..Default::default()
        };
        vec![
            pb::SupplierDeclaration {
                supplier_id: "s1".into(),
                models: vec![model("s1-fast", "code-generation")],
                endpoints: vec![endpoint("https://s1.example/v1/")],
                ..Default::default()
            },
            pb::SupplierDeclaration {
                supplier_id: "s2".into(),
                models: vec![model("s2-cheap", "code-generation")],
                endpoints: vec![endpoint("https://s2.example/v1/")],
                ..Default::default()
            },
        ]
    }

    fn router<'a>(
        registry: &'a [SupplierDeclaration],
        ratios: &'a HashMap<String, f64>,
        bias: &'a BiasConfig,
        health: &'a HealthTracker,
    ) -> Router<'a> {
        Router::new(
            registry,
            ratios,
            DeviationSnapshot::default(),
            bias,
            UserConstraints::default(),
            health,
            "inst-1",
            100,
        )
    }

    fn default_ratios() -> HashMap<String, f64> {
        let mut ratios = HashMap::new();
        ratios.insert("default".to_string(), 1.0);
        ratios
    }

    #[test]
    fn call_mode_parse() {
        assert_eq!(CallMode::parse(""), CallMode::Auto);
        assert_eq!(CallMode::parse("auto"), CallMode::Auto);
        assert_eq!(CallMode::parse("s1-fast"), CallMode::Manual("s1-fast".into()));
        assert_eq!(CallMode::parse("group:fast-lane"), CallMode::Group("fast-lane".into()));
    }

    #[test]
    fn manual_direct_dispatch() {
        let reg = registry();
        let ratios = default_ratios();
        let bias = BiasConfig::default();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        let d = r.resolve(&RawRequest::default(), "s1-fast").expect("manual");
        assert_eq!(d.supplier_id, "s1");
        assert_eq!(d.model_id, "s1-fast");
        assert_eq!(d.endpoint_url, "https://s1.example/v1"); // trailing slash stripped
        assert_eq!(d.request_id, "s1-fast");
    }

    #[test]
    fn manual_unknown_model_errors() {
        let reg = registry();
        let ratios = default_ratios();
        let bias = BiasConfig::default();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        assert!(r.resolve(&RawRequest::default(), "ghost").is_err());
    }

    #[test]
    fn group_priority_wins() {
        let reg = registry();
        let mut bias = BiasConfig::default();
        bias.groups.insert(
            "fast-lane".to_string(),
            crate::config::RoutingGroup {
                description: "test".into(),
                endpoints: vec![
                    crate::config::GroupEndpoint { id: "s2-cheap".into(), priority: 1, weight: 100 },
                    crate::config::GroupEndpoint { id: "s1-fast".into(), priority: 10, weight: 1 },
                ],
            },
        );
        let ratios = default_ratios();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        let d = r.resolve(&RawRequest::default(), "group:fast-lane").expect("group");
        assert_eq!(d.model_id, "s1-fast"); // higher priority wins regardless of weight
    }

    #[test]
    fn group_weight_sampling_deterministic() {
        let reg = registry();
        let mut bias = BiasConfig::default();
        bias.groups.insert(
            "even".to_string(),
            crate::config::RoutingGroup {
                description: "test".into(),
                endpoints: vec![
                    crate::config::GroupEndpoint { id: "s2-cheap".into(), priority: 5, weight: 50 },
                    crate::config::GroupEndpoint { id: "s1-fast".into(), priority: 5, weight: 50 },
                ],
            },
        );
        let ratios = default_ratios();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        let a = r.resolve(&RawRequest::default(), "group:even").expect("group");
        // same instance -> same pick every time
        for _ in 0..50 {
            let d = r.resolve(&RawRequest::default(), "group:even").expect("group");
            assert_eq!(d.model_id, a.model_id);
        }
    }

    #[test]
    fn group_unknown_falls_back_to_auto() {
        let reg = registry();
        let ratios = default_ratios();
        let bias = BiasConfig::default();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        let d = r.resolve(&RawRequest::default(), "group:nope").expect("auto fallback");
        assert!(!d.supplier_id.is_empty());
    }

    #[test]
    fn auto_full_pipeline() {
        let reg = registry();
        let ratios = default_ratios();
        let bias = BiasConfig::default();
        let health = HealthTracker::new(Default::default());
        let r = router(&reg, &ratios, &bias, &health);
        let req = RawRequest {
            prompt: "hello".into(),
            agent_role: "code-generation".into(),
            max_output_tokens: 64,
            ..Default::default()
        };
        let d = r.resolve(&req, "auto").expect("auto");
        assert!(d.supplier_id == "s1" || d.supplier_id == "s2");
    }
}
