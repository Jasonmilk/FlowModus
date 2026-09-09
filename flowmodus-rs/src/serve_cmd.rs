//! 按需状态端点（`flowmodus serve --port N`）——零新依赖的 std HTTP/1.1。
//!
//! 哲学：按需加载（不 serve 不监听）、极致解耦（只读 registry + 一次
//! 路由决策，不碰任何 key）、0 硬编码（端口 CLI 可覆盖，默认 60053 为
//! 文档化约定端口）。检定台（Cellrix）通过本端点拉供应商池与当前路由，
//! 不经文件路径耦合。

use crate::config::{BiasConfig, HealthConfig};
use crate::health::HealthTracker;
use crate::layer2_5_deviation::DeviationSnapshot;
use crate::pb::{RawRequest, UserConstraints};
use crate::registry::RegistryStore;
use crate::router::Router;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;

const DEFAULT_PORT: u16 = 60053;

pub fn cmd_serve(args: &[String]) -> i32 {
    let mut port = DEFAULT_PORT;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Ok(p) = v.parse::<u16>() {
                        port = p;
                    }
                }
            }
            "--help" | "-h" => {
                println!("flowmodus serve [--port N]  (default {DEFAULT_PORT})");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("serve: bind 127.0.0.1:{port} failed: {e}");
            return 1;
        }
    };
    println!("flowmodus serve: http://127.0.0.1:{port}/api/status (Ctrl+C 停止)");
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let _ = handle(&mut s);
            }
            Err(_) => continue,
        }
    }
    0
}

fn handle(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let (status, body) = if method == "GET" && path == "/healthz" {
        (200, r#"{"ok":true}"#.to_string())
    } else if method == "GET" && path == "/api/status" {
        match build_status() {
            Ok(b) => (200, b),
            Err(e) => (500, format!(r#"{{"error":{}}}"#, serde_json::json!(e))),
        }
    } else {
        (404, r#"{"error":{"type":"not_found"}}"#.to_string())
    };

    let resp = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        if status == 200 { "OK" } else { "ERR" },
        body.len(),
        body
    );
    stream.write_all(resp.as_bytes())
}

fn build_status() -> Result<String, String> {
    let store = RegistryStore::at_root();
    let (free, paid) = store.load_all();
    let mut registry: Vec<crate::pb::SupplierDeclaration> = free.clone();
    registry.extend(paid.clone());
    if registry.is_empty() {
        return Err("registry is empty — add suppliers first (flowmodus supplier add ...)".into());
    }

    // 当前路由决策：最小请求（无偏好，纯物理事实 + 免费优先软权重）
    let request = RawRequest {
        prompt: "".into(),
        agent_role: String::new(),
        cognitive_mode: String::new(),
        max_output_tokens: 0,
        extra_headers: HashMap::new(),
    };
    let constraints = UserConstraints {
        max_cost_per_request_usd: 0.0,
        require_regions: vec![],
        require_modalities: vec![],
        require_verified_supplier: false,
        max_claim_deviation_tolerance: 0.0,
    };
    let health = HealthTracker::new(HealthConfig::default());
    let tokenizer_ratios: HashMap<String, f64> = HashMap::new();
    let bias = BiasConfig::default();
    let deviation = DeviationSnapshot::default();
    let router = Router::new(
        &registry,
        &tokenizer_ratios,
        deviation,
        &bias,
        constraints,
        &health,
        "serve",
        0,
    );
    let decision = match router.resolve(&request, "") {
        Ok(d) => serde_json::json!({
            "supplier_id": d.supplier_id,
            "model_id": d.model_id,
            "endpoint_url": d.endpoint_url,
            "estimated_cost_usd": d.estimated_cost_usd,
            "kv_cache_parameter": d.kv_cache_parameter,
            "kv_cache_ttl": d.kv_cache_ttl,
        }),
        Err(e) => serde_json::json!({ "error": e }),
    };

    Ok(serde_json::json!({
        "tiers": { "free": free, "paid": paid },
        "current": decision,
    })
    .to_string())
}
