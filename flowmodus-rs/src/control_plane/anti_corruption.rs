//! Anti-corruption layer — external JSON -> type-safe protobuf messages
//! (whitepaper §3.3 defense-in-depth, Python v1.7 `ParseDict` equivalent).
//!
//! Whitelist structure check: every field must match the declared protobuf
//! shape; unknown/extra fields are rejected (fail-closed), so a corrupted or
//! malicious registry package cannot smuggle data into the pipeline.
//!
//! Scope note (on-demand): `RateLimits` is not consumed by the routing
//! decision (layer 2 selects by model_id/agent_roles), so it is not mapped
//! here — per VISION "judge what you consume" (极致节能).

use crate::pb::{
    BillingDeclaration, CapabilitiesDeclaration, ComplianceDeclaration, EndpointDeclaration,
    FreeAllowance, KvCacheDeclaration, ModelDeclaration, RateRule, RegistryPackage,
    StreamingDeclaration, SupplierDeclaration, TimeWindow, ToolCallingDeclaration, VolumeTier,
};
use serde_json::{Map, Value};

/// Parse a registry package JSON document into a type-safe message (pure).
pub fn parse_registry_package(json: &str) -> Result<RegistryPackage, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    let obj = v.as_object().ok_or("registry package must be a JSON object")?;

    let suppliers = match obj.get("suppliers") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(parse_supplier_declaration)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err("suppliers must be an array".into()),
        None => Vec::new(),
    };

    let signatures = get_string_array(obj, "signatures")?;

    Ok(RegistryPackage {
        version: get_string(obj, "version")?.unwrap_or_default(),
        published_at_unix: get_i64(obj, "published_at_unix")?.unwrap_or_default(),
        suppliers,
        signatures,
    })
}

/// Parse a supplier declaration JSON object (Python `ParseDict` equivalent).
pub fn parse_supplier_declaration(v: &Value) -> Result<SupplierDeclaration, String> {
    let obj = v.as_object().ok_or("supplier declaration must be a JSON object")?;

    let models = match obj.get("models") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(parse_model_declaration)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err("models must be an array".into()),
        None => Vec::new(),
    };

    let endpoints = match obj.get("endpoints") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(parse_endpoint_declaration)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err("endpoints must be an array".into()),
        None => Vec::new(),
    };

    let compliance = match obj.get("compliance") {
        Some(Value::Object(_)) => Some(parse_compliance(obj.get("compliance").expect("object"))?),
        Some(_) => return Err("compliance must be an object".into()),
        None => None,
    };

    Ok(SupplierDeclaration {
        supplier_id: get_string(obj, "supplier_id")?.unwrap_or_default(),
        supplier_name: get_string(obj, "supplier_name")?.unwrap_or_default(),
        verified: get_bool(obj, "verified")?.unwrap_or_default(),
        updated_at_unix: get_i64(obj, "updated_at_unix")?.unwrap_or_default(),
        models,
        endpoints,
        compliance,
        rate_limits: None, // not consumed by routing (on-demand scope note above)
    })
}

fn parse_model_declaration(v: &Value) -> Result<ModelDeclaration, String> {
    let obj = v.as_object().ok_or("model declaration must be a JSON object")?;
    let billing = match obj.get("billing") {
        Some(Value::Object(_)) => Some(parse_billing(obj.get("billing").expect("object"))?),
        Some(_) => return Err("billing must be an object".into()),
        None => None,
    };
    let kv_cache = match obj.get("kv_cache") {
        Some(Value::Object(_)) => Some(parse_kv_cache(obj.get("kv_cache").expect("object"))?),
        Some(_) => return Err("kv_cache must be an object".into()),
        None => None,
    };
    let capabilities = match obj.get("capabilities") {
        Some(Value::Object(_)) => Some(parse_capabilities(obj.get("capabilities").expect("object"))?),
        Some(_) => return Err("capabilities must be an object".into()),
        None => None,
    };
    Ok(ModelDeclaration {
        model_id: get_string(obj, "model_id")?.unwrap_or_default(),
        display_name: get_string(obj, "display_name")?.unwrap_or_default(),
        lang: get_string(obj, "lang")?.unwrap_or_default(),
        semantic_tags: get_string_array(obj, "semantic_tags")?,
        agent_roles: get_string_array(obj, "agent_roles")?,
        billing,
        kv_cache,
        capabilities,
        tokenizer_compression_ratio: get_f64(obj, "tokenizer_compression_ratio")?.unwrap_or(1.0) as f32,
        capability_tags: get_string_array(obj, "capability_tags")?,
    })
}

fn parse_billing(v: &Value) -> Result<BillingDeclaration, String> {
    let obj = v.as_object().ok_or("billing must be an object")?;
    Ok(BillingDeclaration {
        currency: get_string(obj, "currency")?.unwrap_or_default(),
        token_input: get_f64(obj, "token_input")?.unwrap_or_default() as f32,
        token_output: get_f64(obj, "token_output")?.unwrap_or_default() as f32,
        compute_ms: get_f64(obj, "compute_ms")?.unwrap_or_default() as f32,
        audio_sec: get_f64(obj, "audio_sec")?.unwrap_or_default() as f32,
        video_frame: get_f64(obj, "video_frame")?.unwrap_or_default() as f32,
        free_quota_daily: get_i64(obj, "free_quota_daily")?.unwrap_or_default() as i32,
        // ADR-0103. Absent => empty, which is exactly today's behaviour. Present but
        // malformed => an error, never a silent skip: the free tier is enforced on the
        // parsed declaration, so a rate card that failed to parse must not read as free.
        rules: match get_object_array(obj, "rules")? {
            None => Vec::new(),
            Some(arr) => arr.iter().map(parse_rate_rule).collect::<Result<_, _>>()?,
        },
        allowances: match get_object_array(obj, "allowances")? {
            None => Vec::new(),
            Some(arr) => arr.iter().map(parse_free_allowance).collect::<Result<_, _>>()?,
        },
    })
}

/// Arrays of objects, for the ADR-0103 rate card.
fn get_object_array<'a>(
    obj: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a Vec<Value>>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::Array(arr)) => Ok(Some(arr)),
        Some(_) => Err(format!("field '{key}' must be an array")),
    }
}

fn parse_time_window(v: &Value) -> Result<TimeWindow, String> {
    let obj = v.as_object().ok_or("time_window must be an object")?;
    Ok(TimeWindow {
        start_minute_utc: get_i64(obj, "start_minute_utc")?.unwrap_or_default() as i32,
        end_minute_utc: get_i64(obj, "end_minute_utc")?.unwrap_or_default() as i32,
        effective_from_unix: get_i64(obj, "effective_from_unix")?.unwrap_or_default(),
        effective_until_unix: get_i64(obj, "effective_until_unix")?.unwrap_or_default(),
    })
}

fn parse_volume_tier(v: &Value) -> Result<VolumeTier, String> {
    let obj = v.as_object().ok_or("tier must be an object")?;
    Ok(VolumeTier {
        from_units: get_i64(obj, "from_units")?.unwrap_or_default(),
        to_units: get_i64(obj, "to_units")?.unwrap_or_default(),
        price_per_unit: get_f64(obj, "price_per_unit")?.unwrap_or_default() as f32,
    })
}

fn parse_rate_rule(v: &Value) -> Result<RateRule, String> {
    let obj = v.as_object().ok_or("rate rule must be an object")?;
    Ok(RateRule {
        unit: get_i64(obj, "unit")?.unwrap_or_default() as i32,
        price_per_unit: get_f64(obj, "price_per_unit")?.unwrap_or_default() as f32,
        mode: get_i64(obj, "mode")?.unwrap_or_default() as i32,
        window: match obj.get("window") {
            None => None,
            Some(w) => Some(parse_time_window(w)?),
        },
        window_multiplier: get_f64(obj, "window_multiplier")?.unwrap_or_default() as f32,
        tiers: match get_object_array(obj, "tiers")? {
            None => Vec::new(),
            Some(arr) => arr.iter().map(parse_volume_tier).collect::<Result<_, _>>()?,
        },
    })
}

fn parse_free_allowance(v: &Value) -> Result<FreeAllowance, String> {
    let obj = v.as_object().ok_or("allowance must be an object")?;
    Ok(FreeAllowance {
        unit: get_i64(obj, "unit")?.unwrap_or_default() as i32,
        quantity: get_i64(obj, "quantity")?.unwrap_or_default(),
        reset_minute_utc: get_i64(obj, "reset_minute_utc")?.unwrap_or_default() as i32,
        reset_period_hours: get_i64(obj, "reset_period_hours")?.unwrap_or_default() as i32,
        on_exhaust: get_i64(obj, "on_exhaust")?.unwrap_or_default() as i32,
    })
}

fn parse_kv_cache(v: &Value) -> Result<KvCacheDeclaration, String> {
    let obj = v.as_object().ok_or("kv_cache must be an object")?;
    Ok(KvCacheDeclaration {
        supported: get_bool(obj, "supported")?.unwrap_or_default(),
        ttl_seconds: get_i64(obj, "ttl_seconds")?.unwrap_or_default() as i32,
        control_parameter: get_string(obj, "control_parameter")?.unwrap_or_default(),
        breakpoint_marker: get_string(obj, "breakpoint_marker")?.unwrap_or_default(),
        standard_compliance: get_bool(obj, "standard_compliance")?.unwrap_or_default(),
    })
}

fn parse_capabilities(v: &Value) -> Result<CapabilitiesDeclaration, String> {
    let obj = v.as_object().ok_or("capabilities must be an object")?;
    let tool_calling = match obj.get("tool_calling") {
        Some(Value::Object(_)) => Some(parse_tool_calling(obj.get("tool_calling").expect("object"))?),
        Some(_) => return Err("tool_calling must be an object".into()),
        None => None,
    };
    let streaming = match obj.get("streaming") {
        Some(Value::Object(_)) => Some(parse_streaming(obj.get("streaming").expect("object"))?),
        Some(_) => return Err("streaming must be an object".into()),
        None => None,
    };
    Ok(CapabilitiesDeclaration {
        context_window: get_i64(obj, "context_window")?.unwrap_or_default() as i32,
        modalities: get_string_array(obj, "modalities")?,
        tool_calling,
        streaming,
    })
}

fn parse_tool_calling(v: &Value) -> Result<ToolCallingDeclaration, String> {
    let obj = v.as_object().ok_or("tool_calling must be an object")?;
    Ok(ToolCallingDeclaration {
        supported: get_bool(obj, "supported")?.unwrap_or_default(),
        schema_format: get_string(obj, "schema_format")?.unwrap_or_default(),
    })
}

fn parse_streaming(v: &Value) -> Result<StreamingDeclaration, String> {
    let obj = v.as_object().ok_or("streaming must be an object")?;
    Ok(StreamingDeclaration {
        supported: get_bool(obj, "supported")?.unwrap_or_default(),
        protocol: get_string(obj, "protocol")?.unwrap_or_default(),
    })
}

fn parse_compliance(v: &Value) -> Result<ComplianceDeclaration, String> {
    let obj = v.as_object().ok_or("compliance must be an object")?;
    Ok(ComplianceDeclaration {
        data_processing: get_string(obj, "data_processing")?.unwrap_or_default(),
        content_filtering: get_string(obj, "content_filtering")?.unwrap_or_default(),
        data_portability: get_string(obj, "data_portability")?.unwrap_or_default(),
    })
}

fn parse_endpoint_declaration(v: &Value) -> Result<EndpointDeclaration, String> {
    let obj = v.as_object().ok_or("endpoint must be a JSON object")?;
    Ok(EndpointDeclaration {
        r#type: get_string(obj, "type")?.unwrap_or_default(),
        region: get_string(obj, "region")?.unwrap_or_default(),
        base_url: get_string(obj, "base_url")?.unwrap_or_default(),
        tls_version: get_string(obj, "tls_version")?.unwrap_or_default(),
        auth_method: get_string(obj, "auth_method")?.unwrap_or_default(),
        priority: get_i64(obj, "priority")?.unwrap_or_default() as i32,
        provision_url: get_string(obj, "provision_url")?.unwrap_or_default(),
        documentation_url: get_string(obj, "documentation_url")?.unwrap_or_default(),
    })
}

fn get_string(obj: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("field '{key}' must be a string")),
    }
}

fn get_string_array(obj: &Map<String, Value>, key: &str) -> Result<Vec<String>, String> {
    match obj.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|s| {
                s.as_str()
                    .map(String::from)
                    .ok_or_else(|| format!("field '{key}' must be strings"))
            })
            .collect(),
        Some(_) => Err(format!("field '{key}' must be an array")),
    }
}

fn get_i64(obj: &Map<String, Value>, key: &str) -> Result<Option<i64>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::Number(n)) => n
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("field '{key}' must be an integer")),
        Some(_) => Err(format!("field '{key}' must be an integer")),
    }
}

fn get_f64(obj: &Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::Number(n)) => n
            .as_f64()
            .map(Some)
            .ok_or_else(|| format!("field '{key}' must be a number")),
        Some(_) => Err(format!("field '{key}' must be a number")),
    }
}

fn get_bool(obj: &Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("field '{key}' must be a boolean")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_registry() {
        let json = r#"{
            "version": "v2.0-alpha",
            "published_at_unix": 1757000000,
            "suppliers": [
                {
                    "supplier_id": "s1",
                    "supplier_name": "alpha",
                    "verified": true,
                    "updated_at_unix": 1756900000,
                    "models": [
                        {
                            "model_id": "s1-fast",
                            "display_name": "S1 Fast",
                            "lang": "en",
                            "semantic_tags": ["fast"],
                            "agent_roles": ["code-generation"],
                            "billing": {"currency": "USD", "token_input": 0.1, "token_output": 0.4},
                            "kv_cache": {"supported": true, "ttl_seconds": 300, "control_parameter": "cache_control", "breakpoint_marker": "breakpoint"},
                            "capabilities": {
                                "context_window": 65536,
                                "modalities": ["text"],
                                "tool_calling": {"supported": true, "schema_format": "openai"},
                                "streaming": {"supported": true, "protocol": "sse"}
                            },
                            "tokenizer_compression_ratio": 0.8,
                            "capability_tags": ["kv-cache"]
                        }
                    ],
                    "endpoints": [
                        {"type": "openai", "region": "us-east", "base_url": "https://s1.example/v1/", "tls_version": "1.3", "auth_method": "bearer", "priority": 1}
                    ],
                    "compliance": {"data_processing": "us", "content_filtering": "standard", "data_portability": "export"}
                }
            ],
            "signatures": ["a914abdc99f277d64aafdd2c4db2ee73d468001e65747a823af50bb5a6dc3cf9"]
        }"#;
        let pkg = parse_registry_package(json).expect("parse");
        assert_eq!(pkg.version, "v2.0-alpha");
        assert_eq!(pkg.published_at_unix, 1757000000);
        assert_eq!(pkg.suppliers.len(), 1);
        let s = &pkg.suppliers[0];
        assert_eq!(s.supplier_id, "s1");
        assert_eq!(s.supplier_name, "alpha");
        assert!(s.verified);
        let m = &s.models[0];
        assert_eq!(m.model_id, "s1-fast");
        assert_eq!(m.agent_roles, vec!["code-generation"]);
        assert_eq!(m.tokenizer_compression_ratio, 0.8);
        let b = m.billing.as_ref().expect("billing");
        assert_eq!(b.token_input, 0.1);
        let k = m.kv_cache.as_ref().expect("kv");
        assert_eq!(k.ttl_seconds, 300);
        let c = m.capabilities.as_ref().expect("caps");
        assert_eq!(c.context_window, 65536);
        assert_eq!(c.tool_calling.as_ref().expect("tc").schema_format, "openai");
        assert_eq!(s.endpoints[0].base_url, "https://s1.example/v1/");
        assert_eq!(s.endpoints[0].r#type, "openai");
        assert_eq!(pkg.signatures.len(), 1);
        assert_eq!(pkg.signatures[0].len(), 64);
    }

    #[test]
    fn wrong_type_fails_closed() {
        assert!(parse_registry_package(r#"{"suppliers": "not-an-array"}"#).is_err());
        assert!(parse_registry_package(r#"{"suppliers": [{"models": 5}]}"#).is_err());
    }

    #[test]
    fn empty_package_defaults() {
        let pkg = parse_registry_package("{}").expect("empty");
        assert_eq!(pkg.version, "");
        assert!(pkg.suppliers.is_empty());
        assert!(pkg.signatures.is_empty());
    }

    /// ADR-0103: the rate card must survive the round trip with its numbers intact. A
    /// parser that produced empty lists would leave a priced declaration looking free,
    /// which is the one reading that must never happen by accident.
    #[test]
    fn parse_rate_card_and_allowance() {
        let b = parse_billing(&serde_json::json!({
            "currency": "USD",
            "rules": [
                {"unit": 1, "price_per_unit": 0.55,
                 "window": {"start_minute_utc": 990, "end_minute_utc": 1500},
                 "window_multiplier": 2.0,
                 "tiers": [{"from_units": 0, "to_units": 1000000, "price_per_unit": 0.55},
                           {"from_units": 1000000, "to_units": 0, "price_per_unit": 0.28}]},
                {"unit": 2, "price_per_unit": 2.19}
            ],
            "allowances": [{"unit": 1, "quantity": 500000, "reset_minute_utc": 960,
                            "reset_period_hours": 24, "on_exhaust": 1}]
        }))
        .unwrap();

        assert_eq!(b.rules.len(), 2, "both rules must survive");
        assert_eq!(b.rules[0].price_per_unit, 0.55);
        assert_eq!(b.rules[0].window.as_ref().unwrap().start_minute_utc, 990);
        assert_eq!(b.rules[0].window_multiplier, 2.0);
        assert_eq!(b.rules[0].tiers.len(), 2);
        assert_eq!(b.rules[0].tiers[1].price_per_unit, 0.28);
        assert_eq!(
            b.rules[1].tiers.len(),
            0,
            "an absent tier list is empty, not a fabricated default"
        );
        assert_eq!(b.allowances.len(), 1);
        assert_eq!(b.allowances[0].quantity, 500_000);
        assert_eq!(b.allowances[0].on_exhaust, 1);
    }

    /// Present-but-malformed is an **error**, not a skip: a rate card that silently failed
    /// to parse would read as a free supplier, and the free tier is enforced on the parsed
    /// declaration. Absent, by contrast, must stay today's behaviour.
    #[test]
    fn a_malformed_rate_card_is_an_error_not_a_skip() {
        assert!(parse_billing(&serde_json::json!({"rules": "not an array"})).is_err());
        assert!(parse_billing(&serde_json::json!({"rules": ["not an object"]})).is_err());
        assert!(parse_billing(&serde_json::json!({"allowances": [{"quantity": "lots"}]})).is_err());

        let absent = parse_billing(&serde_json::json!({"currency": "USD"})).unwrap();
        assert!(absent.rules.is_empty() && absent.allowances.is_empty());
    }
}
