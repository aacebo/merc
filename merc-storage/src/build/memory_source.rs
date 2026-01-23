use crate::entity::MemorySource;

#[derive(Debug, Clone)]
pub struct MemorySourceBuilder {
    memory_id: uuid::Uuid,
    source_id: uuid::Uuid,
    confidence: f32,
    text: Option<String>,
    hash: String,
    start_offset: i32,
    end_offset: i32,
}

impl MemorySourceBuilder {
    pub fn new(memory_id: uuid::Uuid, source_id: uuid::Uuid, hash: impl Into<String>) -> Self {
        Self {
            memory_id,
            source_id,
            confidence: 1.0,
            text: None,
            hash: hash.into(),
            start_offset: 0,
            end_offset: 0,
        }
    }

    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn offsets(mut self, start: i32, end: i32) -> Self {
        self.start_offset = start;
        self.end_offset = end;
        self
    }

    pub fn build(self) -> MemorySource {
        MemorySource {
            memory_id: self.memory_id,
            source_id: self.source_id,
            confidence: self.confidence,
            text: self.text,
            hash: self.hash,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
        }
    }
}
