use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub expected_decision: String,
    pub expected_labels: Vec<String>,
    pub primary_category: String,
    pub difficulty: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<String>,
    pub conversation_id: usize,
    pub turn_id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dataset {
    pub version: String,
    pub created: String,
    pub source: String,
    pub description: String,
    pub samples: Vec<Sample>,
}

impl Dataset {
    pub fn new(source: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            source: source.into(),
            description: description.into(),
            samples: Vec::new(),
        }
    }
}

/// Calculate difficulty based on word count
pub fn calculate_difficulty(text: &str) -> &'static str {
    let word_count = text.split_whitespace().count();
    if word_count < 5 {
        "easy"
    } else if word_count < 15 {
        "medium"
    } else {
        "hard"
    }
}
