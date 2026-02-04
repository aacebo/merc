use std::io;

use crate::path::FieldPathError;

/// Errors that can occur during configuration operations
#[derive(Debug)]
pub enum ConfigError {
    /// Configuration file not found
    NotFound(String),

    /// IO error reading configuration
    IO(io::Error),

    /// Error parsing configuration content
    Parse(String),

    /// Error deserializing to target type
    Deserialize(String),

    /// Invalid field path
    InvalidPath(FieldPathError),

    /// Provider-specific error
    Provider(String),
}

impl ConfigError {
    pub fn not_found<S: Into<String>>(path: S) -> Self {
        Self::NotFound(path.into())
    }

    pub fn parse<E: std::error::Error>(err: E) -> Self {
        Self::Parse(err.to_string())
    }

    pub fn deserialize<E: std::error::Error>(err: E) -> Self {
        Self::Deserialize(err.to_string())
    }

    pub fn provider<S: Into<String>>(msg: S) -> Self {
        Self::Provider(msg.into())
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    pub fn is_io(&self) -> bool {
        matches!(self, Self::IO(_))
    }

    pub fn is_parse(&self) -> bool {
        matches!(self, Self::Parse(_))
    }

    pub fn is_deserialize(&self) -> bool {
        matches!(self, Self::Deserialize(_))
    }

    pub fn is_invalid_path(&self) -> bool {
        matches!(self, Self::InvalidPath(_))
    }

    pub fn is_provider(&self) -> bool {
        matches!(self, Self::Provider(_))
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "configuration not found: {}", path),
            Self::IO(e) => write!(f, "io error: {}", e),
            Self::Parse(msg) => write!(f, "parse error: {}", msg),
            Self::Deserialize(msg) => write!(f, "deserialize error: {}", msg),
            Self::InvalidPath(e) => write!(f, "invalid path: {}", e),
            Self::Provider(msg) => write!(f, "provider error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<FieldPathError> for ConfigError {
    fn from(err: FieldPathError) -> Self {
        Self::InvalidPath(err)
    }
}
