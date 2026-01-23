use crate::entity::{Memory, Sensitivity};

#[derive(Debug, Clone)]
pub struct MemoryBuilder {
    scope_id: uuid::Uuid,
    score: f32,
    confidence: f32,
    importance: f32,
    sensitivity: Sensitivity,
    tags: Vec<String>,
    embedding: Option<Vec<f32>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MemoryBuilder {
    pub fn new(scope_id: uuid::Uuid) -> Self {
        Self {
            scope_id,
            score: 0.5,
            confidence: 0.5,
            importance: 0.5,
            sensitivity: Sensitivity::Low,
            tags: Vec::new(),
            embedding: None,
            expires_at: None,
        }
    }

    pub fn score(mut self, score: f32) -> Self {
        self.score = score;
        self
    }

    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn importance(mut self, importance: f32) -> Self {
        self.importance = importance;
        self
    }

    pub fn sensitivity(mut self, sensitivity: Sensitivity) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn expires_at(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn build(self) -> Memory {
        let now = chrono::Utc::now();
        Memory {
            id: uuid::Uuid::new_v4(),
            scope_id: self.scope_id,
            score: self.score,
            confidence: self.confidence,
            importance: self.importance,
            sensitivity: self.sensitivity,
            tags: self.tags,
            embedding: self.embedding,
            expires_at: self.expires_at,
            created_at: now,
            updated_at: now,
        }
    }
}
