# FlowModus GROWTH——生长记录

> 记录健康快照；超过 3 条时归档最旧的到 docs/archive/growth/。

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

## 记录 8：README 全面更新 + 生态保护落地（2026-09-06）

**健康快照**：✅ 文档轮（83 测试不受影响）
**物理事实**：
- **README 腐烂实锤**：顶部 4 个生态链接（CIS/CAP/CISS/CIB）全部失效（HTTP 301），
  真实仓库名为 INTENT-7 / CAPABILITY-13 / INTENT-7-SECURE / BIND-19（HTTP 200）
- 全量重写 README：度量衡定位（非网关/非代理/非管理台）、铁律 0、双分支
  说明（rs 主推 83 测试 / main Python 冻结）、真机验证过的 CLI 命令、
  五层管线表、三调用模式、控制面与遥测、与生态关系（judge-points 契约
  两端就绪）、文档链导航、许可与保护
- **生态保护零成本落地**（防御性公开）：
  - NOTICE（Apache 归属声明）
  - docs/prior-art.md（10 项代码级创新点 + 2 项协议级创新，证据路径到
    commit，GitHub 公开历史 = 2026-09-06 先有技术证据）
  - VISION.md 头部版权/许可行
  - main 分支 README 同步修链接 + rs 指引（Python 冻结版门面不再腐烂）
- **外部保护建议审查结论（物理事实优先）**：那段建议有 2 处事实错误——
  ①"FlowModus 现在 MIT 需要改"（实际 2026-09-06 已双分支切 Apache 2.0）；
  ②"上一轮已生成防御性公开 PDF"（实际不存在，本轮才落地）。
  方向正确（协议公开 = 采用率），动作按需裁剪：GitHub 公开历史即最硬
  先有技术证据，IP.com/OIN/正式专利申请均降级为"有需要再动"的议程项
**状态**：✅ 完成
