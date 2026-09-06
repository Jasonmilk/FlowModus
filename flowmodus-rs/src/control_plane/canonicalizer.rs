//! Canonicalizer — deterministic byte normalization for signing.
//!
//! Whitepaper §2.3: every supplier JSON declaration must be normalized before
//! signature verification so crypto signatures are byte-identical everywhere:
//! 1. UTF-8, no BOM (caller's byte source)
//! 2. recursively key-sorted compact JSON (no extra whitespace)
//! 3. Unicode NFC on every string value (visually-equal-but-differently-encoded
//!    characters collapse to identical byte streams)
//!
//! Note: Python v1.7 canonicalizer skipped NFC (only key-sort + compact);
//! rs completes the protocol requirement (gap closed, ADR-0100 D5).

use serde_json::{Map, Value};
use unicode_normalization::UnicodeNormalization;

fn nfc(s: &str) -> String {
    s.nfc().collect()
}

fn canonicalize_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for k in keys {
                let item = map.get(&k).expect("key present");
                sorted.insert(k, canonicalize_value(item.clone()));
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(canonicalize_value).collect()),
        Value::String(s) => Value::String(nfc(&s)),
        other => other,
    }
}

/// Canonicalize a JSON document to deterministic bytes (pure).
/// Returns an error when the input is not valid JSON.
pub fn canonicalize_json(input: &str) -> Result<Vec<u8>, String> {
    let parsed: Value = serde_json::from_str(input).map_err(|e| format!("invalid JSON: {e}"))?;
    let canonical = canonicalize_value(parsed);
    // serde_json's default compact serialization: no extra whitespace,
    // UTF-8 output (non-ASCII NOT escaped), matching Python's
    // `json.dumps(..., separators=(',', ':'), ensure_ascii=False)`.
    let out = serde_json::to_string(&canonical).map_err(|e| format!("serialize: {e}"))?;
    Ok(out.into_bytes())
}

/// NFC-normalize any UTF-8 text (protocol §2.3, exposed for headers/fields).
pub fn nfc_normalize(s: &str) -> String {
    nfc(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_sort_recursive_and_compact() {
        let input = r#"{"b":1,"a":{"d":2,"c":3},"arr":[{"y":1,"x":2}]}"#;
        let out = canonicalize_json(input).expect("canon");
        let s = String::from_utf8(out).expect("utf8");
        assert_eq!(s, r#"{"a":{"c":3,"d":2},"arr":[{"x":2,"y":1}],"b":1}"#);
        // no whitespace anywhere
        assert!(!s.contains(' '));
    }

    #[test]
    fn nfc_collapses_visually_equal() {
        // "é" as U+00E9 (NFC) vs "e" + U+0301 (NFD) — canonical bytes must match
        let nfc_form = "\u{00e9}";
        let nfd_form = "e\u{0301}";
        assert_ne!(nfc_form, nfd_form);
        assert_eq!(nfc(nfc_form), nfc(nfd_form));
        assert_eq!(nfc_normalize(nfd_form), nfc_form);
    }

    #[test]
    fn nfc_applies_to_string_values() {
        let nfd = "e\u{0301}";
        let input = format!(r#"{{"name":"{nfd}"}}"#);
        let out = canonicalize_json(&input).expect("canon");
        let s = String::from_utf8(out).expect("utf8");
        assert_eq!(s, r#"{"name":"é"}"#); // U+00E9
    }

    #[test]
    fn deterministic_same_input_same_bytes() {
        let input = r#"{"b":[3,1,2],"a":"x"}"#;
        let a = canonicalize_json(input).expect("a");
        let b = canonicalize_json(input).expect("b");
        assert_eq!(a, b);
    }

    #[test]
    fn invalid_json_errors() {
        assert!(canonicalize_json("{not json}").is_err());
    }
}
