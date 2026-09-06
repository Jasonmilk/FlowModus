//! Layer 3: standardized costing — Estimated_Cost_USD collapse.
//!
//! Behavior aligned with Python v1.7 `layer3_cost.py`: input/output token
//! billing from the supplier's declared rates, kv-cache savings from the
//! model's parasitic hit-rate snapshot (savings factor 0.9), and a context
//! window capacity gate.

use crate::layer2_5_deviation::DeviationSnapshot;
use crate::pb::{CostEstimate, ModelDeclaration, NormalizedRequest, SupplierDeclaration};

/// Round a float to 6 decimal places (Python `round(x, 6)` semantics).
fn round6(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}

/// Whether a model can handle the request (context-window gate, pure).
fn model_can_handle_request(model: &ModelDeclaration, request: &NormalizedRequest) -> bool {
    let ctx = model.capabilities.as_ref().map(|c| c.context_window).unwrap_or(0);
    if ctx > 0 {
        let estimated_total = request.estimated_input_tokens + request.max_output_tokens;
        if estimated_total > ctx {
            return false;
        }
    }
    true
}

/// Estimate the cost of a request for one supplier/model (pure).
pub fn estimate_cost(
    request: &NormalizedRequest,
    supplier: &SupplierDeclaration,
    model: &ModelDeclaration,
    deviation_snapshot: &DeviationSnapshot,
) -> CostEstimate {
    let billing = model.billing.as_ref();

    let input_cost =
        request.estimated_input_tokens as f64 * billing.map(|b| b.token_input as f64).unwrap_or(0.0);
    let output_cost =
        request.max_output_tokens as f64 * billing.map(|b| b.token_output as f64).unwrap_or(0.0);
    let total_cost = input_cost + output_cost;

    let ste_total = request.ste_input + request.ste_output_estimated;

    let kv_cache_applicable = model.kv_cache.as_ref().map(|k| k.supported).unwrap_or(false);
    let mut kv_cache_savings = 0.0;
    if kv_cache_applicable {
        let hit_rate = deviation_snapshot.kv_cache_hit_rate(&model.model_id);
        if hit_rate > 0.0 {
            kv_cache_savings = total_cost * hit_rate * 0.9;
        }
    }

    CostEstimate {
        supplier_id: supplier.supplier_id.clone(),
        model_id: model.model_id.clone(),
        estimated_cost_usd: total_cost as f32,
        ste_total: ste_total as f32,
        kv_cache_applicable,
        kv_cache_savings_estimate: round6(kv_cache_savings) as f32,
    }
}

/// Estimate cost for all eligible suppliers' models (pure).
pub fn estimate_all(
    request: &NormalizedRequest,
    suppliers: &[&SupplierDeclaration],
    deviation_snapshot: &DeviationSnapshot,
) -> Vec<CostEstimate> {
    let mut estimates = Vec::new();
    for supplier in suppliers {
        for model in &supplier.models {
            if !model_can_handle_request(model, request) {
                continue;
            }
            estimates.push(estimate_cost(request, supplier, model, deviation_snapshot));
        }
    }
    estimates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::{
        BillingDeclaration, CapabilitiesDeclaration, KvCacheDeclaration, ModelDeclaration,
    };

    fn model(id: &str, token_input: f32, token_output: f32) -> ModelDeclaration {
        ModelDeclaration {
            model_id: id.into(),
            billing: Some(BillingDeclaration {
                token_input,
                token_output,
                ..Default::default()
            }),
            capabilities: Some(CapabilitiesDeclaration::default()),
            kv_cache: Some(KvCacheDeclaration::default()),
            ..Default::default()
        }
    }

    fn supplier(id: &str, models: Vec<ModelDeclaration>) -> SupplierDeclaration {
        SupplierDeclaration {
            supplier_id: id.into(),
            models,
            ..Default::default()
        }
    }

    fn req(input_tokens: i32, output_tokens: i32, ste_in: f32, ste_out: f32) -> NormalizedRequest {
        NormalizedRequest {
            estimated_input_tokens: input_tokens,
            max_output_tokens: output_tokens,
            ste_input: ste_in,
            ste_output_estimated: ste_out,
            ..Default::default()
        }
    }

    #[test]
    fn cost_basic() {
        let m = model("m1", 0.55, 2.19);
        let s = supplier("s1", vec![m]);
        let snap = DeviationSnapshot::default();
        let r = req(100, 50, 100.0, 50.0);
        let e = estimate_cost(&r, &s, &s.models[0], &snap);
        // 100*0.55 + 50*2.19 = 55 + 109.5 = 164.5
        assert!((e.estimated_cost_usd - 164.5).abs() < 1e-4);
        assert!((e.ste_total - 150.0).abs() < 1e-4);
        assert!(!e.kv_cache_applicable);
        assert_eq!(e.kv_cache_savings_estimate, 0.0);
    }

    #[test]
    fn kv_cache_savings() {
        let mut m = model("m1", 1.0, 1.0);
        m.kv_cache.as_mut().unwrap().supported = true;
        let s = supplier("s1", vec![m]);
        let mut rates = std::collections::HashMap::new();
        rates.insert("m1".to_string(), 0.5);
        let snap = DeviationSnapshot::with_hit_rates(rates);
        let r = req(100, 100, 100.0, 100.0);
        let e = estimate_cost(&r, &s, &s.models[0], &snap);
        // total = 200, hit 0.5 -> savings = 200*0.5*0.9 = 90
        assert!((e.estimated_cost_usd - 200.0).abs() < 1e-4);
        assert!((e.kv_cache_savings_estimate - 90.0).abs() < 1e-4);
        assert!(e.kv_cache_applicable);
    }

    #[test]
    fn context_window_gate() {
        let mut m = model("m1", 1.0, 1.0);
        m.capabilities.as_mut().unwrap().context_window = 150;
        let s = supplier("s1", vec![m]);
        let snap = DeviationSnapshot::default();
        let r = req(100, 100, 100.0, 100.0); // 200 > 150 -> excluded
        let out = estimate_all(&r, &[&s], &snap);
        assert!(out.is_empty());
        let r2 = req(50, 50, 50.0, 50.0); // 100 <= 150 -> included
        let out2 = estimate_all(&r2, &[&s], &snap);
        assert_eq!(out2.len(), 1);
    }
}
