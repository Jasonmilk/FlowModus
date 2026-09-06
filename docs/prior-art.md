# FlowModus 防御性公开 / 先有技术记录（Prior Art）

- **状态**：公开披露记录
- **公开日期**：2026-09-06（以本仓库 git 历史时间戳为准，所有 commit 在 GitHub 公开可见）
- **公开渠道**：GitHub 公开仓库 `Jasonmilk/FlowModus`（main + rs 双分支，Apache 2.0）
- **目的**：本记录 + 仓库完整 git 历史构成 2026-09-06 的**先有技术（prior art）**，
  用于防止任何第三方就下列创新点获得有效专利（尤其在中国"先申请制"下，
  申请日前公开的技术构成现有技术，可用于新颖性抗辩与无效宣告）。
- **性质声明**：Apache 2.0 允许自由使用与商用——本记录**不限制任何人使用**，
  仅冻结"谁先公开"的事实，防御抢注。

---

## 一、核心创新点清单（代码级证据）

> 证据路径 = 仓库内文件；每个创新点均可在公开 git 历史中溯源到首次提交。

| # | 创新点 | 证据路径（rs 分支） | 首次提交 |
|---|---|---|---|
| 1 | 五层不可变确定性管线（Normalize→Registry→Deviation→Cost→Filter→Score） | `src/layer1_normalize.rs` … `src/layer5_score.rs` | `809151e` |
| 2 | STE 标准 Token 等价估算（ascii/4 + non-ascii/1.5，floor，min 1） | `src/layer1_normalize.rs` | `809151e` |
| 3 | 熵路由确定性种子（seed = sha256(instance:request)，跨进程位级一致） | `src/layer5_score.rs` | `809151e` |
| 4 | 零探测寄生遥测（废除定时心跳，遥测只来自真实流量） | `src/telemetry.rs` | `50618ed` |
| 5 | 失败冷却恢复（首败 DEGRADED → 连续 N 败 TERMINAL → 成功复位；冷却期概率康复） | `src/health.rs` | `7e524b1` |
| 6 | 三调用模式（Manual 直连 / Group 优先级+确定性权重采样 / Auto 全五层） | `src/router.rs` | `7e524b1` |
| 7 | 签名前规范化（递归 key 排序 + Unicode NFC + 紧凑 JSON，字节确定） | `src/control_plane/canonicalizer.rs` | `50618ed` |
| 8 | Ed25519 M-of-N 多重签名验签（签名数组 + 本地阈值 for-loop） | `src/control_plane/verifier.rs` | `50618ed` |
| 9 | 外部 JSON → 类型安全 protobuf 白名单映射（fail-closed） | `src/control_plane/anti_corruption.rs` | `50618ed` |
| 10 | 判断点 Rules 后端（0 tokens 确定性判定：复杂度评估 + 意图分类） | `src/judge_points.rs` | `2609815` |

## 二、协议级创新（白皮书公开）

| 创新点 | 证据 | 公开日 |
|---|---|---|
| 度量衡范式（STE=度 / 声明偏移量=校准 / 一致性测试=检定） | `docs/whitepaper-v1.7.md` | 2026-09-06（v1.7.1 修订记录） |
| 五层不可变管线模型 + 铁律 0（零定时探测） | 同上 §5 | 同上 |
| "不判断，只呈现，判断权属于用户"的裁决层立场 | 同上 / `docs/VISION.md` | 同上 |

## 三、与 CI-144 协议家族的关系

FlowModus 是独立协议实现，可运行于任何 LLM 调度场景；其与 CommonIntents
CI-144 协议家族（INTENT-7 / CAPABILITY-13 / INTENT-7-SECURE / BIND-19 /
PFP-xCF14 / SAP-xCF14）的对接属生态集成，不构成本记录的创新点。
CI-144 家族自身的防御性公开另行维护（CommonIntents 组织，待用户拍板）。

## 四、后续保护动作（按需驱动，非仪式）

| 动作 | 必要性 | 时机 |
|---|---|---|
| 本记录 + git 历史 | 已完成（零成本） | — |
| NOTICE + Apache 2.0（双分支） | 已完成（零成本） | — |
| 提交第三方防御性公开平台（IP.com 等） | 可选（非必须；GitHub 公开已是先有技术证据，平台提交仅多一份第三方时间戳） | 有预算/有需要时 |
| 加入 OIN 等专利互保联盟 | 低优先级（对 AI 协议栈价值有限） | 生态做大后评估 |
| 核心创新点正式申请专利 | 与"协议公开"立场冲突（申请即要求闭锁），通常不适用于以采用率为目标的协议 | 仅在商业增值层（服务/工具）出现时评估 |

**立场**：保护策略同样遵循项目哲学——极致解耦（保护动作独立于开发流水线）、
按需驱动（按真实威胁排序，不为仪式做事）、物理事实优先（GitHub 公开历史
就是最硬的先有技术证据）、极致节能（零成本动作立即做，高成本动作挂议程）。
