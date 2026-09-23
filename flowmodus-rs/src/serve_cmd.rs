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
use crate::registry::{RegistryStore, Tier};
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
    // Read the full request: headers first, then the body up to
    // Content-Length (a supplier declaration can exceed one 2 KiB chunk).
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let n = stream.read(&mut chunk)?;
    buf.extend_from_slice(&chunk[..n]);
    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n");
    let headers = String::from_utf8_lossy(&buf[..head_end.unwrap_or(buf.len())]);
    let mut content_len = 0usize;
    for line in headers.lines() {
        if let Some(v) = line.strip_prefix("Content-Length:") {
            content_len = v.trim().parse().unwrap_or(0);
        }
    }
    let body_off = head_end.map(|i| i + 4).unwrap_or(buf.len());
    while buf.len() - body_off < content_len {
        let m = stream.read(&mut chunk)?;
        if m == 0 { break; }
        buf.extend_from_slice(&chunk[..m]);
    }

    let req = String::from_utf8_lossy(&buf);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");
    let path = raw_path.split('?').next().unwrap_or(raw_path);
    let query = raw_path.split('?').nth(1).unwrap_or("");
    let body = req[body_off.min(req.len())..].to_string();

    let (status, body) = if method == "GET" && path == "/healthz" {
        (200, r#"{"ok":true}"#.to_string())
    } else if method == "GET" && path == "/api/status" {
        match build_status() {
            Ok(b) => (200, b),
            Err(e) => (500, err_json(e)),
        }
    } else if method == "GET" && path == "/api/suppliers" {
        match build_suppliers() {
            Ok(b) => (200, b),
            Err(e) => (500, err_json(e)),
        }
    } else if method == "POST" && path == "/api/suppliers" {
        match add_supplier(&body) {
            Ok(b) => (200, b),
            Err(e) => (400, err_json(e)),
        }
    } else if method == "DELETE" && path == "/api/suppliers" {
        match remove_supplier(query) {
            Ok(b) => (200, b),
            Err(e) => (400, err_json(e)),
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

fn err_json(e: String) -> String {
    serde_json::json!({ "error": e }).to_string()
}

/* ---------- Supplier management endpoints (面板配置入口) ---------- */

/// Body contract of `POST /api/suppliers` (面板 ↔ flowmodus 的唯一契约):
/// a minimal declaration + optional API key. The key is stored apart (secrets
/// store), never inside the declaration file.
#[derive(serde::Deserialize)]
struct SupplierIn {
    tier: String,
    supplier_id: String,
    supplier_name: Option<String>,
    base_url: String,
    models: Vec<String>,
    api_key: Option<String>,
}

fn add_supplier(payload: &str) -> Result<String, String> {
    let input: SupplierIn =
        serde_json::from_str(payload).map_err(|e| format!("bad body: {e}"))?;
    let tier = match input.tier.as_str() {
        "free" => Tier::Free,
        "paid" => Tier::Paid,
        other => return Err(format!("unknown tier {other:?} (free|paid)")),
    };
    if input.supplier_id.trim().is_empty() {
        return Err("supplier_id is required".into());
    }
    if input.base_url.trim().is_empty() {
        return Err("base_url is required".into());
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let name = input
        .supplier_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| input.supplier_id.clone());
    let decl = crate::pb::SupplierDeclaration {
        supplier_id: input.supplier_id.clone(),
        supplier_name: name,
        verified: false,
        updated_at_unix: now,
        models: input
            .models
            .iter()
            .filter(|m| !m.trim().is_empty())
            .map(|m| crate::pb::ModelDeclaration {
                model_id: m.clone(),
                display_name: m.clone(),
                lang: "en".into(),
                semantic_tags: vec![],
                agent_roles: vec![],
                billing: Some(crate::pb::BillingDeclaration::default()),
                kv_cache: None,
                capabilities: None,
                tokenizer_compression_ratio: 0.0,
                capability_tags: vec![],
            })
            .collect(),
        endpoints: vec![crate::pb::EndpointDeclaration {
            r#type: "openai".into(),
            region: String::new(),
            base_url: input.base_url.clone(),
            tls_version: String::new(),
            auth_method: "bearer_token".into(),
            priority: 1,
            provision_url: String::new(),
            documentation_url: String::new(),
        }],
        compliance: None,
        rate_limits: None,
    };
    let store = RegistryStore::at_root();
    store.add(tier, &decl)?;
    if let Some(k) = input.api_key {
        if !k.trim().is_empty() {
            store.set_api_key(tier, &decl.supplier_id, k.trim())?;
        }
    }
    Ok(serde_json::json!({
        "ok": true,
        "supplier_id": decl.supplier_id,
        "tier": tier.as_str(),
        "api_key_set": store.has_api_key(tier, &decl.supplier_id),
    })
    .to_string())
}

fn build_suppliers() -> Result<String, String> {
    let store = RegistryStore::at_root();
    let (free, paid) = store.load_all();
    let mut out = Vec::new();
    for (tier, decls) in [(Tier::Free, free), (Tier::Paid, paid)] {
        for d in decls {
            out.push(serde_json::json!({
                "supplier_id": d.supplier_id,
                "supplier_name": d.supplier_name,
                "tier": tier.as_str(),
                "verified": d.verified,
                "models": d.models.iter().map(|m| m.model_id.clone()).collect::<Vec<_>>(),
                "base_url": d.endpoints.first().map(|e| e.base_url.clone()).unwrap_or_default(),
                "updated_at_unix": d.updated_at_unix,
                /* 打码：列表只暴露"有没有 key"，绝不回显明文（按需加载/0 硬编码） */
                "api_key_set": store.has_api_key(tier, &d.supplier_id),
            }));
        }
    }
    Ok(serde_json::json!({ "suppliers": out }).to_string())
}

fn remove_supplier(query: &str) -> Result<String, String> {
    let mut tier = None;
    let mut id = None;
    for kv in query.split('&') {
        if let Some(v) = kv.strip_prefix("tier=") {
            tier = Some(v.to_string());
        } else if let Some(v) = kv.strip_prefix("id=") {
            id = Some(v.to_string());
        }
    }
    let tier = match tier.as_deref() {
        Some("free") => Tier::Free,
        Some("paid") => Tier::Paid,
        _ => return Err("query needs tier=free|paid".into()),
    };
    let id = id.ok_or_else(|| "query needs id=<supplier_id>".to_string())?;
    let store = RegistryStore::at_root();
    store.remove(tier, &id)?;   // remove() 会一并清掉 .secrets
    Ok(serde_json::json!({ "ok": true, "supplier_id": id, "tier": tier.as_str() }).to_string())
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
