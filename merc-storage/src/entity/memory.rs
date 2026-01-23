use crate::entity::Sensitivity;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Memory {
    pub id: uuid::Uuid,
    pub scope_id: uuid::Uuid,
    pub score: f32,
    pub confidence: f32,
    pub importance: f32,
    pub sensitivity: Sensitivity,
    pub tags: Vec<String>,
    pub embedding: Option<Vec<f32>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
