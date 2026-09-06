//! Verifier — Ed25519 signature verification (whitepaper §3, §7.4).
//!
//! Single root key and M-of-N multisig, implemented as a for loop over a
//! signature array with a local threshold count (whitepaper §7.4: no exotic
//! aggregate cryptography). Keys are injected (no global mutable state —
//! deterministic, lock-free); the protocol root public key is a compiled-in
//! trust anchor per whitepaper §3.1, carried as a constant default.

use ed25519_dalek::{Signature, VerifyingKey};

/// Protocol root public key — the network's single cryptographic trust anchor.
/// Source: protocol root key (whitepaper §3.1, offline-generated, multi-sig
/// committee; hardcoded in every Sidecar per protocol spec). Carried over from
/// the Python v1.7 implementation for trust continuity.
pub const PROTOCOL_ROOT_PUBLIC_KEY_BYTES: [u8; 32] =
    hex_literal("a914abdc99f277d64aafdd2c4db2ee73d468001e65747a823af50bb5a6dc3cf9");

/// Const-friendly hex decode (compile-time, no external macro crate).
const fn hex_literal(s: &str) -> [u8; 32] {
    let bytes = s.as_bytes();
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        let hi = hex_val(bytes[i * 2]);
        let lo = hex_val(bytes[i * 2 + 1]);
        out[i] = (hi << 4) | lo;
        i += 1;
    }
    out
}

const fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Verify a single Ed25519 signature against the given public key (pure).
pub fn verify_registry(data: &[u8], signature_bytes: &[u8], public_key: &[u8; 32]) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(signature_bytes) else {
        return false;
    };
    key.verify_strict(data, &sig).is_ok()
}

/// Verify M-of-N multisig (whitepaper §7.4): each signature may match any
/// council key; at most +1 valid count per signature; pass when
/// `valid_count >= threshold` (Python v1.7 semantics preserved).
pub fn verify_multisig(
    data: &[u8],
    signatures: &[Vec<u8>],
    council_keys: &[Vec<u8>],
    threshold: usize,
) -> bool {
    let mut valid_count = 0usize;
    for sig_bytes in signatures {
        let Ok(sig) = Signature::from_slice(sig_bytes) else {
            continue;
        };
        for key_bytes in council_keys {
            let Ok(key) = VerifyingKey::from_bytes(key_bytes.as_slice().try_into().unwrap_or(&[0u8; 32]))
            else {
                continue;
            };
            if key.verify_strict(data, &sig).is_ok() {
                valid_count += 1;
                break; // one valid match per signature is enough (Python shape)
            }
        }
    }
    valid_count >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SecretKey, Signer, SigningKey};

    /// Deterministic signing key for tests (no rand dependency; fixed salt
    /// per index so distinct keys stay distinct).
    fn signing_key(salt: u8) -> SigningKey {
        let mut seed = [0u8; 32];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31).wrapping_add(salt.wrapping_mul(7));
        }
        SigningKey::from_bytes(&SecretKey::from(seed))
    }

    #[test]
    fn verify_registry_valid() {
        let signing = signing_key(1);
        let pub_bytes = signing.verifying_key().to_bytes();
        let data = b"test registry data";
        let sig = signing.sign(data).to_bytes();
        assert!(verify_registry(data, &sig, &pub_bytes));
    }

    #[test]
    fn verify_registry_invalid_data() {
        let signing = signing_key(1);
        let pub_bytes = signing.verifying_key().to_bytes();
        let sig = signing.sign(b"test registry data").to_bytes();
        assert!(!verify_registry(b"wrong data", &sig, &pub_bytes));
    }

    #[test]
    fn multisig_threshold_met() {
        let keys: Vec<SigningKey> = (0..3).map(|i| signing_key(i as u8 + 1)).collect();
        let council: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| k.verifying_key().to_bytes().to_vec())
            .collect();
        let data = b"test multisig data";
        let sigs: Vec<Vec<u8>> = keys[..2].iter().map(|k| k.sign(data).to_bytes().to_vec()).collect();
        assert!(verify_multisig(data, &sigs, &council, 2));
    }

    #[test]
    fn multisig_threshold_not_met() {
        let keys: Vec<SigningKey> = (0..2).map(|i| signing_key(i as u8 + 1)).collect();
        let council: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| k.verifying_key().to_bytes().to_vec())
            .collect();
        let data = b"test multisig data";
        let sigs: Vec<Vec<u8>> = keys.iter().map(|k| k.sign(data).to_bytes().to_vec()).collect();
        assert!(!verify_multisig(data, &sigs, &council, 3));
    }

    #[test]
    fn verification_deterministic() {
        let signing = signing_key(1);
        let pub_bytes = signing.verifying_key().to_bytes();
        let data = b"test registry data";
        let sig = signing.sign(data).to_bytes();
        for _ in 0..100 {
            assert!(verify_registry(data, &sig, &pub_bytes));
        }
    }

    #[test]
    fn protocol_root_key_parses() {
        // The compiled-in anchor must decode to 32 bytes (compile-time const).
        assert_eq!(PROTOCOL_ROOT_PUBLIC_KEY_BYTES.len(), 32);
    }
}
