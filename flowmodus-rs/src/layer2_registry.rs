//! Layer 2: supplier raw model registry — eligible-supplier lookup.
//!
//! Behavior aligned with Python v1.7 `Layer2Registry`: filter suppliers whose
//! models declare the request's agent role; if none match, fall back to all
//! (later layers decide). The registry itself is the Ed25519-signed verbatim
//! declaration store (whitepaper §5.3 L2); signature verification lands with
//! the control-plane verifier (R-4).

use crate::pb::{NormalizedRequest, SupplierDeclaration};

/// Filter suppliers by agent role (pure).
pub fn get_eligible_suppliers<'a>(
    registry: &'a [SupplierDeclaration],
    request: &NormalizedRequest,
) -> Vec<&'a SupplierDeclaration> {
    if request.agent_role.is_empty() {
        return registry.iter().collect();
    }
    let matched: Vec<&SupplierDeclaration> = registry
        .iter()
        .filter(|supplier| {
            supplier
                .models
                .iter()
                .any(|m| m.agent_roles.iter().any(|r| r == &request.agent_role))
        })
        .collect();
    if matched.is_empty() {
        registry.iter().collect()
    } else {
        matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::{CapabilitiesDeclaration, ModelDeclaration};

    fn model(id: &str, roles: &[&str]) -> ModelDeclaration {
        ModelDeclaration {
            model_id: id.into(),
            agent_roles: roles.iter().map(|s| s.to_string()).collect(),
            capabilities: Some(CapabilitiesDeclaration::default()),
            ..Default::default()
        }
    }

    fn supplier(id: &str, models: Vec<ModelDeclaration>) -> SupplierDeclaration {
        SupplierDeclaration {
            supplier_id: id.into(),
            models,
            ..Default::default()
        }
    }

    #[test]
    fn empty_role_returns_all() {
        let reg = vec![
            supplier("s1", vec![model("m1", &["code-generation"])]),
            supplier("s2", vec![model("m2", &["logical-analysis"])]),
        ];
        let req = crate::pb::NormalizedRequest {
            agent_role: String::new(),
            ..Default::default()
        };
        assert_eq!(get_eligible_suppliers(&reg, &req).len(), 2);
    }

    #[test]
    fn role_match_preferred() {
        let reg = vec![
            supplier("s1", vec![model("m1", &["code-generation"])]),
            supplier("s2", vec![model("m2", &["logical-analysis"])]),
        ];
        let req = crate::pb::NormalizedRequest {
            agent_role: "code-generation".into(),
            ..Default::default()
        };
        let out = get_eligible_suppliers(&reg, &req);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].supplier_id, "s1");
    }

    #[test]
    fn no_match_falls_back_to_all() {
        let reg = vec![
            supplier("s1", vec![model("m1", &["code-generation"])]),
            supplier("s2", vec![model("m2", &["logical-analysis"])]),
        ];
        let req = crate::pb::NormalizedRequest {
            agent_role: "creative-writing".into(),
            ..Default::default()
        };
        assert_eq!(get_eligible_suppliers(&reg, &req).len(), 2);
    }
}
