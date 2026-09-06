//! Control-plane end-to-end: canonicalize -> Ed25519 sign -> verify ->
//! tamper rejected. Proves the deterministic-signing loop byte-for-byte.

use ed25519_dalek::{SecretKey, Signer, SigningKey};
use flowmodus::control_plane::anti_corruption::parse_registry_package;
use flowmodus::control_plane::canonicalizer::canonicalize_json;
use flowmodus::control_plane::verifier::{verify_multisig, verify_registry};

fn signing_key(salt: u8) -> SigningKey {
    let mut seed = [0u8; 32];
    for (i, b) in seed.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(31).wrapping_add(salt.wrapping_mul(7));
    }
    SigningKey::from_bytes(&SecretKey::from(seed))
}

const REGISTRY_JSON: &str = r#"{
    "version": "v2.0-alpha",
    "published_at_unix": 1757000000,
    "suppliers": [
        {
            "supplier_id": "s1",
            "supplier_name": "alpha",
            "verified": true,
            "models": [
                {"model_id": "s1-fast", "agent_roles": ["code-generation"],
                 "billing": {"token_input": 0.1, "token_output": 0.4},
                 "capabilities": {"context_window": 65536},
                 "kv_cache": {"supported": true}}
            ],
            "endpoints": [{"base_url": "https://s1.example/v1/"}]
        }
    ],
    "signatures": []
}"#;

#[test]
fn canonicalize_sign_verify_loop() {
    let canonical = canonicalize_json(REGISTRY_JSON).expect("canonicalize");
    let key = signing_key(9);
    let pub_bytes = key.verifying_key().to_bytes();
    let sig = key.sign(&canonical).to_bytes();

    assert!(verify_registry(&canonical, &sig, &pub_bytes));
    // same canonical bytes every time -> signature stays valid across runs
    let canonical_again = canonicalize_json(REGISTRY_JSON).expect("canonicalize again");
    assert_eq!(canonical, canonical_again);
}

#[test]
fn tampered_registry_rejected() {
    let canonical = canonicalize_json(REGISTRY_JSON).expect("canonicalize");
    let key = signing_key(9);
    let pub_bytes = key.verifying_key().to_bytes();
    let sig = key.sign(&canonical).to_bytes();

    let tampered = canonicalize_json(
        REGISTRY_JSON.replace("\"verified\": true", "\"verified\": false").as_str(),
    )
    .expect("tampered canonicalize");
    assert_ne!(canonical, tampered);
    assert!(!verify_registry(&tampered, &sig, &pub_bytes));
}

#[test]
fn multisig_end_to_end() {
    let canonical = canonicalize_json(REGISTRY_JSON).expect("canonicalize");
    let keys: Vec<SigningKey> = (0..3).map(|i| signing_key(i + 1)).collect();
    let council: Vec<Vec<u8>> = keys
        .iter()
        .map(|k| k.verifying_key().to_bytes().to_vec())
        .collect();
    let sigs: Vec<Vec<u8>> = keys[..2]
        .iter()
        .map(|k| k.sign(&canonical).to_bytes().to_vec())
        .collect();
    assert!(verify_multisig(&canonical, &sigs, &council, 2));
    assert!(!verify_multisig(&canonical, &sigs, &council, 3));
}

#[test]
fn parsed_package_round_trips() {
    // anti-corruption accepts the same document the verifier signs
    let pkg = parse_registry_package(REGISTRY_JSON).expect("parse");
    assert_eq!(pkg.suppliers.len(), 1);
    assert_eq!(pkg.suppliers[0].models[0].model_id, "s1-fast");
}
