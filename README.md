# FlowModus

**The Weights & Measures Bureau for LLM API Scheduling** · The Deterministic Protocol Layer for LLM API Scheduling

[Helix ecosystem](https://github.com/Jasonmilk) · [CommonIntents protocol family](https://github.com/CommonIntents) · [CI-144 semantics INTENT-7](https://github.com/CommonIntents/INTENT-7) · [CI-144 capability CAPABILITY-13](https://github.com/CommonIntents/CAPABILITY-13) · [CI-144 security INTENT-7-SECURE](https://github.com/CommonIntents/INTENT-7-SECURE) · [CI-144 transport BIND-19](https://github.com/CommonIntents/BIND-19)

**License**: Apache 2.0 (code + whitepaper, both branches in sync) | **Positioning**: independent protocol, zero Helix dependency, consumable by any agent framework

> **中文版 (Chinese Version)**: [README.zh-CN.md](./README.zh-CN.md)

---

## What It Is

FlowModus is not a gateway, not a proxy, not an admin console. It is a set of **weights & measures** — a deterministic metering standard and adjudication presentation layer for LLM API scheduling:

| Triad of measures | Implementation |
|---|---|
| **Measure (STE)** | Standard Token Equivalent — folds every vendor's token/billing conventions onto a single ruler |
| **Calibration (declared deviation)** | deviation metric between vendor declarations and real telemetry |
| **Verification (consistency testing)** | five-layer deterministic pipeline, same input → same output, byte-identical across processes |

**Core stance (VISION v2.0-rs)**: **it does not judge, it presents; the right to judge belongs to the user.**
The pipeline presents metering, cost, filtering and scoring all as deterministic data; choices are made by the user/orchestration layer.

## Ironclad Rules

- **Ironclad rule 0 (zero probing)**: no periodic heartbeats or active probing. Telemetry is 100% parasitic on real traffic; the only exception = explicit user on-demand probing.
- **Five immutable layers**: Normalization(STE) → Registry(Ed25519 signatures) → Deviation → Cost(normalized billing) → Filter(hard boundaries) → Score(soft weighting + entropy routing).
- **Determinism first**: entropy routing seed = sha256(instance:request), bit-identical across processes for the same input.
- **Zero hardcoding**: every default has a source (whitepaper / Python dataclass / one-api semantics / protocol root public key), user-overridable.

## Branches

| Branch | Content | Status |
|---|---|---|
| **rs** | Rust rebuild (R-1..R-6 all complete) | ✅ primary, 83 tests green, clippy zero warnings |
| **main** | Python v1.7 (original whitepaper implementation) | 🧊 frozen maintenance |

The rs branch is the deterministic rebuild: three call modes truly landed (Python's Group was a placeholder stub), failure-cooldown recovery completed (Python's health had consumers but no producer), canonicalizer fills the Unicode NFC protocol gap (Python only sorted keys).

## Run

```bash
git clone https://github.com/Jasonmilk/FlowModus.git
cd FlowModus/flowmodus-rs

cargo test        # 87 passed (five layers + three call modes + control plane + judge-points + registry)
cargo run -- judge "帮我探索这个主题"            # JP-1/JP-2 Rules adjudication (0 tokens)
cargo run -- measure "hello world 你好世界"      # STE (measure)
cargo run -- verify '{"version":"v2.0-alpha","suppliers":[]}'  # normalization + sha256 (verification)
```

CLI commands: `judge` (judge-points Rules adjudication) / `measure` (STE) / `verify` (signature normalization) / `supplier` (registry management) / `route` (deterministic decision).
Parsing uses std::env, no clap — minimal dependencies.

## Supplier Registry (度量衡注册台)

Free and paid suppliers are **physically separated** — one file per supplier:

```
registry/
  free/<supplier_id>.json     # billing must be all-zero (enforced, physical fact)
  paid/<supplier_id>.json     # any billing
```

```bash
cargo run -- supplier add --tier free --id groq --base-url https://api.groq.com/openai/v1 \
    --auth bearer_token --model llama-3.3-70b          # free tier rejects non-zero billing
cargo run -- supplier list [--tier free|paid]
cargo run -- supplier get --tier paid --id volc
cargo run -- supplier test --tier free --id groq       # on-demand probe (0 idle probes)
cargo run -- supplier rm --tier free --id groq
cargo run -- route --model auto --prompt "hello"       # free-first soft priority
```

- Registered JSON files are **user-local** (gitignored — endpoints/keys stay out of the repo).
- `route --model auto` applies the free-tier soft bonus in Layer 5 (0-token-first
  survival philosophy); paid suppliers remain the fallback. Judgement stays with
  the user — `free_tier_bonus` in `BiasConfig` is overridable.
- Call modes: `auto` (five-layer pipeline) / `group:NAME` / explicit `MODEL_ID`.

## Five-Layer Deterministic Pipeline

| Layer | Duty | Determinism guarantee |
|---|---|---|
| L1 Normalization | STE estimate + prompt sha256 | ascii/4 + non-ascii/1.5, floor, min 1 |
| L2 Registry | Ed25519-signed vendor declarations | signature verification + structure validation (anti-corruption fail-closed) |
| L2.5 Deviation | declared vs actual offset | settlement weighting (weight=i+1) |
| L3 Cost | normalized billing | billing + kv savings ×0.9 + context_window gate |
| L4 Filter | user hard boundaries | budget/deviation tolerance/vendor bias caps + priority cascade + recovery adjudication |
| L5 Score | soft weighting + entropy routing | softmax + sha256-derived seed (bit-identical across processes) |

## Three Call Modes

- **Manual**: direct connection to a specified model, zero pipeline overhead (`model = "s1-fast"`)
- **Group**: user routing group, priority descending + deterministic weighted sampling within equal priority (`model = "group:fast-lane"`)
- **Auto**: full five-layer pipeline (`model = "auto"` or empty)

## Control Plane & Telemetry

- **canonicalizer**: recursive key sorting + Unicode NFC + compact JSON → byte-deterministic before signing
- **verifier**: Ed25519 single key + M-of-N multisig (key injection, no global mutable state)
- **anti-corruption**: external JSON → type-safe proto, whitelist fail-closed
- **health**: parasitic failure cooldown (first failure DEGRADED → 5 consecutive TERMINAL → success reset), one-api semantics
- **telemetry**: real-traffic sampling + derived aggregation (hit_rate / health_counts)

## Relationship to the Helix Ecosystem

- **Independent and neutral**: core crate has zero Helix dependency, externally referencable (DNA rule 7)
- **Judge-points contract**: `docs/engineering-manual/judge-points-contract.md` (v1.1) —
  FlowModus provides the JP-1/JP-2 Rules backend (0-token deterministic adjudication);
  Anaphase side O-6 (ADR-0024) consumer ready, SmallLlm backend swappable at consumer's
  choice, illegal output always falls back to Rules (fail-safe)
- **Orchestration boundary**: parallel pool / context-window awareness belong to FlowModus; model choice rests with FlowModus, credentials locked in Tuck, data sovereignty stays local

## Document Chain (phyt-DNA methodology)

| Document | Content | Version |
|---|---|---|
| [VISION.md](docs/VISION.md) | Weights & Measures manifesto | v2.0-rs |
| [DNA.md](docs/DNA.md) | immutable principles (ironclad rules) | v1.1 |
| [RNA.md](docs/RNA.md) | methodology loading protocol | — |
| [PLAN.md](docs/PLAN.md) | development navigation board | R-0..R-6 all ✅ |
| [GROWTH.md](docs/GROWTH.md) | growth records | records 0-7 |
| [ADR-0100](docs/decisions/ADR-0100-rs-refactor.md) | rs rebuild decision record | D1-D10 |
| [judge-points-contract](docs/engineering-manual/judge-points-contract.md) | judge-points contract | v1.1 |
| [whitepaper-v1.7](docs/whitepaper-v1.7.md) | protocol whitepaper | v1.7.1 |
| [prior-art](docs/prior-art.md) | defensive publication / prior art record | 2026-09-06 |

## License & Protection

- **Apache 2.0**: code + whitepaper (both branches in sync, v1.7.1 change record traceable)
- **NOTICE**: Apache attribution statement (see repository root)
- **Defensive publication**: core innovations publicly disclosed via `docs/prior-art.md` +
  GitHub public git history as of 2026-09-06 (prior art), preventing third-party patent scooping
- Reference inspiration (not borrowing): mature third-party API management projects (display & API management forms) —
  explicitly rejects gateway / proxy / admin-console forms; no wheel reinvention

---

*Built with mathematical rigor by the FlowModus Community.*

## 按需状态端点（检定台数据源）

```bash
flowmodus serve --port 60053   # GET /api/status → 双 tier 池 + route auto 决策
```
std HTTP/1.1 零新依赖；不 serve 不监听（按需加载）；不暴露任何 key。
