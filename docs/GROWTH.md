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
