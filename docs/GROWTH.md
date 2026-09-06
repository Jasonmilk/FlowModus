# FlowModus GROWTH——生长记录

> 记录健康快照；超过 3 条时归档最旧的到 docs/growth-archive/。

## 记录 0：rs 重构立项（2026-09-06）

**健康快照**：✅ 立项完成（VISION/DNA/RNA/PLAN 立宪 + 调研笔记 + ADR-0100 待写）
**物理事实**：
- FlowModus 本地为 Python 项目（~2375 行 / 53 测试 / 5 层流水线已成形）
- **phyt-DNA 五件套缺失**（无 VISION/DNA/PLAN）——本次补齐
- 无 rs 分支（git 仅 main）；protobuf schema ×5 为可复用资产
**调研**（重构前参考，用户要求）：
- EchoBird（Tauri+Rust）：Model Nexus 展示/管理范式 → 归 Cellrix 驾驶舱；Auto Router
  主动探测 → 拒绝（DNA 铁律 0）
- one-api：权重均衡/失败切换/禁用恢复 → 吸收冷却+恢复语义
- FrugalGPT/RouteLLM/AutoMix：级联升级链 = judge-points 0→3B→L 的实证支撑（98% 成本
  削减）；学习路由 → 未来开放项（M3 边界）
**验证**：调研笔记落盘 docs/research/rs-refactor-research.md；VISION 立场表核对无冲突
**状态**：✅ 完成（下一步 R-1 cargo 骨架 + schema 迁移）

## 记录 1：哲学审查——度量衡范式确立（2026-09-06）

**健康快照**：✅ VISION/DNA 重写（第一版方向错误已纠）
**物理事实**：
- 用户纠正：FlowModus 最大范式是"度量衡"（STE=度 / 声明偏移量=校准 / 一致性测试=检定），
  且完全独立于 Helix（whitepaper §6.4："不依赖任何 Helix 生态的具体项目"）
- 我第一版 VISION 写成"Helix 生态调度协议层"——格局错误，已重写
- **发现文档滞后**：whitepaper v1.7 §5.5 仍有"每小时 Canary Probe"，manual v1.1.3
  铁律 0 已收紧（废除定时探测，唯一例外按需）——rs 以铁律 0 为准（ADR-0100 记录）
**验证**：whitepaper 579 行全文精读；VISION 立场表对照 whitepaper 逐条核对
**状态**：✅ 完成（下一步 ADR-0100 + R-1 cargo 骨架）

## 记录 2：R-1 完成——cargo 骨架 + schema 迁移（2026-09-06）

**健康快照**：✅ R-1 全绿（14 passed：单元 10 + schema 集成 4）
**物理事实**：
- rs 分支已建；flowmodus-rs/ 独立 crate（prost 0.13 编译 5 个 proto 成功）
- prost 生成文件**无外层 package mod**（文件名即 package 名）——lib.rs 直接 include
- prost Message trait 在根（`use prost::Message`，非 prost::message）
**验证**：cargo build（21s）+ cargo test 全绿；4 个 schema 往返测试（telemetry/
deviation/raw_request+gossip）
**状态**：✅ 完成（下一步 R-2 五层行为等价迁移）

## 记录 3：R-2 完成——五层行为等价迁移（2026-09-06）

**健康快照**：✅ R-2 全绿（43 passed：单元 36 + e2e 3 + schema 4）
**物理事实**：
- L1 STE（estimate_token_count 启发式 ascii/4+non-ascii/1.5，Python 语义含 floor/边界）
- L2 registry（agent_role 匹配优先，无匹配回退全量）
- L2.5 deviation（Python claimed==0 语义 + settlement 加权 + snapshot 双查询）
- L3 cost（billing 计价 + kv savings 0.9 + context_window 门）
- L4 filter（预算/deviation 容忍/bias cap + priority cascade + rehab 康复）
- L5 score（softmax + 熵路由 instance-id 去相关 + role bonus +10）
- **确定性改进**：Python rehab 用内置 hash()（跨进程随机）→ rs 用 sha256 派生
  （同输入跨进程位级一致，ADR-0100 D4）；候选顺序保持 Python 输入序（不排序）
- **0 硬编码**：kv_cache 默认 300/"cache_control" 标注来源（whitepaper §2.2 示例）；
  rehab 默认 0.001/300 标注来源（Python dataclass 默认，用户可覆盖）
- prost proto3 message 字段生成 Option<T>（billing/kv_cache/capabilities/cost）
**验证**：cargo test 全绿；e2e 三用例（全链路确定性/硬过滤收敛/用户 bias 反转选择）
**状态**：✅ 完成（下一步 R-3 三调用模式 + 失败冷却恢复）

## 记录 4：R-3 完成——三调用模式 + 失败冷却恢复（2026-09-06）

**健康快照**：✅ R-3 全绿（55 passed：单元 48 + e2e 3 + schema 4）
**物理事实**：
- CallMode 解析（Python resolve_routing_decision 形状）：Manual（非空非 group 非 auto）/
  Group（group: 前缀）/ Auto（默认）
- Manual：registry 直查 endpoint，零管线开销（Python _manual_decision 语义）
- **Group 真实落地**：Python _group_decision 是占位（delegate auto）→ rs 实现
  优先级降序 + 同优先级确定性权重采样（instance-id 种子）；未知/空组回退 Auto
- Auto：全五层（Python _auto_decision 形状，候选 score=0.0 语义照搬）
- **HealthTracker 补齐 Python 空白**：Python health_states 只有 layer4 消费者、
  没有生产者（失败→DEGRADED 从未实现）→ rs 实现失败冷却+恢复
  （one-api 语义：首败 DEGRADED + 时间戳，连续 5 败 TERMINAL，成功复位；
  DEGRADED 冷却期由 layer4 should_rehabilitate 概率放行）
- 100% 寄生遥测：health 只吃真实请求结果，零探测零心跳（铁律 0）
- **0 硬编码**：degrade_after=1 / terminal_after=5 来源 HealthConfig 默认
  （one-api 语义 + 用户可覆盖，ADR-0100 D3）
**验证**：cargo test 全绿；router 8 用例（parse/manual/未知模型报错/组优先级/
权重确定性/回退/auto）+ health 5 用例（成功/降级/终结/复位/未知健康）
**状态**：✅ 完成（下一步 R-4 控制面 canonicalizer/verifier + 遥测）

## 记录 5：R-4 完成——控制面 + 寄生遥测（2026-09-06）

**健康快照**：✅ R-4 全绿（76 passed：单元 65 + 控制面 e2e 4 + pipeline e2e 3 + schema 4）
**物理事实**：
- canonicalizer：递归 key 排序 + 字符串值 Unicode NFC + 紧凑 JSON →
  签名前字节确定（whitepaper §2.3）；**补 Python 协议缺口**（Python 只做
  key 排序没做 NFC，ADR-0100 D8）
- verifier：Ed25519 单键 + M-of-N multisig（whitepaper §7.4 for-loop 语义）；
  **密钥显式注入**（无全局可变状态，Python 模块常量 + monkeypatch 设计收敛，
  ADR-0100 D10）；协议根公钥 = 编译期信任锚点常量（whitepaper §3.1 规定）
- anti-corruption：外部 JSON → 类型安全 proto 手工映射（prost 0.13 无
  serde feature，不引 pbjson 守极简依赖）；白名单 fail-closed；
  RateLimits 按需不映射（路由不消费，ADR-0100 D9）
- telemetry：寄生 collector（真实流量结果 → TelemetrySample append +
  派生聚合 hit_rate/health_counts）；classify_http_error 逐字迁移；
  SQLite 持久化裁剪为内存态（极简，持久化留 sidecar）
- e2e 闭环：canonicalize → sign → verify → 篡改拒绝（字节级）
**验证**：cargo test 全绿；canonicalizer 5 用例（key-sort/compact/NFC 折叠/
确定性/非法输入）+ verifier 6 用例（单键/多签阈值/确定性/锚点解析）+
anti-corruption 3 用例（全字段/错型拒绝/空包默认）+ telemetry 4 用例 +
控制面 e2e 4 用例
**状态**：✅ 完成（下一步 R-5 judge-points 契约打通）

## 记录 6：R-5 完成——judge-points 契约打通（2026-09-06）

**健康快照**：✅ R-5 全绿（83 passed：单元 72 + 控制面 e2e 4 + pipeline e2e 3 + schema 4）
**物理事实**：
- judge_points 模块落地契约 §1/§2 全部 Rules 后端（0 tokens 确定性判定）：
  JP-1 judge_suggested_mode（长度启发式）、JP-2 judge_budget_tier（长度+关键词表）
- 输出枚举 SuggestedMode/BudgetTier = 确定性契约面（后端切换不改变下游枚举）
- JudgePointsConfig 注入（默认值 = 契约 §2 示例启发式，用户可覆盖，0 硬编码）
- CLI 真实化：`flowmodus judge/measure/verify`（std::env 解析，不引 clap 守
  极简依赖）；src/main.rs 补 bin target（此前 lib-only 无法 cargo run）
- 真机验证：judge 中文短句→simple/endogenous、200 字符→complex/exogenous、
  measure "hello world 你好世界"→ste=5（Python 语义 11/4+4/1.5=5.42 截断）、
  verify 输出 canonical_len + sha256、未知命令 exit=2
- 契约 v1.1：§0.5 声明 Rules 后端由 FlowModus 提供；§6 基线口径修正
  （189 → ECOSYSTEM.md SSOT 当前基线）
**验证**：cargo test 全绿 + cargo run 真实 smoke 全通
**状态**：✅ 完成（下一步 R-6 文档链 + 全量验证 + 推送收口）

## 记录 7：R-6 完成——文档链收口，rs 重构全里程碑落地（2026-09-06）

**健康快照**：✅ R-6 全绿（83 passed；clippy -D warnings 零警告）
**物理事实**：
- clippy 全清（unnecessary_cast / is_multiple_of / if-collapse 3 项，--fix）
- 真机验证：cargo test 全绿 + CLI judge/measure/verify 真实运行通过
- 零硬编码自查：全部默认值有来源（whitepaper §2.2 / Python dataclass /
  one-api 语义 / 契约 §2 示例 / 协议根公钥 §3.1），用户可覆盖
- 依赖账本（极致节能）：prost / prost-build / serde / serde_json / sha2 /
  ed25519-dalek / unicode-normalization —— 无 web/ORM/daemon/UI（DNA 铁律 6）
- 文档链终态：VISION v2.0-rs ✅ / DNA v1.1 ✅ / PLAN v1.0 R-0..R-6 全 ✅ /
  GROWTH 记录 0-7 / ADR-0100（D1-D10）/ judge-points-contract v1.1 /
  生态同步 ECOSYSTEM.md（FlowModus 入库）
**状态**：✅ 完成（FlowModus rs 重构全部里程碑落地；后续按需迭代）
