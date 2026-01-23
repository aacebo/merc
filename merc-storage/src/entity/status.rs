#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
pub enum Status {
    Ok,
    Error,
    Cancelled,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "Ok"),
            Self::Error => write!(f, "Error"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}
