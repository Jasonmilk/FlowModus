//! `flowmodus grpc --port N` — gRPC Reason 服务（anaphase 对话的唯一推理入口）。
//!
//! 契约：`proto/flowmodus.proto`（与 anaphase 同款，FlowModus 服务只此一个方法）。
//!
//! 职责边界（对齐工程哲学）：
//! - FlowModus 是供应商与路由的**唯一事实来源**：anaphase 不持任何 key。
//! - 每次 Reason：本服务用五层管线（router.auto）选供应商/模型，再从本机
//!   `registry/.secrets` 取 key 调用上游 OpenAI 兼容 `chat/completions`。
//! - 认知模式（left_brain/right_brain/cerebellum）是**路由提示**不是模型名：
//!   一律走 Auto 管线，模式映射为 agent_role 参与角色加分；不猜模型名。

use crate::config::BiasConfig;
use crate::flowmodus_api::flow_modus_server::{FlowModus, FlowModusServer};
use crate::flowmodus_api::{ReasonRequest, ReasonResponse, Usage};
use crate::health::HealthTracker;
use crate::layer2_5_deviation::DeviationSnapshot;
use crate::pb::{RawRequest, SupplierDeclaration, UserConstraints};
use crate::registry::{RegistryStore, Tier};
use crate::router::Router;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

/// 默认 gRPC 端口（与 HTTP serve 端口区分）。
const DEFAULT_GRPC_PORT: u16 = 60054;

pub fn cmd_grpc(args: &[String]) -> i32 {
    let mut port = DEFAULT_GRPC_PORT;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    port = v;
                    i += 2;
                    continue;
                }
            }
            "--help" | "-h" => {
                println!("flowmodus grpc [--port N]  (default {DEFAULT_GRPC_PORT})");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    let store = RegistryStore::at_root();
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("grpc: tokio runtime failed: {e}");
            return 1;
        }
    };
    let addr = format!("127.0.0.1:{port}");
    let svc = FlowModusServer::new(ReasonService::new(store));
    let result = rt.block_on(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr.parse().expect("bind address"))
            .await
    });
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("grpc: serve failed: {e}");
            1
        }
    }
}

/// Reason 服务：registry 注入（便于测试），每次调用独立路由，无共享可变状态。
pub struct ReasonService {
    store: RegistryStore,
}

impl ReasonService {
    pub fn new(store: RegistryStore) -> Self {
        Self { store }
    }

    /// 认知模式 → agent_role 路由提示（0 硬编码：映射表集中一处，注释即契约）。
    /// left_brain=推理/编码偏好、right_brain=创意偏好、cerebellum=快速低成本；
    /// auto/空 = 纯物理事实路由（无角色加分）。
    fn mode_to_role(mode: &str) -> String {
        match mode.trim() {
            "" | "auto" => String::new(),
            "left_brain" => "reasoning".to_string(),
            "right_brain" => "creative".to_string(),
            "cerebellum" => String::new(),
            other => other.to_string(),
        }
    }

    /// 调用上游 OpenAI 兼容 chat/completions。tokens 取 usage.total_tokens
    /// （0 也不伪装：上游不给就记 0）。
    /// THE CALL NAMES ITS OWN FAILURES (ADR-0048 §362/§367). It used to return one prose string
    /// ("调用上游失败: …"), which cannot be acted on: no failover can tell a refused connection from a
    /// 429, and an HTTP 200 with an unusable body read as SUCCESS. Every ending is now one of the named
    /// classes, so the caller can hand over instead of returning the first failure.
    fn call_chat(
        endpoint: &str,
        model: &str,
        prompt: &str,
        max_tokens: u32,
        key: &str,
    ) -> Result<(String, Option<Usage>), crate::failover::AttemptFailure> {
        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": prompt }],
            "max_tokens": max_tokens,
        });
        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(60))
            .set("Authorization", &format!("Bearer {key}"))
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| crate::failover::classify_ureq(&e))?;
        let body = resp
            .into_string()
            /* A read that dies mid-body is a TRANSPORT fact (the peer stopped answering), not a status. */
            .map_err(|_| crate::failover::AttemptFailure::Unreachable)?;
        let v: serde_json::Value = serde_json::from_str(&body)
            .map_err(|_| crate::failover::AttemptFailure::BadBody)?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        /* Disjoint facts, or NOTHING. This used to read `usage.total_tokens` and
         * hand back one number, which is lossy in exactly the way that made the
         * metering leg unwireable: Anaphase stores a breakdown and refuses to
         * rebuild one from a sum. Both counts must be reported, otherwise the
         * honest answer is "no usage", not a zero-filled breakdown. */
        let u = &v["usage"];
        let usage = match (u["prompt_tokens"].as_u64(), u["completion_tokens"].as_u64()) {
            (Some(prompt_tokens), Some(completion_tokens)) => Some(Usage {
                prompt_tokens,
                completion_tokens,
                cached_tokens: u["prompt_tokens_details"]["cached_tokens"]
                    .as_u64()
                    .or_else(|| u["prompt_cache_hit_tokens"].as_u64()),
                reasoning_tokens: u["completion_tokens_details"]["reasoning_tokens"].as_u64(),
            }),
            _ => None,
        };
        Ok((content, usage))
    }
}

#[tonic::async_trait]
impl FlowModus for ReasonService {
    async fn reason(
        &self,
        request: Request<ReasonRequest>,
    ) -> Result<Response<ReasonResponse>, Status> {
        let req = request.into_inner();
        let prompt = req.prompt.trim().to_string();
        if prompt.is_empty() {
            return Err(Status::invalid_argument("prompt 为空"));
        }
        /* Two DIFFERENT facts, and they used to share one field. The wire field
         * `model` is documented as the cognitive mode, and anaphase's adapter
         * called its parameter `model` while passing `reasoning_mode` — so the
         * model a caller declared never reached the router at all, and routing
         * silently fell back to whatever Auto preferred (measured 2026-09-24:
         * every turn called agnes-ai and died on 403 while a working free
         * supplier sat registered). The proto now carries them separately. */
        let mode = req.cognitive_mode;
        let max_tokens = if req.max_tokens == 0 { 2048 } else { req.max_tokens };

        // 1) registry → router（模板与 serve_cmd 同一套：纯物理事实）
        let (free, paid) = self.store.load_all();
        let mut registry: Vec<SupplierDeclaration> = free.clone();
        registry.extend(paid.clone());
        if registry.is_empty() {
            return Err(Status::failed_precondition(
                "registry 为空 —— 请先在面板 FlowModus 添加供应商并声明模型",
            ));
        }
        let health = HealthTracker::new(Default::default());
        let ratios: HashMap<String, f64> = HashMap::new();
        let bias = BiasConfig::default();
        let constraints = UserConstraints::default();
        let deviation = DeviationSnapshot::default();
        let router = Router::new(
            &registry,
            &ratios,
            deviation,
            &bias,
            constraints,
            &health,
            "grpc",
            0,
        );

        // 2) 路由：调用方**声明的模型**优先（`req.model`）；为空时才是 Auto。
    //    认知模式（`req.cognitive_mode`）是提示，不参与选供应商。
        let raw = RawRequest {
            prompt: prompt.clone(),
            agent_role: Self::mode_to_role(&mode),
            cognitive_mode: mode.clone(),
            max_output_tokens: max_tokens as i32,
            extra_headers: HashMap::new(),
        };
        /* The requested model is the caller's intent, and it was DROPPED here by
         * being replaced with "" — measured 2026-09-24: anaphase declared
         * `reasoning_model=mock-chat-1` (a registered free supplier answering from
         * a local mock), the CLI resolved that model correctly
         * (`supplier=mock-llm endpoint=http://127.0.0.1:59099/v1`), yet every turn
         * called agnes-ai and died on 403, because the gRPC face always ran Auto.
         * Two front doors, two answers — the same defect class this repo names
         * 唯一事实来源.
         *
         * Passing it through cannot regress the "cognitive mode" case the comment
         * above worries about: a bare word parses as a Group, and an UNKNOWN group
         * already falls back to Auto (`router.rs` CallMode::Group). Empty stays
         * Auto. So the router keeps its own precedence; this face stops erasing
         * the caller's. */
        let decision = router.resolve(&raw, &req.model).map_err(|e| {
            Status::unavailable(format!("路由失败: {e}"))
        })?;

        // 3) 取 key（本机 .secrets，唯一来源），调上游
        let tier = if free.iter().any(|s| s.supplier_id == decision.supplier_id) {
            Tier::Free
        } else {
            Tier::Paid
        };
        let key = self.store.api_key(tier, &decision.supplier_id).ok_or_else(|| {
            Status::failed_precondition(format!(
                "供应商 {} 未配置 API key —— 请在面板 FlowModus 的 {} 详情里补 key",
                decision.supplier_id, decision.supplier_id
            ))
        })?;
        let (content, usage) = Self::call_chat(
            &decision.endpoint_url,
            &decision.model_id,
            &prompt,
            max_tokens,
            &key,
        )
        /* THE BOUNDARY KEEPS THE NAMES (ADR-0048 §367): gRPC carries text, so the failure is rendered with
         * its CLASS NAME first and the upstream's own words after it — never collapsed into one prose line. */
        .map_err(|why| Status::aborted(format!("上游调用失败 [{}]", why.describe())))?;

        Ok(Response::new(ReasonResponse {
            content,
            usage,
            /* The ROUTED model — `decision.model_id`, not `req.model`. ADR-0036
             * asks for the fact that actually served the call; echoing the
             * caller's request back would be a configured value standing in for a
             * measured one (anaphase K-115). */
            model: decision.model_id.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_to_role_mapping_is_explicit() {
        assert_eq!(ReasonService::mode_to_role(""), "");
        assert_eq!(ReasonService::mode_to_role("auto"), "");
        assert_eq!(ReasonService::mode_to_role("left_brain"), "reasoning");
        assert_eq!(ReasonService::mode_to_role("right_brain"), "creative");
        assert_eq!(ReasonService::mode_to_role("cerebellum"), "");
    }

    #[test]
    fn empty_registry_errors_with_guidance() {
        let tmp = std::env::temp_dir().join(format!("fm-grpc-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let store = RegistryStore::new(&tmp);
        let svc = ReasonService::new(store);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(async {
            svc.reason(Request::new(ReasonRequest {
                prompt: "hi".into(),
                model: String::new(),
                max_tokens: 100,
                // THE TEST BUILD WAS BROKEN HERE (measured: `missing field cognitive_mode`) — unrelated to
                // the failover work, and left red it would have hidden every later test in this crate.
                // Empty string is the DECLARED-ABSENT value for this field (ADR-0048 §346).
                cognitive_mode: String::new(),
            }))
            .await
        });
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(err.message().contains("registry 为空"), "{}", err.message());
    }
}
