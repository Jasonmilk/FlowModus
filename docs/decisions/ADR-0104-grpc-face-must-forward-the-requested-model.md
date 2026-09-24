# ADR-0104: gRPC 面必须转达调用方声明的模型，不得抹成 Auto

- **状态**: Proposed
- **日期**: 2026-09-24
- **决策范围**: FlowModus 的 gRPC 推理面（`flowmodus-rs/src/grpc_cmd.rs` 的 `ReasonService::reason`）
  与它调用的 `Router::resolve(request, model_param)`
- **相关**: `ADR-0101`（free/paid 注册台）、anaphase `ADR-0045` §5（运行期事实必须有声明通道）、
  `Tuck:ADR-0004` D7（出口与审计）

## 1. 问题（实测，2026-09-24）

链路两侧**都**带着模型：

- anaphase 侧 `ReasonRequest { model: model.to_string(), .. }`（`src/adapters/flowmodus.rs:64`）；
- FlowModus 侧 `let mode = req.model;`（`grpc_cmd.rs`）。

但 gRPC 面紧接着把 `Router::resolve` 的**第二个参数写成空串**：

```rust
let decision = router.resolve(&raw, "").map_err(...)?;   // ← 调用方的意图在这里被丢掉
```

`resolve(request, model_param)` 的语义是 `CallMode::parse(model_param)` ⇒
`Manual(model_id)` / `Group(name)` / `Auto`。传 `""` 恒为 `Auto`。

**后果不是"回落到默认"，而是"两个前台给出两个答案"：**

| 前台 | 传的 model_param | 结果（实测） |
|---|---|---|
| CLI `flowmodus route --model mock-chat-1` | `"mock-chat-1"` | `supplier=mock-llm endpoint=http://127.0.0.1:59099/v1 cost_usd=0` |
| gRPC `ReasonService::reason` | `""` | Auto ⇒ agnes-ai ⇒ 403 ⇒ 每个回合 `impasse` |

于是：**即使注册台里有一个完全可用的免费供应商、CLI 也能路由到它**，
经由 gRPC 的每一个真实回合仍然打在 agnes-ai 上并失败。观察到的症状
（`model: null`、`assistant/reply` 为空、`impasse`）曾被归因为"缺一个可用的
供应商 key"—— **那是误判**：可用的供应商一直在，缺的是把意图转达出去。

## 2. 决策

**gRPC 面转达调用方声明的模型**：`router.resolve(&raw, &mode)`。

## 3. 为什么不退回 `Auto` 也能安全

`CallMode::parse` 对**裸词**给出 `Group(name)`，而**未知 group 已经回落 Auto**
（`router.rs`：`None => self.auto(request)`）。所以：

- `mode == ""` ⇒ `Auto`（与改动前逐字一致，无回归）；
- `mode` 是已注册模型 ⇒ `Manual` ⇒ 路由到该模型的供应商（本次要修的行为）；
- `mode` 是认知模式名（原注释担心的情形，如 `reasoning`）⇒ 未知 group ⇒ `Auto`。

⇒ **优先级仍归路由器所有**，本次只是让 gRPC 面不再抹掉调用方的输入。

## 4. 判据

- 端到端：声明 `ANAPHASE_REASONING_MODEL=mock-chat-1`（本地确定性上游，
  `mock-llm` 已注册且 CLI 路由实测可用）⇒ 回合应**完成**并产出
  `assistant/usage`，而不是 `impasse`。
- 反证：把该行改回 `""` ⇒ 回合重新打在 agnes-ai 上 ⇒ 判据必须变红。

## 4b. ⚠️ 本决策是**必要但不充分**的（同轮实测，必须留痕）

改完（`cargo build` 通过、flowmodus-reason 已换新二进制）后端到端**仍然**打在 agnes-ai 上：

```
WARN Reasoning failed: "调用上游失败: https://apihub.agnes-ai.com/v1/chat/completions: status code 403"
```

因为 anaphase 送进 `model` 字段的**不是模型**：

```rust
// anaphase-helix/src/run_cycle/reasoning.rs:237
.reason(&effective_prompt, &self.run_config.reasoning_mode, &trace_id)
```

⇒ **同一个 `ReasonRequest.model` 字段承载两种含义**：

| 一侧 | 它以为这个字段是 |
|---|---|
| anaphase `adapters/flowmodus.rs` 的签名 `reason(prompt, model, trace_id)` | **模型** |
| FlowModus `grpc_cmd.rs`：`mode_to_role(&mode)` / `cognitive_mode: mode` | **认知模式** |

这与本 ADR 要修的是同一类病（一个字符串只能有一种解释），只是换到了 `model` 字段上，
且**两侧都"没写错"**，所以谁都不报错、只有回合静默失败。
本决策把 **FlowModus 这一面**对齐到字段的字面含义（CLI 面本来就是这样），
是修它的**前提**；真正的收口需要先在 proto 上把「模型」与「认知模式」分成两个字段
（`ReasonRequest` 目前只有 `prompt`/`model`/`max_tokens`），或明确 `model` 只作模型用、
另加模式字段。**在那之前，端到端判据无法变绿，本目标不算达成。**

## 5. 风险

若某个前台真的把"认知模式"塞进 `model` 且该词**恰好等于某个已注册模型 id**，
则它会被当作 Manual 而不是提示。这是**可接受**的：那样的情况下字面就是模型名，
按模型名路由是唯一说得通的解释；而"提示"应由 `cognitive_mode`/`agent_role` 承担
（两者本就在同一个 `RawRequest` 里）。
