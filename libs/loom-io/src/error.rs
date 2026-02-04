use std::io;

/// Errors that can occur during data source read operations
#[derive(Debug)]
pub enum ReadError {
    /// IO error during read
    IO(io::Error),

    /// Read operation panicked during execution
    Panic(String),

    /// Custom error with a message
    Custom(String),
}

impl ReadError {
    pub fn is_io(&self) -> bool {
        matches!(self, Self::IO(_))
    }

    pub fn is_panic(&self) -> bool {
        matches!(self, Self::Panic(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Create a custom error from any error type
    pub fn custom<E: std::error::Error>(err: E) -> Self {
        Self::Custom(err.to_string())
    }

    /// Create a panic error from panic payload
    pub fn panic<S: Into<String>>(msg: S) -> Self {
        Self::Panic(msg.into())
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(e) => write!(f, "io error: {}", e),
            Self::Panic(msg) => write!(f, "read panicked: {}", msg),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

/// Errors that can occur during data source write operations
#[derive(Debug)]
pub enum WriteError {
    /// IO error during write
    IO(io::Error),

    /// Write operation panicked during execution
    Panic(String),

    /// Custom error with a message
    Custom(String),
}

impl WriteError {
    pub fn is_io(&self) -> bool {
        matches!(self, Self::IO(_))
    }

    pub fn is_panic(&self) -> bool {
        matches!(self, Self::Panic(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Create a custom error from any error type
    pub fn custom<E: std::error::Error>(err: E) -> Self {
        Self::Custom(err.to_string())
    }

    /// Create a panic error from panic payload
    pub fn panic<S: Into<String>>(msg: S) -> Self {
        Self::Panic(msg.into())
    }
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(e) => write!(f, "io error: {}", e),
            Self::Panic(msg) => write!(f, "write panicked: {}", msg),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}
