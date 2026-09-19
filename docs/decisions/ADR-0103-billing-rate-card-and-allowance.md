# ADR-0103: 计费模型——费率卡 + 确定性额度，STE 比较才有意义

- **状态**: Proposed
- **日期**: 2026-09-20
- **决策范围**: FlowModus（`schemas/supplier.proto` / `flowmodus-rs/proto/supplier.proto`、`layer3_cost.rs`）
- **关联**: `ADR-0100`（rs 重构）· `ADR-0101`（**free/paid 物理分离**，本 ADR 在其上扩展，不改它）· `ADR-0102`（判定面放既有 serve 面）· VISION v2.0-rs（度量衡第一范式）· DNA v1.1（0 硬编码、物理事实优先）· 生态侧 K-044 / `anaphase:ADR-0008`

> **本 ADR 只声明接口，不实现求值。** 用户的要求是「**必须留下这需求接口**，否则后补要增加工作量」。
> 新增字段全部**追加**且**留空即等于今天的行为**，所以合入它不会改变任何一个现有报价。

---

## 1. 背景：STE 是"度"，但"报价"目前表达不出来

FlowModus 的第一范式是**度量衡**：STE（Standard Token Equivalent）统一"度"。
但一把尺子只有在**被测的东西能被表达**时才有意义：

```
现状  BillingDeclaration = { currency, token_input, token_output,
                             compute_ms, audio_sec, video_frame, free_quota_daily }
现状  layer3_cost.rs     = 输入token × token_input + 输出token × token_output
```

**⇒ 一张平的单价表。** 它表达不了：峰谷/时段价、阶梯价、批量价、缓存读写的差价、以及"额度用完了会怎样"。
**⇒ 于是 `cost/STE` 这个跨供应商比较量，在真实计费方式下是错的 —— 而错的方式不可见。**

### 实测：一个"声明了但没被建模"的字段

`free_quota_daily` 存在，但**从不参与计价** —— `layer3_cost.rs` 完全没读它，
它只作为 `layer5_score.rs` 的一个加分（`free_tier_bonus`）。
**⇒ 声明与实现不一致，而声明是别人排期时会依赖的那一份。**

### 与 ADR-0101 的关系（不重开）

`ADR-0101`（Accepted，用户 2026-09-09 拍板）已经定了 **free/paid 物理分离**：
`registry/free/<id>.json` 的计费**强制全 0**，`registry/paid/<id>.json` 任意。
本 ADR **不修改它**，只回答它没有回答的问题：**免费额度用完之后发生什么**。
本轮用户再次确认方向：**默认彻底分离**，理由是"彻底杜绝混淆的机会"。

---

## 2. 决策

### D1｜费率卡 = 可乘因子的**数据**，不是每家一段代码

统一算式（**唯一一条**，所有计费方式都是它的特例）：

```
paid_u  = max(0, quantity_u − allowance_u)                      # 额度先抵扣
price_u = base_u × M_mode(mode) × M_tier(u, window_usage) × M_time(at)
cost    = Σ_u paid_u × price_u
```

- **四个因子全是声明里的数据**，求值是**纯函数且全域** ⇒ 没有 per-supplier 分支（0 硬编码）。
- **不声明 ⇒ 因子 = 中性（1）** ⇒ 今天的平表逐字不变，不会静默改口径。
- 平的单价表 = **一条无条件规则** ⇒ 向后兼容，不是替换。

**可比的量是 `cost_per_ste = cost / ste_total`，而它现在是时间的函数。**
这正是平表表达不了、而峰谷定价必须表达的东西 —— 也是"STE 才有价值、比较才有意义"的落点。

### D2｜额度与费率**分离**（用户本轮裁定）

**默认彻底分离，除非 100% 确认，否则不给混淆的机会。** 落成三条：

1. **额度不是费率。** `FreeAllowance` 与 `RateCard` 是两组独立字段，
   **额度不参与价格计算**，只在价格之前**抵扣数量**。⇒ `paid` 永远是 `(费率表, 抵扣后的量)` 的**纯函数**。
2. **免费额度永不自动作成付费。** `on_exhaust` 的**缺省值是 `EXHAUST`**（拒绝/改路由），
   **不是 `SPILL_TO_PAID`**。要溢到付费必须**显式声明**。
   ⇒ 用户说的"可能会吃到一点付费额度"这件事，**只有在声明里写明才可能发生**，而不是默认行为。
   ⇒ 且"用户根本没充钱"时，`EXHAUST` 就是干净地失败 —— 不可能产生意外账单。
3. **免费供应商仍然必须全 0 费率**（`ADR-0101` 不动）。
   免费额度**不使**一个供应商变成付费，付费费率**不使**一个免费额度变成收入。

**确定性 vs 运行态**（这决定字段住哪）：

| | 住哪 | 为什么 |
|---|---|---|
| 费率卡 + **确定性**额度（固定重置时刻） | **签名声明**（supplier.proto） | 可验证、可预测、可被面板直接渲染；同一请求同一时刻在任何地方同价 |
| 充值余额 / 促销 / 信用额度 | **运行态遥测**（metrics.proto） | **签名覆盖的是声明，不是活余额** —— 把活余额塞进声明，签名就失去意义 |

> 用户补充："未来如果可以获得确定性的剩余额度，那就更好了" ⇒ 那是把余额也变成**可预测事实**，
> 而它属于运行态那一侧；本 ADR 给它留了位置（D5 的 `allowance_remaining`），不假装现在有。

### D3｜声明形状（**追加字段，留空即今天行为**）

`BillingDeclaration` 追加两个字段（号 8+，老读者忽略）：

```proto
repeated RateRule      rules       = 8;   // 空 ⇒ 退回上面那三个平的 float 字段
repeated FreeAllowance allowances  = 9;   // 空 ⇒ 无额度，与今天一致
```

- `RateRule{ unit, price_per_unit, mode, window, window_multiplier, tiers[] }`
- `TimeWindow{ start_minute_utc, end_minute_utc, effective_from_unix, effective_until_unix }`
  —— **UTC 且声明化**，所以"同一时刻同价"是可判定的。
- `VolumeTier{ from_units, to_units, price_per_unit }` —— 阶梯，升序。
- `FreeAllowance{ unit, quantity, reset_minute_utc, reset_period_hours, on_exhaust }`
- `BillingUnit` / `BillingMode` / `AllowanceExhaustPolicy` 三个 enum，**0 = 未指定 = 中性/严格缺省**。

**扩展的兼容口径**沿用生态已裁定的 B2a/B2b：
老读者**忽略**不认识的扩展（可渐进升级）；但**认识的字段给了不认识的值**（如未知 enum）⇒ **拒绝 + 响**。

### D4｜检定：每家一张 golden 向量

度量衡三件套是「STE=度 / 声明偏移量=校准 / **一致性测试=检定**」。本 ADR 补齐第三件：

> 例：同一请求在 `00:30` 与 `12:30`，报价必须**恰好**相差声明倍数。

**这是"比较才有意义"唯一可证伪的地方** —— 没有它，费率卡只是一段没人验过的声明。

### D5｜面板读模型 → Cellrix 渲染（FlowModus 只出事实）

FlowModus 暴露一个**带版本号**的投影，Cellrix 负责显示；FlowModus 不做渲染：

| 字段 | 含义 |
|---|---|
| `cost_per_ste_now` | 当前时刻的跨供应商可比量 |
| `cost_per_ste_by_window` | 24h 曲线（峰谷使它是折线，不是点） |
| `allowance_remaining` | 只在**确定性**额度可算时有值，否则显式 `unknown` |
| `rate_card_version` / `declared_at` / 签名 | 这份报价对应哪一版声明 |

**⇒ 与 D2 的"确定性 vs 运行态"一一对应**：曲线来自声明，剩余额度来自运行态。

### D6｜落地顺序（本 ADR 只做第 0 步）

| # | 步骤 | 状态 |
|---|---|---|
| **0** | **只加声明字段**（本 ADR） | ✅ 本次 |
| 1 | 统一求值函数（纯）+ 单测 | 未做 |
| 2 | deepseek / kimi / glm / openai 四家费率卡 + golden 向量 | 未做 |
| 3 | `is_free_supplier` 扩展到"没有任何非零价格" | ✅ 本次（见下） |
| 4 | Cellrix 面板 | 未做 |

`is_free_supplier` 本次一并扩展为「平字段为 0 **且** 所有 `rules` 的 `price_per_unit` 为 0」，
否则一个用 `rules` 表达非零价格的声明会被误判为免费 —— **那正是免费层强制全 0 想防的事**。

**⚠️ 既存缺口（不在本次范围内，如实记下）**：现有实现只查 `token_input`/`token_output`，
**没查 `compute_ms` / `audio_sec` / `video_frame`**。一个 token 价为 0 但音频价非零的声明，
今天会被判为免费。修它会改变现有声明的判定 ⇒ 需要独立决定，故此处只记录、不顺手改。

---

## 3. 影响

- **正面**：`cost_per_ste` 从"一个数"变成"一条随时段变化的折线"；峰谷/阶梯/批量/额度**都变成数据**；
  Cellrix 有稳定的读模型可渲染。
- **代价（不粉饰）**：声明变复杂，填写者要理解因子语义。**用「不填即中性」抵消**：
  只需要平表的供应商，填法一个字都不用改。
- **不解决**：本 ADR **不实现求值函数**，所以现阶段 `layer3_cost.rs` 的行为**逐字不变**。
  声明存在但未被消费 —— **这本身要被记账**（见 D6 的步骤 1），否则就是又一个
  "声明了但没被建模"的 `free_quota_daily`。

### 一处必须先解决的同族缺陷：这个 schema 有**三份**副本

实测：

| 副本 | 角色 |
|---|---|
| `flowmodus-rs/proto/supplier.proto` | **`build.rs` 实际读这份**（Rust 代码生成的源） |
| `schemas/supplier.proto` | 对外发布的 schema |
| `src/flowmodus/schemas/supplier_pb2.py` + `.pyi` | **Python 侧的生成物** |

`.proto` 两份目前**逐字节一致**（五个 proto 文件共 10 份，全部一致），但**没有一致性断言**（`build.rs` 不读 `schemas/`，也没有脚本比对）。
**三份手工维护的副本 = K-002 同族**：改一份忘另一份，代码生成与"发布出去的 schema"就会**静默分叉**。

**⇒ 本次的处置（有意的、被记录的）**：

1. 两份 `.proto` **已同步修改**（逐字节一致已复验）。
2. **Python 生成物本次不重新生成。** 理由：`protoc` 在本机是 **34.1**，而现有 `supplier_pb2.py` 是别的版本生成的 ——
   用不同版本重写整个生成物会带上不同的 `_runtime_version` 守卫，可能与已安装的 protobuf 运行时冲突。
   **这是真实风险，不是为了省事。**
3. **⇒ 后果必须说清**：Python 侧现在**不认识** `rules` / `allowances`。
   这在**当前是安全的**，因为两者缺省为空 ⇒ Python 的行为**逐字不变**。
   但**在任何费率卡真正被使用之前必须重新生成**，否则"Rust 按费率卡算、Python 按平表算"会给出**两个不同的报价**，
   而两者都自称是同一份声明。
4. **⇒ 建议（独立决定，此处只标记）**：定一份为源、其余**生成**，并加一条一致性断言；
   在那之前，**每改一次这个 schema 都要手工同步三处**，且必须在提交信息里写明做了。

> **旁证：Python 侧与 Rust 侧是同一模型的两份实现。**
> `src/flowmodus/data_plane/pipeline/layer3_cost.py` 与 `flowmodus-rs/src/layer3_cost.rs`
> 用的是**同一个平表公式**（`输入×token_input + 输出×token_output`）。
> 所以本 ADR 的统一算式**将来要在两处实现**，而 D4 的 golden 向量正是用来**证明两份给同一个答案**的 ——
> 那才是"一致性测试=检定"在这里的具体用途。

---

## 4. 参考实现的核对：一个第三方模型管理客户端（2026-09-20，**读源码而非市场文案**）

> **⚠️ 本节不写它的名字** —— 本仓规则：**活文档不出现外部项目名**。
> 需要核实时，**下引的类型定义与字段名都是逐字抄录的，可按内容检索**；身份不在此文件里承载。
> **取数方式**：对该参考实现做 `--depth 1` 浅克隆（36 MB）后逐文件读，缓存放在工作区之外。
> **顺带记一条本机网络事实**（它曾让几轮引用无法核验）：DNS 把域名解析到 `198.18.0.0/15`
> （保留段，代理的 fake-IP 模式），所以 `web_fetch` 一律拒读（非公网 IP 护栏）；
> 但 **`git` / `curl` 走代理是通的** ⇒ **引用可核，且已在别处据此更正了三处**。

### 4.1 它**是**什么的参考（三条，逐条有据）

**(1) 免费额度的分类 —— 本 ADR 的 D2 缺一维。** 其 `api/freeModels.ts`：

```ts
export type FreeModelType = 'perpetual' | 'renewing-quota' | 'recurring-credit' | 'trial-credit';
```

| 它的类型 | 本 ADR 的 `FreeAllowance` 能表达吗 |
|---|---|
| `renewing-quota`（周期性配额） | ✅ `reset_period_hours = 24` |
| `trial-credit`（一次性） | ✅ `reset_period_hours = 0` |
| `perpetual`（永久免费） | ❌ **本 ADR 强制要求 `quantity`**，表达不出"没有额度上限" |
| `recurring-credit`（**周期性信用**） | ❌ 本 ADR 是**单位量**（token 等）模型，**没有货币/信用这一维** |

**⇒ 这正是用户说的"分离一个免费额度分类"。** D2 必须补 `perpetual` 与 `credit` 两种形态，
否则把一家"每月送 $5 信用"的供应商硬塞进"每月送 N token"的模型里，**比较会算错，而错得不可见**。

**(2) 与计费无关、但决定"这条路能不能走"的事实。** 它的免费条目还带
`cardRequired` · `phoneRequired` · `commercialOk`（可空布尔）· `verifiedAt`。
**这不是计费，是资格与合规** —— 但它们同样是**声明**，同样需要"何时核验过"的时间戳。
**⇒ 归供应商声明，不归 `BillingDeclaration`**（单一职责，别把两种事实混成一张表）。

**(3) 版本化目录本身就是声明。** 它的目录结构是
`{ version, updatedAt, models[] }` —— 带版本号与更新时间的声明，
与 D2 的"确定性进签名声明"是同一形状。**旁证，不是新决定。**

### 4.2 它**不是**什么的参考（这条必须说清）

**它没有调度算法。** 实测：在其前端与 Rust 侧搜
`weight|priority|fallback|load-balanc|round-robin|routing`，去掉 CSS 的 `font-weight` 之后，
**只剩免费模型列表的优先级徽章**（一个 i18n 键 `freeModels.router.priority`，
与一个可拖拽、带数字徽章的有序列表组件）。

**⇒ 它的 "router" 是"把免费模型排个序"，不是评分/采样/降级。**
而 FlowModus **已经有**更多：softmax + 确定性加权采样 + 降级/康复（`layer5_score.rs` / `layer4_filter.rs`）。
**⇒ "参考它补调度逻辑"这一条不成立；它能补的是"排序交互"与"覆盖面"。**

### 4.3 一条交互形态上的收获

它的用户偏好表达是**有序列表 + 整数优先级（拖拽即改）**，**不是自由浮点权重**。

**⇒ 这支持一件事**：把**整数优先级 + 显式顺序**作为**面向用户的主控件**，
`bias_score: f64` 留作**细调**层。理由是可解释性 —— 用户能说清"我把它排第 3"，
说不清"我把它的 bias 设成 0.37"。**不可解释的权重等于没有权重（D5 已立此论）。**

### 4.4 结论

| 它的元素 | 处置 |
|---|---|
| 免费额度四分类 | ✅ **吸收** —— 补 D2 缺的那一维 |
| `cardRequired` / `phoneRequired` / `commercialOk` / `verifiedAt` | ✅ **吸收**，归供应商声明，**不归计费** |
| 版本化目录 | ✅ 旁证（与 D2 一致） |
| **供应商面覆盖清单** | ✅ **吸收** —— 作为"要适配哪些供应商面"的依据 |
| 整数优先级 + 拖拽排序 | ✅ **吸收为交互形态** |
| 调度算法 | ❌ **它没有**；FlowModus 已有的更强 |
| 它的 UI 本身 | ❌ 归消费者（`ADR-0101`：不做又一个管理台） |

**⚠️ 本节的 D2 补充尚未落到 proto**：`FreeAllowance` 目前仍只有"单位量 + 重置"一种形态。
补 `perpetual` / `credit` 属接口变更，**是下一步的第一件**，此处先把它记成已采纳的决定，
避免它退化成又一个"声明了但没有入口"。

---

*状态：Proposed。接口先冻结，实现另开切片。*
