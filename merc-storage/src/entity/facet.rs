#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Facet {
    pub id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    #[sqlx(rename = "type")]
    pub ty: FacetType,
    pub confidence: f32,
    pub data: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum FacetType {
    /// Stable likes/dislikes, “always/usually” choices
    Preference,

    /// Long-lived profile facts (role, timezone, tools, background) that tend not to change often
    Profile,

    /// People + how they relate (manager, partner, teammate), plus lightweight social graph
    Relationship,

    /// A durable fact about the world or the user that’s useful later (often “as-of” dated)
    Fact,

    /// A decision the user made (“We will use Postgres”, “No meetings Fridays”)
    Decision,

    /// Rules / constraints / policies (“Don’t mention X”, “Always ask before sharing”)
    Policy,

    /// Plans, goals, tasks, commitments (usually time-bounded)
    Plan,

    /// Something that happened (an event/experience) with time + participants + context
    Episode,

    /// Ongoing work context (project state, architecture, open questions) with versioning
    ProjectContext,

    /// A lesson learned / postmortem / “next time do X”
    Insight,

    /// Short-lived status (“I’m traveling”, “I’m sick today”) — should almost always have TTL
    Status,

    /// The user’s feedback signal (reward/punishment, satisfaction), typically TTL-limited
    Feedback,
}
