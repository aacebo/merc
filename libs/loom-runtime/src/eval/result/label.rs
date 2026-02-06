use serde::{Deserialize, Serialize};

/// Raw counts for a specific label.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LabelResult {
    pub expected_count: usize,
    pub detected_count: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
}
