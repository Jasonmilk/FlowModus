# FlowModus

**The Deterministic Protocol Layer for LLM API Scheduling.**  
*Decentralized. Vendor-Neutral. Community-Driven.*

---

### ⚡ TL;DR: What is FlowModus?
FlowModus is a **cryptographically‑verified, deterministic routing layer** between your Agents and LLM Providers.
Core pipeline: STE Normalization → Crypto‑Registry → Cost Inference → Hard Filters → Entropy Routing

- **Problem**: Legacy gateways route tokens based on vendor profit, leading to opaque billing, non‑deterministic latency, and massive compute waste.
- **Solution**: Local‑first sidecar executing fixed 5‑layer deterministic pipeline.
- **Result**: Verifiable per‑token cost attribution, zero‑waste execution, full user sovereignty over compute.

[Read the Abstract] | [Quick Start] | [Whitepaper v1.5]

---

> "Dispatch is the ultimate sovereignty in the era of commodity compute." 
> — From the FlowModus Whitepaper v1.5
> *Technical Essence: Deterministic control over your compute, not just opaque access.*

## 🛠 The Fundamental Problem (From Whitepaper Chapter 1)
The LLM API ecosystem has reached a critical inflection point. As compute evolves into a global, fungible commodity, the infrastructure connecting users to models remains trapped in a legacy paradigm designed for centralized cloud services.

Current "best‑effort" gateways suffer from three irredeemable flaws rooted in their architectural design:
1. **Non‑determinism**: No guarantees on latency, cost, or data residency. Requests are routed based on vendor profit motives, not user preferences.
2. **Opacity**: Users have no visibility into how their tokens are processed, where their data goes, or why they are being charged a certain price.
3. **Waste**: Redundant polling, unnecessary data processing, and inefficient routing algorithms result in billions of tokens worth of computational energy wasted every day. Inefficient routing isn't just a cost issue; it's a sustainability crisis for AI scale.

These flaws are not bugs—they are features of a system designed to extract rent from users rather than empower them.

## 🎯 Our Mission
FlowModus was not built to be "just another gateway." It emerged from a fundamental necessity during the development of [Anaphase‑Helix](https://github.com/Jasonmilk): **The need for an infrastructure that adheres to the laws of physics and information theory, rather than the arbitrary policies of vendors.**

We are replacing the legacy rent‑extraction model with a **Deterministic Protocol** that returns sovereignty over compute to the users. This is not an incremental improvement—it is a complete reimagining of how AI compute should be distributed.

## 🌌 The 5‑Layer Deterministic Pipeline (Whitepaper Chapter 3)
To eliminate non‑determinism at every level, the FlowModus Sidecar processes every request through a mathematically rigorous fixed sequence. No step is optional, no decision is arbitrary.

1.  **Normalization (STE)**: Converts all vendor‑specific token formats to Standard Token Equivalents for fair cross‑model comparison.
2.  **Raw Registry**: Ed25519‑signed, immutable supplier definitions pulled via IPFS/IPNS, no central registry.
3.  **Cost Inference**: Real‑time pricing calculation factoring in Helix‑Callosum cache hints, latency and reliability.
4.  **Hard Filter**: Enforces user‑defined constraints (Budget, Region, Data Residency, Logic Integrity) locally before outbound requests.
5.  **Entropy‑Weighted Routing**: Deterministic jitter injection to avoid global traffic herd effects under fixed latency bounds.

## 🧬 Engineering Principles (Iron Laws from Whitepaper Chapter 2)
These principles are non‑negotiable. Any change violating them will be rejected.

-   **Minimal Action**: No active polling. No unnecessary computation. All telemetry is parasitic and event‑driven. Design toward theoretical computational efficiency limits.
-   **Security by Math**: Every routing table is cryptographically verified. No single point of failure, no centralized authority. Verification over trust.
-   **Local Sovereignty**: API keys and prompt metadata never leave your local instance by default. You own your data.
-   **Zero Waste Philosophy**: Continuously reduce overhead toward physical limits; target <1% overhead vs direct API calls.

## 📚 Core Architecture (Whitepaper Chapter 4)
*Sovereignty here refers exclusively to the user's unalienable, deterministic control over four core dimensions: cost attribution, data residency, routing logic, and failure domain management. It has no political connotations whatsoever.*

FlowModus adopts a sidecar architecture for maximum compatibility and minimal intrusion. Full component‑level architecture diagram will be released alongside v1.0 stable.

## 🆚 How We Compare to Legacy Gateways
| Feature | FlowModus | Legacy Gateways |
|---------|-----------|-----------------|
| Routing Logic | Deterministic, user‑defined | Opaque, vendor‑controlled |
| Cost Attribution | Verifiable per‑token | Aggregate, approximate |
| Data Residency | Edge‑enforceable | Unreliable |
| Target Overhead | <1% | 5‑20% |
| Centralized Points | None | Multiple |
| Sustainability Focus | Native design | None |

## 🚀 Roadmap (Whitepaper Chapter 6)
- **Q2 2026**: v0.1 Alpha Release (Core Pipeline, Basic Routing)
- **Q3 2026**: v0.5 Beta Release (Helix‑Callosum Integration, Advanced Filters)
- **Q4 2026**: v1.0 Stable Release (Full Protocol Spec, Independent Security Audit)
- **2027**: Decentralized Governance, Multi‑Cluster Support, Global Registry v2

## 🤝 Contributing
FlowModus is community‑driven. Contributions must align with core engineering principles and long‑term protocol roadmap. Please read [Contributing Guidelines](CONTRIBUTING.md) before submitting.

## 📄 Licenses
FlowModus is multi‑licensed:
- **Core Protocol & Sidecar Base**: MIT License — permanent public utility.
- **Advanced Enterprise Extensions**: Apache License 2.0 — legal & patent protection for enterprise deployments.

FlowModus Whitepaper licensed under CC BY‑ND 4.0 to preserve authoritative protocol specification.
See [LICENSE](LICENSE) for full terms.

## ⚠️ Disclaimer
Pre‑release software, provided "as‑is" without warranty. Protocol subject to change until v1.0 stable.

---

*Built with mathematical rigor by the Anaphase‑Helix research team.*

[FlowModus Whitepaper v1.5](link‑to‑whitepaper.pdf) | [GitHub Organization](https://github.com/flowmodus) | [Discussions](link‑to‑discussions)
