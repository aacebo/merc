use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use super::ScoreLabelConfig;

/// Category definition containing labels
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ScoreCategoryConfig {
    /// Category name (e.g., "sentiment", "emotion", "outcome", "context")
    #[validate(min_length = 3)]
    pub name: String,

    /// Number of top labels to consider for this category
    #[serde(default = "ScoreCategoryConfig::top_k")]
    #[validate(minimum = 1)]
    pub top_k: usize,

    /// Labels belonging to this category
    #[validate]
    #[validate(min_items = 1)]
    pub labels: Vec<ScoreLabelConfig>,
}

impl ScoreCategoryConfig {
    fn top_k() -> usize {
        2
    }
}

impl Default for ScoreCategoryConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            top_k: Self::top_k(),
            labels: Vec::new(),
        }
    }
}
