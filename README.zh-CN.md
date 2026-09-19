# FlowModus

> **English**: [README.md](./README.md)

**LLM API 调度的度量衡局（Weights & Measures Bureau）** · The Deterministic Protocol Layer for LLM API Scheduling

[Helix 生态](https://github.com/Jasonmilk) · [CommonIntents 协议家族](https://github.com/CommonIntents) · [CI-144 语义层 INTENT-7](https://github.com/CommonIntents/INTENT-7) · [CI-144 能力层 CAPABILITY-13](https://github.com/CommonIntents/CAPABILITY-13) · [CI-144 安全层 INTENT-7-SECURE](https://github.com/CommonIntents/INTENT-7-SECURE) · [CI-144 传输层 BIND-19](https://github.com/CommonIntents/BIND-19)

**许可证**：Apache 2.0（代码 + 白皮书，双分支同步）｜ **定位**：独立协议，零 Helix 依赖，任何 Agent 框架可消费

---

## 是什么

FlowModus 不是网关，不是代理，不是管理台。它是一套**度量衡**——LLM API 调度的
确定性计量标准与裁决呈现层：

| 度量衡三件套 | 落地 |
|---|---|
| **度（STE）** | 标准 Token 等价（Standard Token Equivalent）——把各家 token/计费口径折叠到同一把尺子 |
| **校准（声明偏移量）** | 供应商声明 vs 真实遥测的偏差度量（Deviation） |
| **检定（一致性测试）** | 五层确定性管线，同输入 → 同输出，跨进程字节级一致 |

**核心立场（VISION v2.0-rs）**：**不判断，只呈现，判断权属于用户。**
管线把度量、成本、过滤、评分全部呈现为确定性数据；选择由用户/编排层做出。

## 铁律

- **铁律 0（零探测）**：废除一切定时心跳/主动探测。遥测 100% 寄生真实流量；
  唯一例外 = 用户显式按需探测。
- **五层不可变**：Normalization(STE) → Registry(Ed25519 签名) → Deviation →
  Cost(标准化计价) → Filter(硬边界) → Score(软加权 + 熵路由)。
- **确定性优先**：熵路由 seed = sha256(instance:request)，同输入跨进程位级一致。
- **零硬编码**：每个默认值有来源（whitepaper / Python dataclass / one-api 语义 /
  协议根公钥），用户可覆盖。

## 分支

| 分支 | 内容 | 状态 |
|---|---|---|
| **rs** | Rust 重构（R-1..R-6 全部完成） | ✅ 主推，83 测试全绿，clippy 零警告 |
| **main** | Python v1.7（白皮书原版实现） | 🧊 冻结维护 |

rs 分支是确定性重构：三调用模式真实落地（Python 的 Group 原是占位 stub）、
失败冷却恢复补齐（Python 的 health 只有消费者没有生产者）、canonicalizer 补
Unicode NFC 协议缺口（Python 只做 key 排序）。

## 运行

```bash
git clone https://github.com/Jasonmilk/FlowModus.git
cd FlowModus/flowmodus-rs

cargo test        # 87 passed（五层 + 三调用模式 + 控制面 + judge-points + 注册台）
cargo run -- judge "帮我探索这个主题"            # JP-1/JP-2 Rules 判定（0 tokens）
cargo run -- measure "hello world 你好世界"      # STE（度）
cargo run -- verify '{"version":"v2.0-alpha","suppliers":[]}'  # 规范化 + sha256（检定）
```

CLI 命令：`judge`（判断点 Rules 判定）/ `measure`（STE）/ `verify`（签名规范化）/ `supplier`（注册台管理）/ `route`（确定性决策）。
解析用 std::env，无 clap——依赖极简。

## 供应商注册台（度量衡注册台）

免费与付费供应商**物理分离**——一供应商一文件：

```
registry/
  free/<supplier_id>.json     # 计费必须全 0（强制，物理事实）
  paid/<supplier_id>.json     # 任意计费
```

```bash
cargo run -- supplier add --tier free --id groq --base-url https://api.groq.com/openai/v1 \
    --auth bearer_token --model llama-3.3-70b          # free 层拒绝非零计费
cargo run -- supplier list [--tier free|paid]
cargo run -- supplier get --tier paid --id volc
cargo run -- supplier test --tier free --id groq       # 显式按需探测（0 空闲探针）
cargo run -- supplier rm --tier free --id groq
cargo run -- route --model auto --prompt "hello"       # 免费优先软优先
```

- 注册的 JSON 文件为**用户本地**（gitignore——端点/密钥不进仓库）。
- `route --model auto` 在 L5 施加免费层软加分（0 tokens 优先生存哲学）；付费供应商兜底。判断权属于用户——`BiasConfig.free_tier_bonus` 可覆盖。
- 调用模式：`auto`（五层管线）/ `group:NAME` / 显式 `MODEL_ID`。

## 五层确定性管线

| 层 | 职责 | 确定性保证 |
|---|---|---|
| L1 Normalization | STE 估算 + prompt sha256 | ascii/4 + non-ascii/1.5，floor，min 1 |
| L2 Registry | Ed25519 签名供应商声明 | 验签 + 结构校验（anti-corruption fail-closed） |
| L2.5 Deviation | 声明 vs 实际偏移量 | settlement 加权（weight=i+1） |
| L3 Cost | 标准化计价 | billing + kv 节省 ×0.9 + context_window 门 |
| L4 Filter | 用户硬边界 | 预算/偏差容忍/供应商 bias 上限 + priority cascade + 康复判定 |
| L5 Score | 软加权 + 熵路由 | softmax + sha256 派生种子（跨进程位级一致） |

## 三调用模式

- **Manual**：直连指定模型，零管线开销（`model = "s1-fast"`）
- **Group**：用户路由组，优先级降序 + 同优先级确定性权重采样（`model = "group:fast-lane"`）
- **Auto**：全五层管线（`model = "auto"` 或空）

## 控制面与遥测

- **canonicalizer**：递归 key 排序 + Unicode NFC + 紧凑 JSON → 签名前字节确定
- **verifier**：Ed25519 单键 + M-of-N multisig（密钥注入，无全局可变状态）
- **anti-corruption**：外部 JSON → 类型安全 proto，白名单 fail-closed
- **health**：寄生失败冷却（首败 DEGRADED → 连续 5 败 TERMINAL → 成功复位），one-api 语义
- **telemetry**：真实流量采样 + 派生聚合（hit_rate / health_counts）

## 与 Helix 生态的关系

- **独立中立**：核心 crate 零 Helix 依赖，外部可独立引用（DNA 铁律 7）
- **判断点契约**：`docs/engineering-manual/judge-points-contract.md`（v1.1）——
  FlowModus 提供 JP-1/JP-2 的 Rules 后端（0 tokens 确定性判定）；
  Anaphase 侧 O-6（ADR-0024）消费方已就绪，SmallLlm 可换后端由消费方选择，
  非法输出一律回退 Rules（fail-safe）
- **编排边界**：并行池 / 上下文窗口感知归 FlowModus；模型选择权在 FlowModus，
  凭证锁在 Tuck，数据主权在本地

## 文档链（phyt-DNA 方法论）

| 文档 | 内容 | 版本 |
|---|---|---|
| [VISION.md](docs/VISION.md) | 度量衡宣言 | v2.0-rs |
| [DNA.md](docs/DNA.md) | 不可变原则（铁律） | v1.1 |
| [RNA.md](docs/RNA.md) | 方法论加载协议 | — |
| [PLAN.md](docs/PLAN.md) | 开发导航牌 | R-0..R-6 全 ✅ |
| [GROWTH.md](docs/GROWTH.md) | 生长记录 | 记录 0-7 |
| [ADR-0100](docs/decisions/ADR-0100-rs-refactor.md) | rs 重构决策记录 | D1-D10 |
| [judge-points-contract](docs/engineering-manual/judge-points-contract.md) | 判断点契约 | v1.1 |
| [whitepaper-v1.7](docs/whitepaper-v1.7.md) | 协议白皮书 | v1.7.1 |
| [prior-art](docs/prior-art.md) | 防御性公开 / 先有技术记录 | 2026-09-06 |

## 许可与保护

- **Apache 2.0**：代码 + 白皮书（双分支同步，v1.7.1 变更记录可溯）
- **NOTICE**：Apache 归属声明（见仓库根目录）
- **防御性公开**：核心创新点已在 `docs/prior-art.md` + GitHub 公开 git 历史
  构成 2026-09-06 的公开披露（prior art），防止第三方抢注专利
- 参考灵感（非借用）：成熟的第三方 API 管理项目（展示与管理 API 形态）——
  明确拒绝网关 / 代理 / 管理台形态，不重复造轮子

---

*Built with mathematical rigor by the FlowModus Community.*
