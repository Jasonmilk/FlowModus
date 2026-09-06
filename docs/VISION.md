# FlowModus VISION

> **Helix 生态的 LLM API 调度确定性协议层**——零浪费的本地 sidecar。
> 版本 v2.0-rs（重构宣言）｜ 2026-09-06 ｜ Jasonmilk / CommonIntents
> 继承 Python v1.7 的哲学与铁律；**实现重构，哲学演进**。

---

## 〇、一句话

> 众人做的是 API 网关——转发、计费、闭源黑盒；
> FlowModus 做的是**调度宪法**——每一条路由决策都可验证、可审计、无浪费。

## 一、为什么存在（问题）

LLM API 调用是 Helix 生态的呼吸。当前世界的通病：

1. **路由黑盒**：网关按供应商利润路由，不按用户意图；计费不透明；
2. **确定性缺失**：同请求不同结果，延迟不可预测，无法回放审计；
3. **计算浪费**：无差别的"全走大模型"——大多数查询本不需要前沿模型；
4. **凭证扩散**：每个 Agent 一份 Key，泄露面随组件数线性增长。

## 二、FlowModus 是什么（定位）

本地 sidecar（`localhost`），介于 Agent（Anaphase/Callosum 等）与 LLM 供应商之间，
执行**数学上可验证的确定性调度**：

- 五层不可变流水线：归一化 → 签名注册表 → 成本推断 → 硬过滤 → 熵权重路由；
- 注册表经 Ed25519 签名，路由表可验证、可溯源；
- 遥测 100% 寄生于真实流量（**零主动探测**），成本归属逐 token 可验证；
- 凭证零接触：Key 锁在 Tuck，FlowModus 只见调度权。

## 三、核心立场（与世界的分界）

| 立场 | FlowModus 立场 | 世界主流（拒绝） |
|---|---|---|
| 路由依据 | **用户约束 + 确定性信号**（注册表/成本/预算/熵） | 供应商利润 / 黑盒模型 |
| 遥测 | **寄生**（真实流量派生） | 主动探测（心跳/探针） |
| 确定性 | 同输入 → 位级相同输出（instance-id 去相关） | 概率性路由 |
| 凭证 | **零接触**（Tuck 物理持有） | Agent 各自持有 |
| 模型选择 | **显式判断点**（judge-points 契约：Rules → 3B → L） | 无脑自动路由 |
| 学习 | 确定性升级链 + 判据信号（可审计） | 在线学习路由（黑盒训练） |
| 依赖 | 极简（无 web 框架/无 daemon/无 ORM） | 重型网关栈 |

## 四、级联升级链（VISION 的灵魂）

**FrugalGPT 范式，Helix 化**：大多数查询不需要前沿模型——这不是猜测，是
RouteLLM/FrugalGPT 的实证结论（85%+ 查询可小模型解决，98% 成本削减）。

FlowModus 的升级链由**确定性判据**驱动，不是 LLM 自评：

```
Rules（0 tokens，默认）
   ↓ 判据不满足（复杂度/意图/置信信号）
3B 级分类（judge-points JP-1/JP-2）
   ↓ 仍不满足
大模型（主通道，用户指定或约束路由）
   ↓ 极端/敏感
本地 LLM（Tuck 兼容端点，数据主权兜底）
```

- 升级信号来自 judge-points 契约 + criteria 判据（**可审计**）；
- 每一级失败/不确定 → 显式升级，不做黑盒自评；
- 与 Helix 编排哲学同构：**0 tokens 优先 > 少 tokens/小 LLM > 多 tokens/大 LLM**。

## 五、五层流水线（不可变资产，重构继承）

```
1. Normalization（STE）  供应商 token → 标准 token 等价（可验证换算）
2. Raw Registry          Ed25519 签名供应商声明（IPFS/IPNS 分发）
3. Cost Inference        实时成本估计（缓存命中 + 声明偏差校正）
4. Hard Filter          预算/地域/驻留/用户约束（本地强制执行）
5. Entropy Routing       确定性抖动路由（instance-id 去相关，防羊群）
```

- 五层顺序不可变、不可跳过（铁律）；
- schema（protobuf）是不可变契约资产，rs 重构直接继承；
- **新增语义（吸收成熟实践）**：失败冷却 + 恢复（one-api 语义，确定性来源真实流量）。

## 六、边界（哪些不是 FlowModus）

- ❌ 不是网关 UI/管理台——展示与管理（EchoBird Model Nexus 范式）属 Cellrix 驾驶舱；
- ❌ 不持有凭证——Tuck 物理持有，FlowModus 只拿调度权；
- ❌ 不做在线学习路由（RouteLLM/EcoRouter）——M3 边界，记未来可选，不冻结；
- ❌ 不主动探测（EchoBird Auto Router 的探测）——最小作用量铁律，遥测寄生；
- ❌ 不做权限/分组——CAPABILITY-13 的事，不重复建设；
- ❌ 不内置 UI/daemon/ORM/web 框架——极简依赖铁律。

## 七、未来开放项（不冻结，好用再说）

- **O-1**：偏好数据升级路径（RouteLLM 式）——需训练管线，M3 之后评估；
- **O-2**：多协议适配器（Anthropic 等）——供应商适配层扩展；
- **O-3**：KV 缓存命中协同（与 Callosum 对齐）——前缀稳定后的成本红利。

## 八、成功判据（rs 重构验收）

1. 五层流水线 + protobuf schema 全部迁移，行为与 Python v1.7 等价（测试对齐）；
2. 铁律 8 条 + Helix 哲学 8 条全部生效，代码可核验；
3. judge-points 契约（JP 清单）与 FlowModus 调度语义打通；
4. 无主动探测（grep 核验：无 heartbeat/probe 字面量）；
5. 测试全绿 + README + ADR 维护 + 推 GitHub。

---

> 众人做的是转发，FlowModus 做的是宪法。
> 每一条边都有签名，每一分钱都有出处，每一次调用都不浪费。
