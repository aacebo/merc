use serde::{Deserialize, Serialize};
use serde_valid::Validate;

/// Complete label definition
/// Note: Label name is the key in the parent BTreeMap
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ScoreLabelConfig {
    /// Hypothesis text for zero-shot classification
    #[validate(min_length = 1)]
    pub hypothesis: String,

    /// Weight applied to score when calculating importance
    #[serde(default = "ScoreLabelConfig::weight")]
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub weight: f32,

    /// Minimum score threshold for this label to be considered
    #[serde(default = "ScoreLabelConfig::threshold")]
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub threshold: f32,

    /// Platt scaling parameter A (default: 1.0 for identity)
    #[serde(default = "ScoreLabelConfig::platt_a")]
    pub platt_a: f32,

    /// Platt scaling parameter B (default: 0.0 for identity)
    #[serde(default)]
    pub platt_b: f32,
}

impl ScoreLabelConfig {
    fn weight() -> f32 {
        0.50
    }

    fn threshold() -> f32 {
        0.70
    }

    fn platt_a() -> f32 {
        1.0
    }
}

impl Default for ScoreLabelConfig {
    fn default() -> Self {
        Self {
            hypothesis: String::new(),
            weight: Self::weight(),
            threshold: Self::threshold(),
            platt_a: Self::platt_a(),
            platt_b: 0.0,
        }
    }
}
