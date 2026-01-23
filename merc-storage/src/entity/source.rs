#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Source {
    pub id: uuid::Uuid,
    pub scope_id: uuid::Uuid,
    pub external_id: String,
    #[sqlx(rename = "type")]
    pub ty: SourceType,
    pub uri: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum SourceType {
    Chat,
    Document,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat => write!(f, "Chat"),
            Self::Document => write!(f, "Document"),
        }
    }
}
