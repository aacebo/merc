use crate::entity::{Source, SourceType};

#[derive(Debug, Clone)]
pub struct SourceBuilder {
    scope_id: uuid::Uuid,
    external_id: String,
    ty: SourceType,
    uri: Option<String>,
}

impl SourceBuilder {
    pub fn new(scope_id: uuid::Uuid, external_id: impl Into<String>, ty: SourceType) -> Self {
        Self {
            scope_id,
            external_id: external_id.into(),
            ty,
            uri: None,
        }
    }

    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn build(self) -> Source {
        Source {
            id: uuid::Uuid::new_v4(),
            scope_id: self.scope_id,
            external_id: self.external_id,
            ty: self.ty,
            uri: self.uri,
            created_at: chrono::Utc::now(),
        }
    }
}
