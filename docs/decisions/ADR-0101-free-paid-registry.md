# ADR-0101: 供应商注册台——free/paid 物理分离 + 免费优先软权重

- **状态**: Accepted（用户拍板"免费付费需要分开"，2026-09-09）
- **日期**: 2026-09-09
- **决策范围**: FlowModus（flowmodus-rs，rs 分支）
- **关联**: VISION v2.0-rs（度量衡第一范式）/ DNA v1.1（0 硬编码、物理事实优先）/
  ADR-0100（rs 重构立项）/ Helix 生态生存哲学（0 tokens 优先 > 少 tokens/小 LLM >
  多 tokens/大 LLM）

## 1. 背景

FlowModus 需要支持用户提供更多 API、免费 API 池，并适配 Helix 生态。
用户明确：**免费付费分开管理**；API 管理参考**若干成熟的第三方 API 网关/管理台**的
*管理思路*（不重复造轮子——FlowModus 的价值在"度量衡"范式，不在又一个管理台）。

> **注（2026-09-20）**：此句原文**具名了两个外部项目**。按本仓「**活文档不出现外部项目名**」的规则，
> 名词已改为描述性表述，**决定内容一字未改**。旧名见提交历史（不回改历史）。

## 2. 决策

### D1: 注册台物理分离（目录即真相）

```
registry/
  free/<supplier_id>.json     # 计费必须全 0（强制）
  paid/<supplier_id>.json     # 任意计费
```

- 一供应商一文件（极致解耦：增删不触碰其他文件）。
- `supplier_id` 全局唯一（跨层冲突拒绝——一个 id 只能存在于一个层）。

### D2: 免费 = 物理事实，不是标签

- 免费判据：该供应商**所有模型**的 `token_input == 0 && token_output == 0`。
- `supplier add --tier free` 时强制校验，非零计费**拒绝写入**（防人标错，
  物理事实优先）。
- 本地模型（Ollama/Llama 经 Tuck base URL）天然全 0 → 落 free 层，
  承载"敏感话题本地执行"的安全需求。

### D3: 免费优先 = 用户默认偏好（软，不越度量衡边界）

- 在 L5 打分引入 `free_tier_bonus`（默认 +5.0，`BiasConfig` 可覆盖）。
- **软优先**：免费 + 健康 + 通过硬过滤时优先；付费兜底。
- 哲学澄清：这不是 FlowModus "推荐免费供应商"（那违反"不判断只呈现"），
  而是**用户默认偏好的呈现**——Helix 生存哲学的默认值，用户最终覆盖权
  （VISION 五·分界保留）。

### D4: 注册数据是用户私有

- `registry/{free,paid}/*.json` 进 `.gitignore`——端点与密钥偏好零入库。
- 目录结构（free/、paid/）随仓库提交（.gitkeep）。
- 凭证仍不持有：auth_method 只记录方式，密钥在 Tuck（唯一门）。

### D5: CLI（std::env，无 clap）

`supplier add|list|get|rm|test` + `route`。`test` 为**显式按需探测**
（唯一例外，铁律 0 的允许形态：用户主动调用才发探针，0 空闲心跳）。

## 3. 明确不做（一期）

- 不做 gRPC 服务端（FlowModus proto 的 Reason 服务、anaphase 接入、UI 检定台
  Model Nexus 式界面 → 二期，与 Helix 生态适配一起）。
- 不持有凭证、不代理流量（拓扑不变：Anaphase → FlowModus(决策) → Tuck(唯一门)）。

## 4. 后果

- 正向：免费池/付费池边界由物理事实保证，杜绝"标错免费"；用户本地配置不入库。
- 成本：二期接入前，决策接口为 CLI/库（无网络服务）。

## 5. 验证

- 单测：free 层拒非零计费、跨层 id 冲突、增删查往返（registry.rs 4 项）。
- 实测：free+paid 并存时 route auto 连续 3/3 选 free；仅 paid 时兜底；
  手动点名 MODEL_ID 直达。
- 全量：cargo test 87 passed；clippy --all-targets 0 warnings。
