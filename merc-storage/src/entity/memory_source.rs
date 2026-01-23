#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MemorySource {
    pub memory_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub confidence: f32,
    pub text: Option<String>,
    pub hash: String,
    pub start_offset: i32,
    pub end_offset: i32,
}
