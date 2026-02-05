use serde::{Deserialize, Serialize};

/// Category classification for benchmark samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Task,
    Emotional,
    Factual,
    Preference,
    Decision,
    Phatic,
    Ambiguous,
}
