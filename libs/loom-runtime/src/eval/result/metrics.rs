use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Computed metrics for overall benchmark performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub per_category: HashMap<String, CategoryMetrics>,
    pub per_label: HashMap<String, LabelMetrics>,
}

/// Computed metrics for a specific category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub accuracy: f32,
}

/// Computed metrics for a specific label.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LabelMetrics {
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
}
