#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TraceAction {
    pub trace_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    pub target: Target,
    pub action: Action,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Target {
    Memory,
    Facet,
    Source,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Action {
    Create,
    Update,
    Delete,
    Read,
    Cite,
}
