#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Type {
    /// Generic events
    Event,
    /// Represents a timed operation (has duration)
    Span,
    /// Numeric measurements
    Metric,
    /// Log-style messages
    Log,
}

impl Type {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Event => "event",
            Self::Span => "span",
            Self::Metric => "metric",
            Self::Log => "log",
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
