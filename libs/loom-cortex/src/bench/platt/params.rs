use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Trained Platt scaling parameters for a single label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattParams {
    pub a: f32,
    pub b: f32,
}

impl Default for PlattParams {
    fn default() -> Self {
        Self { a: 1.0, b: 0.0 }
    }
}

/// Result of training Platt parameters for all labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattTrainingResult {
    pub params: HashMap<String, PlattParams>,
    pub metadata: PlattTrainingMetadata,
}

/// Metadata about the Platt training process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattTrainingMetadata {
    pub total_samples: usize,
    pub samples_per_label: HashMap<String, LabelStats>,
}

/// Statistics for a single label during training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStats {
    pub positive: usize,
    pub negative: usize,
    pub skipped: bool,
}
