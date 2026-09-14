# ADR-0102: 判定面——把 judge-points 规则放到已有的 serve 面上

- **状态**: Accepted（2026-09-15 用户拍板：采纳诊断、驳回处方，四案按此冻结）
- **日期**: 2026-09-14
- **决策范围**: FlowModus（`flowmodus-rs`，`serve_cmd.rs` / `judge_points.rs`）
- **关联**: DNA v1.1（0 硬编码、物理事实优先、铁律 0 零主动探测）／ PLAN R-5（judge-points 打通）／
  `docs/engineering-manual/judge-points-contract.md`（v1.1-draft）／ ADR-0100（rs 重构立项）／
  `anaphase:ADR-0039`（消费侧收敛）

## 1. 背景

契约把 FlowModus 定为 JP-1 / JP-2 的**提供方**，实现也已在 `src/judge_points.rs`
（`judge_suggested_mode` / `judge_budget_tier`，纯函数，0 token，阈值来自 `JudgePointsConfig`）。
PLAN 的 **R-5** 也正是「judge-points 打通」。

缺的是**面**：`serve_cmd.rs:71-73` 只路由 `GET /healthz` 与 `GET /api/status`，
`judge` 只在 CLI。消费方 Anaphase **物理上拿不到**这份判定，于是自己重算了一遍
（三套阈值：48/192、10/40、20/60，见 ADR-0039）。契约在纸面成立，在物理上不成立。

## 2. 决策

### D1｜在既有 serve 面新增 `POST /api/judge`

```
POST /api/judge
  → 请求体: {"input": "<用户输入原文>"}
  → 响应体: {"suggested_mode": "simple|moderate|complex",
             "budget_tier": "endogenous|augmentable|exogenous",
             "rule_version": "<JudgePointsConfig 的 hash>"}
```

`rule_version` 回传的是**配置 hash，不是阈值本身** —— 既让判定可复现，
又不把阈值变成第二份来源（见 D5）。

纯函数、无状态、0 token、不碰 registry、不碰任何 key、不代理流量。
拓扑不变：Anaphase → FlowModus（决策）→ Tuck（唯一门）。

### D2｜按需触发，不是心跳

判定只在**被请求时**计算。无定时轮询、无空闲心跳、无主动探测——符合 DNA 铁律 0
与 ADR-0101 §3「零主动探测」。这是**请求驱动**，与「探测」是两回事。

### D3｜零新依赖

`serve_cmd.rs` 是手写的 std HTTP/1.1（无 web 框架）。新增一个路由分支即可，
不引入任何依赖，不违反 PLAN 铁律 6「依赖极简」。

### D4｜请求体读取按 Content-Length，不用固定缓冲

现状 `handle()` 用 `[0u8; 2048]` 一次性读（`serve_cmd.rs:62`）。判定请求带**用户原文**，
长度不可预知：固定 2048 会**静默截断**输入 → 判定基于被截断的文本 → 造出一个错误的事实。

实现要求：
- 先解析请求头拿到 `Content-Length`，再按需读满（有上限，上限来自配置常量，不写字面量）；
- 超限 → 明确报错（`413`），**不得**截断后照常判定。

### D5｜阈值唯一来源，`rule_version` 是例外

`JudgePointsConfig` 是长度的唯一来源。响应体**不回传阈值**（那是内部事实，不是判定结果）；
唯一例外是 `rule_version` —— 它是该份配置的 **hash**，用于让消费方把判定与规则版本绑定归档。
必须保证「同一个 `input` + 同一个 `rule_version` 永远得到同一个 `suggested_mode`」——
可回放、可断言。

### D6｜超时、并发与错误码（判定面在关键路径上）

`serve` 是**手写 std HTTP/1.1，串行 accept**（`serve_cmd.rs:50-57`）。
判定请求位于消费方**每轮推理的关键路径**，一次慢请求会挂住整轮。故：

- **读超时**：`TcpStream` 设 `set_read_timeout`（值来自配置常量，零字面量）。
- **请求体上限**：来自配置常量；超限 → `413`，**不得**截断后照常判定。
- **绑定范围保持 `127.0.0.1`**（`serve_cmd.rs:42` 已是），显式写死为本机回环，不对外网暴露。
- **错误码 → 消费方行为表**（与 `anaphase:ADR-0039` §D7 共用同一张表）：

| 响应 | 含义 | 消费方行为 |
|---|---|---|
| `200` + 合法 JSON | 判定成功 | 采用 |
| `404` | 判定面未上线 | 无建议 + 熔断计数 |
| `413` | 输入超上限 | **无建议**（不得截断判定） |
| `5xx` | 内部错误（含 registry 空） | 无建议 + 熔断计数 |
| 超时 / 连接失败 | 不可达 | 无建议 + 熔断计数 |
| `200` 但 JSON 非法 / 字段缺失 | 解析失败 | 无建议 + 熔断计数 + 记诊断 |

> **判定面保持哑**：不在 Tuck 审计链上，不写自己的审计记录。审计由消费方（Anaphase）写。

## 3. 明确不做

- **不做 gRPC judge 服务**。proto（gossip / metrics / registry / routing / supplier）无 judge 服务；
  新增要两仓 codegen 同步。HTTP 面已在，改动最小。（ADR-0101 §3 已把 gRPC 服务端列为二期，
  本 ADR 不改变该结论。）
- **不做规则下发 / 规则同步**。Anaphase 不缓存规则副本——那会立刻造出第二个来源。
- **不做 `POST /api/judge` 的批量形态**。按需驱动：一次请求一次判定。

## 4. 放弃了什么

- **放弃了「判定面不在也能工作」这条退路。** 判定面成为 Anaphase 判定链上的必需面；
  它不在时消费方降级为「无建议」（ADR-0039 D5），而非回落到本地重算。
  这是刻意的：**宁可留空，不猜数**。
- **放弃了把阈值回传给消费方的便利。** 消费方拿不到阈值就无法自己算——这正是本 ADR 想要的效果。
  代价是排障时不能只看响应体就复现判定，需读 FlowModus 的 config。
- **放弃了「用同一个端点顺手把供应商池也判定一遍」的想法。** `/api/status` 保持只读呈现，
  判定是独立的一个面——一个端点一个关注点。

## 5. 后果

**正面**
- 契约的「提供方 = FlowModus」在物理上成立。
- 判定可被审计链 / 证轨记录（一次请求一条事实）。
- 跨语言可消费：非 Rust 消费方（面板）可读同一判定。

**负面**
- `serve` 从「只读呈现」扩展为「只读呈现 + 纯函数计算」。仍无状态、仍不碰 key，
  但面的语义变宽了一点。
- 多一处需要维护的路由分支与请求体读取逻辑。

## 6. 实施追踪

| 任务 | 状态 |
|---|---|
| T1 `handle()` 增加 `POST /api/judge` 分支 + `Content-Length` 读取（D4） | 待实施 |
| T2 判定实现复用 `judge_points.rs` 的两个纯函数，不新增规则 | 待实施 |
| T3 响应体加 `rule_version`（`JudgePointsConfig` hash） | 待实施 |
| T4 读超时 + 请求体上限 + 错误码表落地（D6）；绑定保持 `127.0.0.1` | 待实施 |
| T5 单元测试：同输入同 `rule_version` 同输出（确定性）；超长输入报 413 而非截断判定 | 待实施 |
| T6 与 `anaphase:ADR-0039` T2 联调（判定面客户端 ↔ 判定面） | 待实施 |
| T7 阈值落 judge-points 契约 v1.2 + 写入生效日期（供 T0 shadow run 对照） | 待实施 |

**顺序硬约束**：本 ADR 必须先于 `anaphase:ADR-0039` 的 T2 上线并验证，
否则消费侧一合并就是**全量降级**。故两份 ADR **必须成对冻结**。

提交信息关联：`(ADR-0102 §T1)` … `(ADR-0102 §T7)`

## 7. 参考

- `FlowModus/docs/engineering-manual/judge-points-contract.md`（v1.1-draft）
- `FlowModus/docs/PLAN.md` R-5（judge-points 打通）
- `FlowModus/docs/decisions/ADR-0101-free-paid-registry.md`（零主动探测）
> 跨仓引用一律**仓名限定**：`anaphase:ADR-0039` 与 `Cellrix:ADR-0039`（若将来出现）是两回事。

- `anaphase:ADR-0039`（判定权收敛的消费侧；与本 ADR 成对冻结）
