use serde::{Deserialize, Serialize};

use crate::eval::Decision;

/// Result for a single sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleResult {
    pub id: String,
    pub expected_decision: Decision,
    pub actual_decision: Decision,
    pub correct: bool,
    pub score: f32,
    pub expected_labels: Vec<String>,
    pub detected_labels: Vec<String>,
}
