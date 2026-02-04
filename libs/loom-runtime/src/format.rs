#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Format {
    Json,
    Yaml,
    Toml,
    Xml,
    Csv,
    Markdown,
    Html,
    Text,
    Binary,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Xml => write!(f, "xml"),
            Self::Csv => write!(f, "csv"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Text => write!(f, "text"),
            Self::Binary => write!(f, "binary"),
        }
    }
}
