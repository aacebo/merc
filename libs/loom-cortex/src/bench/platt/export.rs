use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Raw score export data for Platt calibration training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawScoreExport {
    pub samples: Vec<SampleScores>,
}

/// Individual sample with raw scores for each label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleScores {
    pub id: String,
    pub text: String,
    pub scores: HashMap<String, f32>,
    pub expected_labels: Vec<String>,
}
