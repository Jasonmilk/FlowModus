//! Judge points — the deterministic Rules backends of the judge-points
//! contract (docs/engineering-manual/judge-points-contract.md).
//!
//! FlowModus provides the **0-token** side of every judgment point
//! (contract §1): deterministic, config-sourced classifiers that never use an
//! LLM. The optional SmallLlm backends live on the consumer (Anaphase) side;
//! when they fail or emit an out-of-enum label, the consumer falls back to
//! these Rules (fail-safe, contract §2).
//!
//! Output enums are the deterministic contract surface — swapping backends
//! never changes the downstream enum (contract §2 "映射到同一枚举").

use serde::{Deserialize, Serialize};

/// JP-1 suggested mode (contract §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestedMode {
    Simple,
    Moderate,
    Complex,
}

impl SuggestedMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SuggestedMode::Simple => "simple",
            SuggestedMode::Moderate => "moderate",
            SuggestedMode::Complex => "complex",
        }
    }
}

/// JP-2 budget tier (contract §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetTier {
    Endogenous,
    Augmentable,
    Exogenous,
}

impl BudgetTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            BudgetTier::Endogenous => "endogenous",
            BudgetTier::Augmentable => "augmentable",
            BudgetTier::Exogenous => "exogenous",
        }
    }
}

/// Judge-point Rules configuration.
/// Defaults: contract §2 example heuristics (user-overridable — no hardcode).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JudgePointsConfig {
    /// input length <= this -> simple (JP-1)
    pub skilled_len: usize,
    /// input length < this -> moderate; >= this -> complex (JP-1)
    pub anchor_len: usize,
    /// keyword table for exploratory-intent detection (JP-2)
    pub explore_keywords: Vec<String>,
}

impl Default for JudgePointsConfig {
    fn default() -> Self {
        Self {
            // contract §2 example heuristic thresholds, user-overridable
            skilled_len: 48,
            anchor_len: 192,
            explore_keywords: vec![
                "explore".to_string(),
                "search".to_string(),
                "compare".to_string(),
                "how".to_string(),
                "what".to_string(),
            ],
        }
    }
}

/// JP-1: complexity assessment by length heuristic (pure, 0 tokens).
/// `<= skilled_len -> simple`, `< anchor_len -> moderate`, `>= anchor_len -> complex`.
pub fn judge_suggested_mode(input: &str, cfg: &JudgePointsConfig) -> SuggestedMode {
    let len = input.chars().count();
    if len <= cfg.skilled_len {
        SuggestedMode::Simple
    } else if len < cfg.anchor_len {
        SuggestedMode::Moderate
    } else {
        SuggestedMode::Complex
    }
}

/// JP-2: intent classification by length + keyword table (pure, 0 tokens).
/// - contains any explore keyword (case-insensitive) -> augmentable
///   (needs augmentation/search)
/// - long input with no keyword -> exogenous (needs external tooling)
/// - otherwise -> endogenous (self-contained)
pub fn judge_budget_tier(input: &str, cfg: &JudgePointsConfig) -> BudgetTier {
    let lowered = input.to_lowercase();
    let has_keyword = cfg
        .explore_keywords
        .iter()
        .any(|k| lowered.contains(&k.to_lowercase()));
    if has_keyword {
        BudgetTier::Augmentable
    } else if input.chars().count() >= cfg.anchor_len {
        BudgetTier::Exogenous
    } else {
        BudgetTier::Endogenous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_brackets() {
        let cfg = JudgePointsConfig::default();
        assert_eq!(judge_suggested_mode("hi", &cfg), SuggestedMode::Simple);
        assert_eq!(judge_suggested_mode(&"a".repeat(48), &cfg), SuggestedMode::Simple);
        assert_eq!(judge_suggested_mode(&"a".repeat(49), &cfg), SuggestedMode::Moderate);
        assert_eq!(judge_suggested_mode(&"a".repeat(191), &cfg), SuggestedMode::Moderate);
        assert_eq!(judge_suggested_mode(&"a".repeat(192), &cfg), SuggestedMode::Complex);
        assert_eq!(judge_suggested_mode(&"a".repeat(500), &cfg), SuggestedMode::Complex);
    }

    #[test]
    fn budget_tiers() {
        let cfg = JudgePointsConfig::default();
        assert_eq!(judge_budget_tier("write a haiku", &cfg), BudgetTier::Endogenous);
        assert_eq!(judge_budget_tier("explore this topic", &cfg), BudgetTier::Augmentable);
        assert_eq!(judge_budget_tier("HOW to deploy", &cfg), BudgetTier::Augmentable); // case-insensitive
        assert_eq!(judge_budget_tier(&"a".repeat(200), &cfg), BudgetTier::Exogenous);
        assert_eq!(judge_budget_tier(&format!("{}explore", "a".repeat(200)), &cfg), BudgetTier::Augmentable);
    }

    #[test]
    fn enum_labels_stable() {
        // downstream contract surface — backend swaps must never change these
        assert_eq!(SuggestedMode::Complex.as_str(), "complex");
        assert_eq!(BudgetTier::Exogenous.as_str(), "exogenous");
    }

    #[test]
    fn config_override_shape() {
        let cfg = JudgePointsConfig {
            skilled_len: 10,
            anchor_len: 20,
            explore_keywords: vec!["自定义".to_string()],
        };
        assert_eq!(judge_suggested_mode("12345678901", &cfg), SuggestedMode::Moderate);
        assert_eq!(judge_budget_tier("自定义主题", &cfg), BudgetTier::Augmentable);
    }
}
