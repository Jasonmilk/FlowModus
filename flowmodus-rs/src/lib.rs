//! FlowModus rs — the weights-and-measures bureau for LLM API scheduling.
//!
//! Philosophy (see docs/VISION.md, docs/DNA.md):
//! - FlowModus does NOT judge; it only presents. Judgment belongs to the user.
//! - Five immutable pipeline layers (whitepaper v1.7, DNA v1.1):
//!   L1 measure (STE) -> L2 signed registry -> L2.5 deviation ->
//!   L3 standardized cost -> L4 hard filter -> L5 soft weighting + entropy routing.
//! - Zero timing probes (manual iron law 0): telemetry is parasitic on real
//!   traffic; the only exception is user-explicit on-demand probing.
//! - Zero Helix dependency: this crate is an independent protocol, consumable
//!   by any agent framework.

/// Compiled protobuf contracts (immutable schema boundary).
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/flowmodus.rs"));
}

/// Layer 1: protocol & measure specification (STE standard token equivalent).
pub mod layer1_normalize;

/// Layer 2: supplier raw model registry (Ed25519-signed declarations).
pub mod layer2_registry;

/// Layer 2.5: deterministic baseline — declaration deviation (parasitic
/// telemetry + user-explicit on-demand probing; no timing heartbeat).
pub mod layer2_5_deviation;

/// Layer 3: standardized costing (Estimated_Cost_USD collapse).
pub mod layer3_cost;

/// Layer 4: user preferences & hard boundaries (hard filter).
pub mod layer4_filter;

/// Layer 5: agent intent & dynamic preference (soft weighting + entropy routing).
pub mod layer5_score;

/// Control plane: canonicalizer, verifier, anti-corruption.
pub mod control_plane;

/// Telemetry: collector (parasitic) + deviation (declaration vs actual).
pub mod telemetry;

/// CLI entrypoint (sidecar).
pub mod cli;
