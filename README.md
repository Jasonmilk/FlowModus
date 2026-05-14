# FlowModus

**The Deterministic Protocol Layer for LLM API Scheduling**  
*Decentralized. Vendor‑Neutral. Community‑Driven.*

---

### ⚡ TL;DR

FlowModus is a **cryptographically‑verified, deterministic routing layer** between your Agents and LLM Providers. It replaces opaque, rent‑seeking gateways with a local‑first sidecar executing a mathematically rigorous 5‑layer pipeline.

*   **Problem**: Legacy gateways route based on vendor profit, not user intent. Opaque billing, non‑deterministic latency, and compute waste.
*   **Solution**: A zero‑waste sidecar that enforces user‑defined constraints, verifies every routing table via Ed25519, and derives all telemetry from real traffic — never active probing.
*   **Result**: Verifiable per‑token cost attribution, full data sovereignty, and routing that obeys *your* rules, not the vendor's.

📖 [Whitepaper v1.7](docs/whitepaper-v1.7.md) · 🛠 [Engineering Manual v1.1.3](docs/engineering-manual/v1.1.3.md)

---

## 🧬 The 5‑Layer Deterministic Pipeline

Every request passes through five immutable layers. No step is optional, no decision is arbitrary.

1.  **Normalization (STE)** – Converts vendor‑specific tokens to Standard Token Equivalents.
2.  **Raw Registry** – Ed25519‑signed, IPFS/IPNS‑distributed supplier declarations.
3.  **Cost Inference** – Real‑time price estimation factoring cache hints and claim deviations.
4.  **Hard Filter** – Enforces budget, region, residency, and user‑defined bias locally.
5.  **Entropy‑Weighted Routing** – Deterministic jitter via instance‑ID to prevent global herd effects.

---

## 🪨 Engineering Iron Laws (from the Manual)

*These are non‑negotiable. Code violating any of them will not be merged.*

0.  **Principle of Least Action** – Zero active polling. All telemetry is parasitic. No waste.
1.  **Determinism First** – Same input → bit‑identical output, globally decorrelated by instance‑ID.
2.  **Pure Functions** – All routing logic is side‑effect‑free.
3.  **Lock‑Free** – Message passing over shared memory. No deadlocks.
4.  **No Blocking** – Async I/O only. Control plane separated from data plane.
5.  **Schema Enforcement** – Protobuf between modules. Never raw `dict` or JSON.
6.  **Minimal Dependencies** – No web frameworks, no ORMs, no daemons.
7.  **Naming as Documentation** – Google style. Self‑explanatory names.

---

## 🏗 Architecture

FlowModus runs as a **local sidecar** (`localhost:8080`). It never sees your API keys — they are injected at the edge and never leave memory. Supplier registries are pulled from IPFS and verified with the Protocol Root Key. The system decays gracefully: if the maintainer disappears, the registry freezes and the community can fork.

*Control Plane / Data Plane separation ensures that cryptographic verification never competes with your request latency.*

---

## 🚀 Roadmap (from the Manual)

| Phase | Timeline | Focus |
|:---|:---|:---|
| **Phase 1** | 0–3 months | Personal Sidecar MVP, core 5‑layer pipeline, 1‑Token probe prototype |
| **Phase 2** | 3–6 months | Community ecosystem, IPFS/IPNS distribution, Gossip health network |
| **Phase 3** | 6–12 months | Standard solidification, donation to LF AI & Data / CNCF sandbox |
| **Phase 4** | 12+ months | Ubiquitous infrastructure, edge‑cloud unified scheduling |

---

## 📄 Licenses

*   **Core Protocol & Sidecar Base**: MIT — permanent public utility.
*   **Enterprise Extensions**: Apache 2.0 — legal & patent protection for enterprise.
*   **Whitepaper**: CC BY‑ND 4.0 — authoritative protocol specification.

---

## ⚠️ Disclaimer

Pre‑release software. Provided "as‑is" without warranty. Protocol subject to change until v1.0 stable. FlowModus is a neutral technical standard and does not constitute investment advice or service guarantees.

---

*Built with mathematical rigor by the FlowModus Community.*
