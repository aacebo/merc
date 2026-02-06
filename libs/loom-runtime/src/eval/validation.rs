use serde::{Deserialize, Serialize};

/// Validation error for a benchmark sample.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationError {
    pub sample_id: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.sample_id, self.message)
    }
}
