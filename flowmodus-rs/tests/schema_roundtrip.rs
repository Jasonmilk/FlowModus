//! Schema contract round-trip tests: the 5 protobuf contracts must survive
//! encode -> decode with bit-identical semantics (DNA iron law 5).

use flowmodus::pb;
use prost::Message;

#[test]
fn telemetry_sample_roundtrip() {
    let s = pb::TelemetrySample {
        supplier_id: "sup-a".into(),
        model_id: "model-x".into(),
        sample_time_unix: 1_752_000_000,
        ttft_ms: 12.5,
        total_duration_ms: 340.0,
        kv_cache_hit: true,
        billed_tokens: 512,
        status_code: 200,
        health_state: "healthy".into(),
    };
    let bytes = s.encode_to_vec();
    let decoded = pb::TelemetrySample::decode(bytes.as_slice()).expect("decode");
    assert_eq!(s, decoded);
    // Determinism: same input, same bytes.
    assert_eq!(bytes, s.encode_to_vec());
}

#[test]
fn claim_deviation_roundtrip() {
    let d = pb::ClaimDeviation {
        supplier_id: "sup-b".into(),
        metric_name: "token_input".into(),
        claimed_value: 0.55,
        actual_value: 0.57,
        deviation_percent: 3.636,
        calculated_at_unix: 1_752_000_100,
    };
    let bytes = d.encode_to_vec();
    let decoded = pb::ClaimDeviation::decode(bytes.as_slice()).expect("decode");
    assert_eq!(d, decoded);
}

#[test]
fn raw_request_with_headers_roundtrip() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("x-agent-role".to_string(), "tool-orchestration".to_string());
    let r = pb::RawRequest {
        prompt: "summarize".into(),
        agent_role: "tool-orchestration".into(),
        cognitive_mode: "partner".into(),
        max_output_tokens: 1024,
        extra_headers: headers,
    };
    let bytes = r.encode_to_vec();
    let decoded = pb::RawRequest::decode(bytes.as_slice()).expect("decode");
    assert_eq!(r, decoded);
    assert_eq!(decoded.extra_headers.get("x-agent-role").map(|s| s.as_str()), Some("tool-orchestration"));
}

#[test]
fn gossip_message_roundtrip() {
    let g = pb::GossipMessage {
        instance_id: "inst-7".into(),
        timestamp_unix: 1_752_000_200,
        message_type: "deviation".into(),
        target_id: "sup-c".into(),
        payload: "{\"deviation\":1.2}".into(),
    };
    let bytes = g.encode_to_vec();
    let decoded = pb::GossipMessage::decode(bytes.as_slice()).expect("decode");
    assert_eq!(g, decoded);
}
