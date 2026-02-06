use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    Unknown,
    Cancel,
    NotFound,
    BadArguments,
}

impl ErrorCode {
    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown => true,
            _ => false,
        }
    }

    pub fn is_cancel(&self) -> bool {
        match self {
            Self::Cancel => true,
            _ => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            Self::NotFound => true,
            _ => false,
        }
    }

    pub fn is_bad_arguments(&self) -> bool {
        match self {
            Self::BadArguments => true,
            _ => false,
        }
    }
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "cancel"),
            Self::Unknown => write!(f, "unknown"),
            Self::NotFound => write!(f, "not-found"),
            Self::BadArguments => write!(f, "bad-arguments"),
        }
    }
}
