# FlowModus PLAN——rs 重构导航牌

> **版本**：v1.0 ｜ 2026-09-06 ｜ ≤150 行
> 目标：Python v1.7 → Rust rs 分支，哲学继承、行为等价、铁律生效。

---

## 一、里程碑总览

| 里程碑 | 内容 | 验收 | 状态 |
|---|---|---|---|
| **R-0** | 立项：VISION/DNA/RNA/PLAN/GROWTH + ADR-0100 + 调研笔记 | 五件套齐 + ADR Accepted | ✅ 本轮 |
| **R-1** | cargo 工程骨架 + schema 迁移（protobuf 继承） | `cargo build` + schema 测试 | ⏳ |
| **R-2** | 五层流水线迁移（normalize/registry/cost/filter/score） | 行为等价（测试向量对齐 Python） | ⏳ |
| **R-3** | 三调用模式（Manual/Group/Auto）+ 失败冷却恢复 | 模式测试全绿 | ⏳ |
| **R-4** | 控制面（verifier/anti-corruption）+ 遥测（寄生，零探测） | grep 无探测字面量 | ⏳ |
| **R-5** | judge-points 契约打通（JP 升级链与调度语义对接） | JP-1/JP-2 可配 + 测试 | ⏳ |
| **R-6** | 文档链 + 全量验证 + 推送 | README 可用 + 推 rs | ⏳ |

## 二、R-1 骨架与 schema（依赖序）

1. `cargo new --lib flowmodus`（rs 分支，工作区独立）
2. protobuf 迁移：5 个 proto 编译进 crate（tonic-build/prost），契约不变
3. 模块骨架：`data_plane`（pipeline 5 层 + proxy）/ `control_plane`（verifier/canonicalizer）/
   `telemetry`（collector/deviation）/ `cli`（entrypoint）
4. 铁律 6：依赖极简（tokio + tonic + prost + ed25519-dalek + serde，无 web 框架）

## 三、R-2 五层流水线（核心）

| 层 | Python 现状 | Rust 落点 | 等价判据 |
|---|---|---|---|
| 1 Normalize | STE 换算 | 纯函数 crate | 同 token 输入同 STE 输出 |
| 2 Registry | Ed25519 签名声明 | ed25519-dalek 验证 | 签名验证向量对齐 |
| 3 Cost | 实时估计+偏差 | 纯函数 + telemetry 输入 | 同遥测同成本 |
| 4 Filter | 预算/地域/驻留 | 约束纯函数 | 同约束同过滤结果 |
| 5 Score | 熵权重 | instance-id 去相关 | 同 instance 同排序 |

- 层间 protobuf（schema 强制铁律）；
- **新增**：失败冷却 + 恢复（one-api 语义，确定性来源真实流量遥测——拒绝探测）。

## 四、R-3 三调用模式

- Manual：直连指定模型（无管线开销）
- Group：组内优先级/权重
- Auto：全五层
- 模式选择在请求体 `model` 参数（继承 v1.7 语义）

## 五、R-4 控制面 + 遥测

- verifier：注册表 Ed25519 验证；canonicalizer：供应商声明归一
- telemetry：collector（真实流量派生）+ deviation（成本偏差）——**零主动探测**
  （DNA 铁律 0；grep 核验无 probe/heartbeat/poll 字面量）

## 六、R-5 judge-points 打通

- JP 契约（FlowModus docs，v1.0-draft）落地：
  - JP-1/JP-2 语义判断点 → FlowModus Manual/Group（"small-llm" 组）调度
  - 升级链信号（判据不满足 → 升级）与 layer4/5 对接
- 与 Anaphase judge.rs 的端点语义对齐（OpenAI 兼容）

## 七、边界（禁止清单）

- ❌ 不建 UI/daemon/ORM/web 框架（DNA 铁律 6）
- ❌ 不持有凭证（Tuck 管）；不做权限分组（CAPABILITY-13）
- ❌ 不主动探测（DNA 铁律 0）
- ❌ 不做在线学习路由（M3 边界，VISION 开放项）
- ❌ 不破坏 5 个 protobuf 契约（扩展走新字段）

## 八、验证与交付

- 每里程碑：`cargo test` + 铁律 grep 核验 + GROWTH 记录
- 文档链：ADR → PLAN → GROWTH → README → ECOSYSTEM（FlowModus 行更新）→ HANDOFF
- 推送：rs 分支（全生态测试数统计口径：FlowModus 入列）

---

**导航牌使命：让每一步生长有据可依，哲学自洽，逻辑闭环。**
