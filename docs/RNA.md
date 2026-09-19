# FlowModus RNA——方法论加载协议
> © 2026 Jason Milk · Apache 2.0

> **版本**：v1.0 ｜ 2026-09-06
> RNA 是 DNA 的可执行副本：规定"如何生长"，不改变"是什么"。

## 一、phyt-DNA 方法论（项目像植物一样生长）

- **文档五件套流转**：`VISION.md`（为什么存在）→ `DNA.md`（不可变原则）→
  `PLAN.md`（导航牌，≤150 行）→ `GROWTH.md`（生长记录，超 3 条归档最旧）→
  `ADR-XXXX`（决策先于代码）；
- **生长节奏**：一个里程碑 = 一个 ADR + PLAN 更新 + GROWTH 记录 + README 同步；
- **文档顺序**：先子后父——ADR → PLAN → GROWTH → README → 上级导航
  （Helix-Mind ECOSYSTEM.md）→ HANDOFF；
- **全部成果推 GitHub**；README 必须可用（命令可记、可跑）。

## 二、rs 重构的加载顺序（当前会话）

```
VISION.md（已立）→ DNA.md（已立）→ PLAN.md（导航牌）→ GROWTH.md（记录 0 初始化）
→ ADR-0100（rs 重构立项，决策先于代码）→ 代码按 PLAN 生长 → 每阶段验证
```

## 三、验证门（每个里程碑必过）

1. `cargo test` 全绿（迁移对齐：Python 53 测试语义 → Rust 等价覆盖）；
2. 铁律核验：grep 无探测字面量、无硬编码残留（阈值/模型名有 config 来源）；
3. 五层流水线行为等价（同输入同输出，测试向量对齐）；
4. 文档链同步（PLAN/GROWTH/README/ADR 一个不落）；
5. 推 GitHub（rs 分支）。

## 四、健康检查（GROWTH 记录内容）

每条记录含：健康快照（✅/🚧/❌ + 测试数）、物理事实（探查/真伤）、
验证结果（怎么验的）、状态（完成/阻塞+原因）。

---

**RNA 只加载方法，不改变宪法。DNA 是唯一真相。**
