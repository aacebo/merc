use serde::{Deserialize, Serialize};

/// Raw counts for a specific category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryResult {
    pub total: usize,
    pub correct: usize,
}
