//! END-TO-END FAILOVER (ADR-0048 §369): a candidate set that CONTAINS a dead supplier must still answer.
//!
//! The unit criteria in `failover.rs` prove the RULE; this proves the RULE IS WIRED INTO THE PATH. It is
//! in-process (no port, no real key): `ReasonService::new(store)` + `FlowModus::reason(&svc, …)` directly,
//! with a two-declaration registry on disk and a tiny local mock standing in for a working supplier.
//!
//! The mutation is built in: the third phase keeps ONLY the dead supplier, and the request must then FAIL
//! with a NAMED class — so the test cannot pass by ignoring the failure.

use flowmodus::flowmodus_api::flow_modus_server::FlowModus;
use flowmodus::flowmodus_api::ReasonRequest;
use flowmodus::grpc_cmd::ReasonService;
use flowmodus::registry::{RegistryStore, Tier};
use std::io::{Read, Write};
use std::net::TcpListener;
use tonic::Request;

/// A working supplier, answering one canned OpenAI-shaped response. Returns its address.
fn spawn_mock() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut s = match stream { Ok(s) => s, Err(_) => continue };
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let body = r#"{"choices":[{"message":{"content":"pong"}}],"usage":{"prompt_tokens":1,"completion_tokens":1}}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    addr
}

fn write_declaration(root: &std::path::Path, id: &str, base_url: &str) {
    // THE TEMPLATE IS A REAL REGISTRY FILE (measured layout: `<root>/free/<id>.json`, one declaration per
    // file), so the shape cannot drift from what the store actually reads.
    let template = std::fs::read_to_string("registry/free/mock-llm.json")
        .expect("the production registry file is the template");
    let mut v: serde_json::Value = serde_json::from_str(&template).unwrap();
    v["supplier_id"] = serde_json::Value::String(id.into());
    v["supplier_name"] = serde_json::Value::String(id.into());
    v["endpoints"][0]["base_url"] = serde_json::Value::String(base_url.into());
    let dir = root.join("free");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{id}.json")), serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

fn temp_root(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "fm-failover-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

async fn ask(store: RegistryStore) -> Result<String, tonic::Status> {
    let svc = ReasonService::new(store);
    let resp = FlowModus::reason(&svc, Request::new(ReasonRequest {
        prompt: "hi".into(),
        model: String::new(),          // AUTO: failover is allowed (a manual selector deliberately is not)
        max_tokens: 16,
        cognitive_mode: "auto".into(),
    }))
    .await?;
    Ok(resp.into_inner().content)
}

#[tokio::test]
async fn a_dead_supplier_in_the_set_does_not_fail_the_request() {
    let mock = spawn_mock();
    let root = temp_root("mixed");
    write_declaration(&root, "dead-llama", "http://127.0.0.1:1/v1"); // nothing listens here
    write_declaration(&root, "live-mock", &format!("http://{mock}/v1"));
    let store = RegistryStore::new(&root);
    for id in ["dead-llama", "live-mock"] {
        store.set_api_key(Tier::Free, id, "test-key").expect("a key for every candidate");
    }

    let answer = ask(store).await.expect("the set contains a working supplier");
    assert!(!answer.trim().is_empty(), "the failover answer must carry text, got {answer:?}");
    let _ = std::fs::remove_dir_all(&root);
}

/// THE MUTATION: with only the dead supplier there is nothing to hand over to, and that must be a NAMED
/// failure — not an empty string and not a silent Ok. Removing the failover (or swallowing the failure)
/// makes this test fail.
#[tokio::test]
async fn an_exhausted_set_fails_by_name() {
    let root = temp_root("dead-only");
    write_declaration(&root, "dead-llama", "http://127.0.0.1:1/v1");
    let store = RegistryStore::new(&root);
    store.set_api_key(Tier::Free, "dead-llama", "test-key").unwrap();

    let err = ask(store).await.expect_err("nothing can answer");
    let msg = err.message().to_string();
    assert!(
        msg.contains("refused") || msg.contains("unreachable") || msg.contains("timeout"),
        "the failure must name a transport class, got {msg:?}"
    );
    assert!(msg.contains("dead-llama"), "and it must name the supplier it tried, got {msg:?}");
    let _ = std::fs::remove_dir_all(&root);
}
