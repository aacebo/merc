use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Category;

/// Coverage report for a benchmark dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_samples: usize,
    pub samples_by_category: HashMap<Category, usize>,
    pub samples_by_label: HashMap<String, usize>,
    pub missing_labels: Vec<String>,
    pub accept_count: usize,
    pub reject_count: usize,
}
