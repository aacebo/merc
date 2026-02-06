use serde::{Deserialize, Serialize};

/// Progress information passed to callbacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub sample_id: String,
    pub correct: bool,
}
