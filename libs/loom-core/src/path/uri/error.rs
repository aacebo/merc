use super::SchemeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorityError {
    InvalidPort,
    InvalidSyntax,
}

impl std::fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPort => write!(f, "invalid port"),
            Self::InvalidSyntax => write!(f, "invalid authority syntax"),
        }
    }
}

impl std::error::Error for AuthorityError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriError {
    Empty,
    MissingScheme,
    InvalidScheme(SchemeError),
    InvalidAuthority(AuthorityError),
}

impl std::fmt::Display for UriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty URI"),
            Self::MissingScheme => write!(f, "missing scheme"),
            Self::InvalidScheme(e) => write!(f, "invalid scheme: {:?}", e),
            Self::InvalidAuthority(e) => write!(f, "invalid authority: {}", e),
        }
    }
}

impl std::error::Error for UriError {}

impl From<AuthorityError> for UriError {
    fn from(e: AuthorityError) -> Self {
        Self::InvalidAuthority(e)
    }
}
