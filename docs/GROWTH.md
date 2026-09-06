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
