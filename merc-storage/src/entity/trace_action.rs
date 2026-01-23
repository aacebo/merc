#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TraceAction {
    pub trace_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    pub target: Target,
    pub action: Action,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
pub enum Target {
    Memory,
    Facet,
    Source,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
pub enum Action {
    Create,
    Update,
    Delete,
    Read,
    Cite,
}
