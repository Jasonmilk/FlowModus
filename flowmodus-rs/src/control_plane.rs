//! Control plane: canonicalizer, verifier, anti-corruption.
//!
//! - Canonicalizer: deterministic byte normalization BEFORE signing
//!   (whitepaper §2.3): UTF-8 no BOM, recursively key-sorted compact JSON,
//!   Unicode NFC. Byte-identical on every platform/process.
//! - Verifier: Ed25519 signature verification — single root key and
//!   M-of-N multisig (whitepaper §7.4: signature array + local threshold,
//!   a for loop, no exotic cryptography).
//! - Anti-corruption: external JSON -> type-safe protobuf messages
//!   (whitepaper §3.3 defense-in-depth: whitelist structure check).

pub mod anti_corruption;
pub mod canonicalizer;
pub mod verifier;
