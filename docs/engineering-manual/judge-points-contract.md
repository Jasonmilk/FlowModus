# 判断点契约（Judge-Points Contract）

**版本**：v1.0-draft（不冻结，随生态演进）
**日期**：2026-09-06
**消费方**：Anaphase-Helix（编排中枢）
**提供方**：FlowModus（LLM API 调度层，5 层确定性流水线）
**语言**：中文正文，英文协议/字段

---

## §0 目的

Helix 生态"按需调用需要的 LLM 类型/大小"由本契约承载：把**判断点**（judge
points）——编排中需要"判断/分类/评估"的位置——显式列出，标注每个判断点的
后端选项、LLM 类型/大小需求与失败回退。FlowModus 按此契约建模型池与路由组；
Anaphase 按此契约在判断点选择后端。

**铁律**：
- 确定性契约判据永远走 Rules（0 tokens），不得用 LLM 替代（见 §3）；
- 语义判断点 Rules 为默认（0 tokens 优先），SmallLlm 为可选（质量 ROI 足够高时）；
- **显式选择**，不做系统级 Auto Router（M3 边界：禁无脑自动路由，不禁按需判断点）；
- SmallLlm 失败一律回退 Rules（fail-safe，确定性优先）；
- 每个判断点有唯一编号（JP-x），跨文档引用。

---

## §1 判断点清单

| # | 判断点 | 输入 | 输出 | 默认后端 | 可换后端 | LLM 需求 |
|---|---|---|---|---|---|---|
| JP-1 | 复杂度评估（suggested_mode） | user_input | simple / moderate / complex | Rules（长度启发式） | SmallLlm | 3B 级分类 |
| JP-2 | 意图分类（budget_tier） | user_input | endogenous / augmentable / exogenous | Rules（长度+关键词表） | SmallLlm | 3B 级分类 |
| JP-3 | Rails 命中判定 | user_input | hit / miss | Rules（确定性匹配） | 无（0 tokens 是 Rails 的意义） | — |
| JP-4 | 结构化命令解析 | user_input | tool JSON | Rules（语法解析） | 无（语法确定性） | — |
| JP-5 | 判据核验（criteria） | evidence | MET / UNMET | Rules（契约函数） | 无（契约不得交 LLM） | — |
| JP-6 | 记忆相关度排序（将来） | query + nodes | 排序结果 | Rules（预算截断） | SmallLlm | 3B 级重排 |
| JP-7 | 元认知差距评估（Mind 侧） | expected vs actual feedback | 差距标度 | Mind 价值评估器 | Mind 自决 | 由 Mind 定 |

---

## §2 语义判断点细则（JP-1 / JP-2）

### JP-1 复杂度评估
- **Rules 后端（当前）**：输入字符长度启发式（skilled_len / anchor_len，config
  来源）。输出映射：`≤ skilled_len → simple`，`< anchor_len → moderate`，
  `≥ anchor_len → complex`。
- **SmallLlm 后端（可选）**：3B 级分类模型，输出三选一标签。映射到同一枚举，
  保证下游不变（suggested_mode 枚举是确定性契约）。
- **回退**：SmallLlm 超时 / 非三选一输出 / 错误 → 回退 Rules。

### JP-2 意图分类
- **Rules 后端（当前）**：长度 + 关键词表（explore_keywords，config 来源）。
- **SmallLlm 后端（可选）**：3B 级分类模型，输出三选一标签（endogenous /
  augmentable / exogenous），映射到 budget_tier 枚举。
- **回退**：同上，回退 Rules。

### 共性约束
- SmallLlm 输出**必须**是枚举内标签，任何自由文本视为失败（确定性优先）；
- 分类请求走 FlowModus **Manual**（直连指定 3B 模型）或 **Group**（"small-llm"
  组，多供应商标配）——Anaphase 不关心具体模型，只声明"3B 级分类"需求；
- 请求体遵循 FlowModus protobuf schema（routing.proto / supplier.proto）。

---

## §3 永不换后端的判断点（确定性契约）

| 判断点 | 原因 |
|---|---|
| JP-3 Rails 命中 | Rails 的意义就是 0 tokens 原文引用；LLM 参与即引入幻觉风险 |
| JP-4 命令解析 | 语法确定性；LLM 解析结构化命令是过度工程 |
| JP-5 判据核验 | 契约函数（threshold/tolerance 等）不得交 LLM 判断 |

---

## §4 模型池需求规格（FlowModus 输入）

| 档位 | 模型类型 | 用途 | 调用模式 | 备注 |
|---|---|---|---|---|
| S（small） | 3B 级分类 | JP-1 / JP-2 / JP-6 | Manual / Group("small-llm") | 低延迟，成本≈0 |
| E（embedding） | 嵌入模型（将来） | 记忆检索相似度 | Group | 与 Mind SA-Core 检索对齐 |
| L（large） | 主通道大模型 | Reasoning 生成 | Manual / Auto | 用户指定或 5 层路由 |
| X（本地） | 本地 LLM（如 llama） | 敏感话题本地执行（Tuck 兼容 OpenAI base URL） | Manual | 安全兜底，见 §5 |

---

## §5 安全边界（与 Tuck 的关系）

- 敏感话题 / 敏感数据：走**本地 LLM**（Tuck 已提供 OpenAI 兼容 base URL），
  统一由 FlowModus 调度、Tuck 管控——**模型选择权在 FlowModus，数据主权在本地**；
- 凭证（OAuth / API Key）锁在 Tuck，Mind / FlowModus 只拿标签与调度权，不见凭证；
- 任何判断点的 LLM 调用都过 Tuck 管控（CAPABILITY-13 模式权限）。

---

## §6 验收口径

1. 每个判断点：后端可配置（config），默认 Rules（0 tokens 优先）；
2. SmallLlm 输出非法 → 回退 Rules，无半结构化污染（fail-safe）；
3. 下游枚举（suggested_mode / budget_tier）不变——后端切换对编排透明；
4. 全生态测试数不回退（Anaphase 189 基线）。

---

**文档结束**
> 本契约是 Anaphase（消费方）与 FlowModus（提供方）之间的需求规格，随生态演进
> 更新；不冻结，好用且不再改时再冻结。
