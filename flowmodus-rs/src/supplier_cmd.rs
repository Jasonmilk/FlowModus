//! Supplier registry CLI — add / list / get / rm / test across free|paid
//! tiers (度量衡注册台). Parsing stays std::env only (最小依赖铁律).
//!
//! Usage:
//!   flowmodus supplier add --tier free --id groq --base-url https://... \
//!       --auth bearer_token --model llama-3.3-70b [--in 0 --out 0] [--name ...]
//!   flowmodus supplier list [--tier free|paid]
//!   flowmodus supplier get --tier free --id groq
//!   flowmodus supplier rm --tier free --id groq
//!   flowmodus supplier test --tier free --id groq   (on-demand probe, 0 idle probes)

use crate::pb::{BillingDeclaration, EndpointDeclaration, ModelDeclaration, SupplierDeclaration};
use crate::registry::{is_free_supplier, RegistryStore, Tier};
use std::collections::HashMap;

/// Parse `--key value` / `--key=value` args into a map.
fn parse_args(args: &[String]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(rest) = a.strip_prefix("--") {
            if let Some(eq) = rest.find('=') {
                out.insert(rest[..eq].to_string(), rest[eq + 1..].to_string());
            } else if i + 1 < args.len() {
                out.insert(rest.to_string(), args[i + 1].clone());
                i += 1;
            } else {
                out.insert(rest.to_string(), String::new());
            }
        }
        i += 1;
    }
    out
}

fn tier_from(args: &HashMap<String, String>) -> Result<Tier, String> {
    match args.get("tier").map(|s| s.as_str()) {
        Some("free") => Ok(Tier::Free),
        Some("paid") => Ok(Tier::Paid),
        _ => Err("--tier must be free|paid".into()),
    }
}

fn require(args: &HashMap<String, String>, key: &str) -> Result<String, String> {
    args.get(key)
        .filter(|v| !v.is_empty())
        .cloned()
        .ok_or_else(|| format!("--{key} required"))
}

/// `supplier add`: build a SupplierDeclaration from flags, enforce tier,
/// write one JSON file under registry/{free,paid}/{id}.json.
fn cmd_add(args: &HashMap<String, String>) -> Result<(), String> {
    let tier = tier_from(args)?;
    let id = require(args, "id")?;
    let base_url = require(args, "base-url")?;
    let auth = args.get("auth").cloned().unwrap_or_else(|| "bearer_token".into());
    let model_id = require(args, "model")?;
    let name = args.get("name").cloned().unwrap_or_else(|| id.clone());
    let in_rate: f32 = args.get("in").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let out_rate: f32 = args.get("out").and_then(|v| v.parse().ok()).unwrap_or(0.0);

    let decl = SupplierDeclaration {
        supplier_id: id.clone(),
        supplier_name: name,
        verified: false,
        updated_at_unix: chrono_now_unix(),
        models: vec![ModelDeclaration {
            model_id: model_id.clone(),
            display_name: model_id,
            lang: "en".into(),
            semantic_tags: vec![],
            agent_roles: vec![],
            billing: Some(BillingDeclaration {
                currency: "USD".into(),
                token_input: in_rate,
                token_output: out_rate,
                compute_ms: 0.0,
                audio_sec: 0.0,
                video_frame: 0.0,
                free_quota_daily: 0,
            }),
            kv_cache: None,
            capabilities: None,
            tokenizer_compression_ratio: 0.0,
            capability_tags: vec![],
        }],
        endpoints: vec![EndpointDeclaration {
            r#type: "cloud".into(),
            region: args.get("region").cloned().unwrap_or_else(|| "global".into()),
            base_url,
            tls_version: "1.3".into(),
            auth_method: auth,
            priority: args.get("priority").and_then(|v| v.parse().ok()).unwrap_or(10),
            provision_url: String::new(),
            documentation_url: String::new(),
        }],
        compliance: None,
        rate_limits: None,
    };

    let store = RegistryStore::at_root();
    store.add(tier, &decl)?;
    println!("registered {} -> registry/{}/{}.json", decl.supplier_id, tier.dir(), decl.supplier_id);
    println!("free={} (physical truth: all models zero billing)", is_free_supplier(&decl));
    Ok(())
}

fn chrono_now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn cmd_list(args: &HashMap<String, String>) -> Result<(), String> {
    let store = RegistryStore::at_root();
    match args.get("tier").map(|s| s.as_str()) {
        Some("free") => print_tier(&store, Tier::Free),
        Some("paid") => print_tier(&store, Tier::Paid),
        _ => {
            print_tier(&store, Tier::Free);
            print_tier(&store, Tier::Paid);
        }
    }
    Ok(())
}

fn print_tier(store: &RegistryStore, tier: Tier) {
    let decls = store.load_tier(tier);
    println!("== {} ({}) ==", tier.as_str(), decls.len());
    for d in decls {
        let models: Vec<&str> = d.models.iter().map(|m| m.model_id.as_str()).collect();
        println!("  {} [{}] models={}", d.supplier_id, d.supplier_name, models.join(","));
    }
}

fn cmd_get(args: &HashMap<String, String>) -> Result<(), String> {
    let tier = tier_from(args)?;
    let id = require(args, "id")?;
    let store = RegistryStore::at_root();
    match store.get(tier, &id) {
        Some(d) => {
            println!("{}", serde_json::to_string_pretty(&d).map_err(|e| e.to_string())?);
            Ok(())
        }
        None => Err(format!("not found: tier={} id={id}", tier.as_str())),
    }
}

fn cmd_rm(args: &HashMap<String, String>) -> Result<(), String> {
    let tier = tier_from(args)?;
    let id = require(args, "id")?;
    RegistryStore::at_root().remove(tier, &id)?;
    println!("removed tier={} id={id}", tier.as_str());
    Ok(())
}

/// `supplier test`: explicit on-demand connectivity probe (0 idle probes —
/// Ironclad rule 0). No credentials are held; auth is Tuck's job.
fn cmd_test(args: &HashMap<String, String>) -> Result<(), String> {
    let tier = tier_from(args)?;
    let id = require(args, "id")?;
    let store = RegistryStore::at_root();
    let decl = store
        .get(tier, &id)
        .ok_or_else(|| format!("not found: tier={} id={id}", tier.as_str()))?;
    for ep in &decl.endpoints {
        if ep.base_url.is_empty() {
            continue;
        }
        let out = std::process::Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code} %{time_total}s", "--max-time", "8", "-A", "flowmodus-probe", &ep.base_url])
            .output()
            .map_err(|e| format!("curl: {e}"))?;
        let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let ok = out.status.success();
        println!("{} [{}] {} -> {} (reachable={})", id, tier.as_str(), ep.base_url, line, ok);
    }
    Ok(())
}

/// Run the `supplier` subcommand. Returns exit code.
pub fn run(args: &[String]) -> i32 {
    let Some(sub) = args.first() else {
        println!("supplier <add|list|get|rm|test> [--tier free|paid] ...");
        return 0;
    };
    let parsed = parse_args(&args[1..]);
    let result = match sub.as_str() {
        "add" => cmd_add(&parsed),
        "list" => cmd_list(&parsed),
        "get" => cmd_get(&parsed),
        "rm" => cmd_rm(&parsed),
        "test" => cmd_test(&parsed),
        _ => {
            println!("supplier <add|list|get|rm|test>");
            return 0;
        }
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("supplier {sub}: {e}");
            2
        }
    }
}
