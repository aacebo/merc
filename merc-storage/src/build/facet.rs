use crate::entity::{Facet, FacetType};

#[derive(Debug, Clone)]
pub struct FacetBuilder {
    memory_id: uuid::Uuid,
    ty: FacetType,
    confidence: f32,
    data: Vec<u8>,
}

impl FacetBuilder {
    pub fn new(memory_id: uuid::Uuid, ty: FacetType) -> Self {
        Self {
            memory_id,
            ty,
            confidence: 1.0,
            data: Vec::new(),
        }
    }

    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Facet {
        let now = chrono::Utc::now();
        Facet {
            id: uuid::Uuid::new_v4(),
            memory_id: self.memory_id,
            ty: self.ty,
            confidence: self.confidence,
            data: self.data,
            created_at: now,
            updated_at: now,
        }
    }
}
