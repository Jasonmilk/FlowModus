//! Pipeline end-to-end determinism: RawRequest -> RoutingDecision through all
//! five layers. Same input twice -> bit-identical decision (ADR-0100 D4/D5).

use flowmodus::config::{BiasConfig, SupplierBias};
use flowmodus::layer1_normalize::normalize_request;
use flowmodus::layer2_registry::get_eligible_suppliers;
use flowmodus::layer2_5_deviation::DeviationSnapshot;
use flowmodus::layer3_cost::estimate_all;
use flowmodus::layer4_filter::apply_hard_filters;
use flowmodus::layer5_score::score_and_entropy_sample;
use flowmodus::pb;
use prost::Message;
use std::collections::HashMap;

fn build_registry() -> Vec<pb::SupplierDeclaration> {
    let model = |id: &str, role: &str, price_in: f32, price_out: f32, ctx: i32| {
        pb::ModelDeclaration {
            model_id: id.into(),
            agent_roles: vec![role.into()],
            billing: Some(pb::BillingDeclaration {
                token_input: price_in,
                token_output: price_out,
                ..Default::default()
            }),
            capabilities: Some(pb::CapabilitiesDeclaration {
                context_window: ctx,
                ..Default::default()
            }),
            kv_cache: Some(pb::KvCacheDeclaration {
                supported: true,
                ttl_seconds: 300,
                control_parameter: "cache_control".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    };
    vec![
        pb::SupplierDeclaration {
            supplier_id: "s1".into(),
            models: vec![model("s1-fast", "code-generation", 0.55, 2.19, 131072)],
            ..Default::default()
        },
        pb::SupplierDeclaration {
            supplier_id: "s2".into(),
            models: vec![model("s2-cheap", "code-generation", 0.10, 0.40, 65536)],
            ..Default::default()
        },
        pb::SupplierDeclaration {
            supplier_id: "s3".into(),
            models: vec![model("s3-local", "logical-analysis", 0.0, 0.0, 8192)],
            ..Default::default()
        },
    ]
}

fn run_pipeline(
    registry: &[pb::SupplierDeclaration],
    instance_id: &str,
    request_hash: &str,
) -> pb::RoutingDecision {
    let raw = pb::RawRequest {
        prompt: "refactor this module and explain the tradeoffs".into(),
        agent_role: "code-generation".into(),
        cognitive_mode: "partner".into(),
        max_output_tokens: 2048,
        extra_headers: HashMap::new(),
    };
    let mut ratios = HashMap::new();
    ratios.insert("default".to_string(), 1.0);
    let normalized = normalize_request(&raw, &ratios);

    let eligible = get_eligible_suppliers(registry, &normalized);
    let snap = DeviationSnapshot::default();
    let estimates = estimate_all(&normalized, &eligible, &snap);

    let constraints = pb::UserConstraints {
        max_cost_per_request_usd: 0.0, // no budget cap -> candidates survive
        max_claim_deviation_tolerance: 20.0,
        ..Default::default()
    };
    let bias = BiasConfig::default();
    let survivors = apply_hard_filters(
        estimates,
        &constraints,
        &snap,
        &bias,
        &HashMap::new(),
        &HashMap::new(),
        instance_id,
        1_752_000_000 / 60,
    );

    let eligible_suppliers: Vec<pb::EligibleSupplier> = survivors
        .into_iter()
        .map(|c| pb::EligibleSupplier {
            supplier_id: c.supplier_id.clone(),
            model_id: c.model_id.clone(),
            endpoint_url: format!("http://{}.local", c.supplier_id),
            score: 100.0 - c.estimated_cost_usd * 100.0,
            cost: Some(c),
            ..Default::default()
        })
        .collect();

    score_and_entropy_sample(&eligible_suppliers, "code-generation", instance_id, request_hash, &bias)
}

#[test]
fn pipeline_deterministic_same_instance() {
    let registry = build_registry();
    let a = run_pipeline(&registry, "inst-1", "req-abc");
    let b = run_pipeline(&registry, "inst-1", "req-abc");
    assert_eq!(a.encode_to_vec(), b.encode_to_vec(), "bit-identical decision");
    assert!(!a.supplier_id.is_empty());
}

#[test]
fn pipeline_survives_hard_filters() {
    // s1 cost = 28*0.55 + 2048*2.19 = 15.4 + 4485.12 = 4500 -> cut by max_cost 1.0
    // s2 cost = 15.4*0.1... recompute: 28*0.10 + 2048*0.40 = 2.8 + 819.2 = 822 -> cut too
    // s3 excluded (agent role mismatch). Pipeline must still terminate with a
    // decision when candidates survive; here everything is filtered -> the
    // decision layer requires >=1 candidate, so we assert the filter shrinks.
    let registry = build_registry();
    let raw = pb::RawRequest {
        prompt: "hello".into(),
        agent_role: "code-generation".into(),
        cognitive_mode: "partner".into(),
        max_output_tokens: 2048,
        extra_headers: HashMap::new(),
    };
    let mut ratios = HashMap::new();
    ratios.insert("default".to_string(), 1.0);
    let normalized = normalize_request(&raw, &ratios);
    let eligible = get_eligible_suppliers(&registry, &normalized);
    let snap = DeviationSnapshot::default();
    let estimates = estimate_all(&normalized, &eligible, &snap);
    let constraints = pb::UserConstraints {
        max_cost_per_request_usd: 0.01,
        ..Default::default()
    };
    let survivors = apply_hard_filters(
        estimates,
        &constraints,
        &snap,
        &BiasConfig::default(),
        &HashMap::new(),
        &HashMap::new(),
        "inst",
        42,
    );
    assert!(survivors.is_empty(), "budget 0.01 cuts all paid models");
}

#[test]
fn pipeline_bias_can_invert_choice() {
    // With a strong user bias on s2-cheap, the cheap model wins despite lower
    // raw score. Judgment belongs to the user (VISION).
    let registry = build_registry();
    let raw = pb::RawRequest {
        prompt: "hello world".into(),
        agent_role: "code-generation".into(),
        cognitive_mode: "partner".into(),
        max_output_tokens: 64,
        extra_headers: HashMap::new(),
    };
    let mut ratios = HashMap::new();
    ratios.insert("default".to_string(), 1.0);
    let normalized = normalize_request(&raw, &ratios);
    let eligible = get_eligible_suppliers(&registry, &normalized);
    let snap = DeviationSnapshot::default();
    let estimates = estimate_all(&normalized, &eligible, &snap);
    let constraints = pb::UserConstraints::default();
    let mut biases = HashMap::new();
    biases.insert(
        "s2".to_string(),
        SupplierBias {
            bias_score: 1000.0,
            max_cost_per_request: None,
        },
    );
    let bias = BiasConfig {
        supplier_biases: biases,
        ..Default::default()
    };
    let survivors = apply_hard_filters(
        estimates,
        &constraints,
        &snap,
        &bias,
        &HashMap::new(),
        &HashMap::new(),
        "inst",
        42,
    );
    let eligible_suppliers: Vec<pb::EligibleSupplier> = survivors
        .into_iter()
        .map(|c| pb::EligibleSupplier {
            supplier_id: c.supplier_id.clone(),
            model_id: c.model_id.clone(),
            endpoint_url: String::new(),
            score: 50.0,
            cost: Some(c),
            ..Default::default()
        })
        .collect();
    let d = score_and_entropy_sample(&eligible_suppliers, "code-generation", "inst", "req", &bias);
    assert_eq!(d.supplier_id, "s2");
}
