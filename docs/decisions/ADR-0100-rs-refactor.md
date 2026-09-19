# ADR-0100: FlowModus rs 重构立项——度量衡范式的 Rust 落点

- **状态**: Accepted（用户审查 VISION v2.0-rs 放行）
- **日期**: 2026-09-06
- **决策范围**: FlowModus（Python v1.7 → Rust rs 分支）
- **关联**: VISION v2.0-rs / DNA v1.1 / PLAN v1.0 / whitepaper v1.7 / manual v1.1.3 /
  judge-points-contract v1.0-draft

## 1. 背景

FlowModus Python v1.7 五层管线已成形（~2375 行 / 53 测试 / 5 个 protobuf schema），
但工程形态为解释型脚本，无法满足"度量衡局"对确定性、可分发、零依赖的长期承诺
（Phase 3：多语言参考实现）。用户授权 rs 分支重构，要求哲学继承、行为等价、
铁律生效。

## 2. 决策

### D1: 工程形态——单 crate 模块化（flowmodus-rs/）

- 仓库 rs 分支内新建 `flowmodus-rs/` 独立 crate（包名 `flowmodus`）；
- Python v1.7 源码保留为**行为参照**（物理事实优先：等价迁移以真实代码为准）；
- 模块 = 五层管线（layer1..layer5）+ control_plane + telemetry + cli；
  层间经 protobuf 契约（DNA 铁律 5），不物理分 crate（如无必要勿增实体）。

### D2: 探测立场裁决——铁律 0 优先（修正 whitepaper 滞后）

- whitepaper v1.7 §5.5 "每小时 Canary Probe" 为文档滞后；manual v1.1.3 铁律 0
  （更晚更严）为准：**废除定时探测，遥测 100% 寄生真实流量；唯一例外 = 用户显式
  按需探测（成本提前告知）**；
- L2.5 声明偏移量 = 寄生遥测 + 按需探测派生；无心跳、无轮询；
- **后续动作**：whitepaper 下一版本（v1.8）修订 §5.5 对齐铁律 0。

### D3: 不造轮子边界——拒绝网关形态

- 不实现第三方 API 管理台式的网关/管理台/UI（VISION §七）；

> **注（2026-09-20）**：此处原具名两个外部项目，按「活文档不出现外部项目名」改为描述性表述，约束未改。
- 仅吸收成熟运维语义：**失败冷却 + 恢复**（确定性，来源真实流量遥测）入
  L4/L5 辅助；模型别名/重定向（注册表层可验证映射）；
- 展示/管理/驾驶舱属消费者（Cellrix），不进 FlowModus。

### D4: 熵增路由的确定性语义

- 熵增路由 = instance-id 去相关：同 instance-id 同输入 → 位级相同输出（确定性
  保留）；不同 instance-id 才分散（防羊群）；
- 概率分布抽样在 L5 内完成，输入含 instance-id 派生种子，无全局随机。

### D5: 迁移策略——行为等价

- Python 53 测试语义 → Rust 测试向量对齐（同输入同输出）；
- 5 个 protobuf 契约原样编译进 crate（prost-build），字段/语义零改动；
- 新增长：失败冷却恢复测试、确定性熵路由测试（同种子同输出）。

### D6: 依赖极简清单（按需加载）

- R-1: `prost` / `prost-build` / `serde`；
- R-2+: `ed25519-dalek`（注册表验签）、`tokio` + `tonic`（sidecar 代理）；
- 禁止：web 框架 / ORM / daemon / UI 库（DNA 铁律 6）。

### D8: 控制面——canonicalizer NFC 协议缺口补全（R-4）

- whitepaper §2.3 要求签名前 Unicode NFC 归一；Python v1.7 canonicalizer
  只做 key 排序 + 紧凑 JSON，**未做 NFC**（协议缺口）。
- rs 补全：`canonicalize_json` = 递归 key 排序（BTreeMap）→ 字符串值 NFC →
  紧凑序列化（serde_json 默认，等价 Python `separators=(',',':'), ensure_ascii=False`）。
- 理由：签名字节跨平台一致性的协议承诺，不应因实现偷懒而缩水。

### D9: anti-corruption 映射范围——按需驱动（R-4）

- Python 用 `ParseDict` 全量转换；rs 手工映射（prost 0.13 无 serde feature，
  引入 pbjson/prost-reflect 违反 DNA 铁律 6 极简依赖）。
- 映射覆盖 RegistryPackage / Supplier / Model / Endpoint / KvCache /
  Capabilities / ToolCalling / Streaming / Compliance 全字段；
  **RateLimits 不映射**（layer2 路由决策不消费，VISION "只判断你消费的"）。
- 白名单结构校验，未知/错型字段 fail-closed（whitepaper §3.3 纵深防御）。

### D10: verifier 密钥注入——无全局可变状态（R-4）

- Python 用模块级 `PROTOCOL_ROOT_PUBLIC_KEY_BYTES` 常量 + 测试里 monkeypatch
  替换（全局可变）；rs 改为**显式注入**（`verify_registry(data, sig, key)`），
  确定性 + 无锁 + 可测。
- 协议根公钥仍是编译期信任锚点（whitepaper §3.1 协议规定硬编码），
  以 `PROTOCOL_ROOT_PUBLIC_KEY_BYTES` 常量默认值携带，由 sidecar 装配注入。
### D7: 独立中立——零 Helix 依赖

- 核心 crate 不引用任何 Helix 项目 crate；judge-points 契约是**文档层**对接
  （消费者消费 FlowModus 数据标准），非代码依赖。

## 3. 后果

**正面**：度量衡三件套（STE/偏移量/一致性测试）获得可编译、可分发、可验证的
确定性实现；独立中立承诺在代码层落地（零 Helix 依赖可被外部引用验证）；
铁律 0 与 whitepaper 的矛盾在 rs 中一次性收敛。
**负面**：Rust 生态无 Python 便利性，五层迁移需逐层行为对齐（测试向量驱动）；
proto 编译链（prost-build）首次构建较慢。
**风险**：行为等价迁移遗漏 Python 边角语义 → 对策：以 Python 测试为逐层对齐
基准，R-2 每层迁移即跑对照向量。

## 4. 里程碑映射

R-1 骨架+schema → R-2 五层 → R-3 三调用模式+冷却恢复 → R-4 控制面+遥测（✅ 76 测试）→
R-5 judge-points 打通 → R-6 文档链+全量验证+推送（✅ 全部完成，83 测试）。
