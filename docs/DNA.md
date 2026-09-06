# FlowModus DNA——不可变原则

> **版本**：v1.0（rs 重构立宪）｜ 2026-09-06
> 违反本章任一条 = 返工。本章无例外，不可变（变更须 ADR 且全文会审）。

## 一、Helix 工程哲学（八条，最高位阶）

1. **极致解耦**——模块间只经 schema/契约通信，无隐式耦合；
2. **按需加载**——请求只带本轮所需；模块惰性求值，无请求零活动；
3. **按需驱动**——一切能力显式声明、按需启用，无默认开启的隐性行为；
4. **极致复用**——同一资产（schema/判据/时钟/回退链）只存在一份；
5. **物理事实优先**——用真实代码/真实流量验证假设，不按文档照搬；
6. **确定性优先**——同输入 → 位级相同输出（instance-id 仅去相关）；
7. **0 硬编码**——每个字面量（阈值/模型名/循环上限/时间窗）必须有来源
   （config/契约/派生/协议默认）；
8. **极致节能**——0 tokens 优先 > 少 tokens/小 LLM > 多 tokens/大 LLM（默认通道
   不是教条，判断点按 ROI 显式选后端）。

## 二、FlowModus 铁律（继承 Python manual v1.1.3，重构不变量）

0. **最小作用量**——废除一切定时轮询与主动探测；遥测 100% 寄生于真实业务数据流；
1. **确定性优先**——同输入 → 位级相同输出，全局由 instance-id 去相关；
2. **纯函数**——路由逻辑无副作用（side-effect-free）；
3. **无锁**——消息传递优于共享内存，无死锁；
4. **无阻塞**——异步 I/O 唯一；控制面与数据面分离；
5. **Schema 强制**——模块间 protobuf 契约，禁止裸 dict/JSON；
6. **极简依赖**——无 web 框架、无 ORM、无 daemon、无 UI；
7. **命名即文档**——Google 风格，自解释命名（Helix：零行业词汇，只用
   tool/args/expect/numbers/rate/text/fixture/mock/schema 族）。

## 三、不可变资产（rs 重构继承，不推翻）

| 资产 | 不变约束 |
|---|---|
| 五层流水线 | Normalization → Registry → Cost → Filter → Entropy，顺序不可变、不可跳过 |
| protobuf schema | routing/metrics/gossip/supplier/registry 契约不可变（扩展走新字段，不破坏） |
| 铁律 0 探测禁令 | grep 核验：无 heartbeat/probe/poll 字面量（除注释声明拒绝） |
| 凭证零接触 | Key 锁 Tuck，FlowModus 只见调度权（identity_labels 语义） |
| 三调用模式 | Manual（直连指定模型）/ Group（组内权重）/ Auto（全五层）语义不变 |

## 四、显式边界（不是 FlowModus 的事）

- 不持有凭证（Tuck）；不做权限分组（CAPABILITY-13）；不做 UI/展示（Cellrix 驾驶舱）；
- 不做在线学习路由（M3 边界，VISION 未来开放项）；不做守护进程。

## 五、方法论约束（phyt-DNA）

- 文档先于代码：VISION → DNA → RNA → PLAN → GROWTH → ADR；
- 里程碑后更新 README + 维护 ADR + 推 GitHub；
- 文档更新顺序：先子后父（ADR → PLAN → GROWTH → README → ECOSYSTEM → HANDOFF）；
- 决策先于代码：ADR 先行，代码落地。

---

**本文件是 FlowModus 的宪法。变更须 ADR + 全文会审，否则视为违反。**
