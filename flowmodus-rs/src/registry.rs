//! Registry store — free/paid physical separation (度量衡注册台).
//!
//! One SupplierDeclaration per file, under `registry/free/` and
//! `registry/paid/`. Tier is a *physical fact*, not a label: the free tier
//! is enforced by checking that every model's billing is zero (0 硬编码,
//! 极致解耦 — adding/removing one supplier never touches another file).

use crate::pb::SupplierDeclaration;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Registry tier — directory-level separation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Free,
    Paid,
}

impl Tier {
    pub fn dir(self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Paid => "paid",
        }
    }
    pub fn as_str(self) -> &'static str {
        self.dir()
    }
}

/// Physical truth: a supplier is free iff every model bills zero.
pub fn is_free_supplier(decl: &SupplierDeclaration) -> bool {
    decl.models.iter().all(|m| {
        m.billing
            .as_ref()
            .map(|b| b.token_input == 0.0 && b.token_output == 0.0)
            .unwrap_or(true)
    })
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// File-based supplier registry.
pub struct RegistryStore {
    root: PathBuf,
}

impl RegistryStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default location relative to the crate root.
    pub fn at_root() -> Self {
        Self::new("registry")
    }

    fn tier_dir(&self, tier: Tier) -> PathBuf {
        self.root.join(tier.dir())
    }

    fn file_path(&self, tier: Tier, id: &str) -> PathBuf {
        self.tier_dir(tier).join(format!("{id}.json"))
    }

    /// Load every supplier of a tier (deterministic order by id).
    pub fn load_tier(&self, tier: Tier) -> Vec<SupplierDeclaration> {
        let dir = self.tier_dir(tier);
        let mut out = BTreeMap::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(decl) = serde_json::from_str::<SupplierDeclaration>(&text) {
                        out.insert(decl.supplier_id.clone(), decl);
                    }
                }
            }
        }
        out.into_values().collect()
    }

    /// Load both tiers.
    pub fn load_all(&self) -> (Vec<SupplierDeclaration>, Vec<SupplierDeclaration>) {
        (self.load_tier(Tier::Free), self.load_tier(Tier::Paid))
    }

    pub fn get(&self, tier: Tier, id: &str) -> Option<SupplierDeclaration> {
        let path = self.file_path(tier, id);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
    }

    /// Add (or replace) a supplier in a tier, with tier enforcement.
    pub fn add(&self, tier: Tier, decl: &SupplierDeclaration) -> Result<(), String> {
        if !valid_id(&decl.supplier_id) {
            return Err(format!(
                "invalid supplier_id {:?} (alnum/-/_ only, <=64)",
                decl.supplier_id
            ));
        }
        // Physical truth enforcement: free tier requires zero billing.
        if tier == Tier::Free && !is_free_supplier(decl) {
            return Err(format!(
                "tier=free rejected: {} has non-zero billing (physical fact, 0 硬编码)",
                decl.supplier_id
            ));
        }
        // Cross-tier id collision: one id lives in exactly one tier.
        let other = match tier {
            Tier::Free => Tier::Paid,
            Tier::Paid => Tier::Free,
        };
        if self.get(other, &decl.supplier_id).is_some() {
            return Err(format!(
                "supplier_id {} already registered in tier={}",
                decl.supplier_id,
                other.as_str()
            ));
        }
        let dir = self.tier_dir(tier);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("create {}: {e}", dir.display()))?;
        let text = serde_json::to_string_pretty(decl)
            .map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(self.file_path(tier, &decl.supplier_id), text)
            .map_err(|e| format!("write: {e}"))?;
        Ok(())
    }

    pub fn remove(&self, tier: Tier, id: &str) -> Result<(), String> {
        let path = self.file_path(tier, id);
        if !path.exists() {
            return Err(format!("not found: tier={} id={id}", tier.as_str()));
        }
        std::fs::remove_file(&path).map_err(|e| format!("remove: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::{BillingDeclaration, ModelDeclaration};

    /// Minimal std-only temp dir (最小依赖铁律 — no tempfile crate).
    struct TmpDir(std::path::PathBuf);
    impl TmpDir {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "flowmodus-reg-test-{}-{:?}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&p).unwrap();
            TmpDir(p)
        }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn free_decl(id: &str) -> SupplierDeclaration {
        SupplierDeclaration {
            supplier_id: id.into(),
            supplier_name: id.into(),
            verified: false,
            updated_at_unix: 0,
            models: vec![ModelDeclaration {
                model_id: "m1".into(),
                display_name: "M1".into(),
                lang: "en".into(),
                semantic_tags: vec![],
                agent_roles: vec![],
                billing: Some(BillingDeclaration::default()),
                kv_cache: None,
                capabilities: None,
                tokenizer_compression_ratio: 0.0,
                capability_tags: vec![],
            }],
            endpoints: vec![],
            compliance: None,
            rate_limits: None,
        }
    }

    fn paid_decl(id: &str) -> SupplierDeclaration {
        let mut d = free_decl(id);
        if let Some(b) = d.models[0].billing.as_mut() {
            b.token_input = 0.5;
        }
        d
    }

    #[test]
    fn free_tier_rejects_nonzero_billing() {
        let dir = TmpDir::new();
        let store = RegistryStore::new(&dir.0);
        let err = store.add(Tier::Free, &paid_decl("paid-in-free")).unwrap_err();
        assert!(err.contains("non-zero billing"), "{err}");
    }

    #[test]
    fn free_tier_accepts_zero_billing() {
        let dir = TmpDir::new();
        let store = RegistryStore::new(&dir.0);
        store.add(Tier::Free, &free_decl("groq")).unwrap();
        assert_eq!(store.load_tier(Tier::Free).len(), 1);
        assert!(store.load_tier(Tier::Paid).is_empty());
    }

    #[test]
    fn cross_tier_id_collision_rejected() {
        let dir = TmpDir::new();
        let store = RegistryStore::new(&dir.0);
        store.add(Tier::Free, &free_decl("dup")).unwrap();
        let err = store.add(Tier::Paid, &paid_decl("dup")).unwrap_err();
        assert!(err.contains("already registered"), "{err}");
    }

    #[test]
    fn add_remove_roundtrip() {
        let dir = TmpDir::new();
        let store = RegistryStore::new(&dir.0);
        store.add(Tier::Paid, &paid_decl("volc")).unwrap();
        assert!(store.get(Tier::Paid, "volc").is_some());
        store.remove(Tier::Paid, "volc").unwrap();
        assert!(store.get(Tier::Paid, "volc").is_none());
    }
}
