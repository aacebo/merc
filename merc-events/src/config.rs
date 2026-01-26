#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EventsConfig {
    pub uri: String,
}
