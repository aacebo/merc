#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Sensitivity {
    Low,
    Personal,
    Sensitive,
    Restricted,
}

impl std::fmt::Display for Sensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Personal => write!(f, "Personal"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Sensitive => write!(f, "Sensitive"),
        }
    }
}
