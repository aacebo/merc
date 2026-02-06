use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use super::ScoreLabelConfig;

/// Category definition containing labels
/// Note: Category name is the key in the parent BTreeMap
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ScoreCategoryConfig {
    /// Number of top labels to consider for this category
    #[serde(default = "ScoreCategoryConfig::top_k")]
    #[validate(minimum = 1)]
    pub top_k: usize,

    /// Labels belonging to this category (keyed by label name)
    pub labels: BTreeMap<String, ScoreLabelConfig>,
}

impl ScoreCategoryConfig {
    fn top_k() -> usize {
        2
    }
}

impl Default for ScoreCategoryConfig {
    fn default() -> Self {
        Self {
            top_k: Self::top_k(),
            labels: BTreeMap::new(),
        }
    }
}
