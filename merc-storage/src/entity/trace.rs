use crate::entity::Status;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Trace {
    pub id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
    pub request_id: Option<String>,
    pub status: Status,
    pub status_message: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}
