use crate::chan::error::{RecvError, SendError};

/// Errors that can occur during task execution or when awaiting a task
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    /// Task was cancelled before completion
    Cancelled,

    /// Task panicked during execution
    Panic(String),

    /// Custom error with a message
    Custom(String),

    /// Task handle was dropped without sending a result
    Dropped,

    /// Failed to receive the task result
    Recv(RecvError),

    /// Failed to send the task result
    Send(SendError),
}

impl TaskError {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    pub fn is_panic(&self) -> bool {
        matches!(self, Self::Panic(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn is_dropped(&self) -> bool {
        matches!(self, Self::Dropped)
    }

    pub fn is_recv(&self) -> bool {
        matches!(self, Self::Recv(_))
    }

    pub fn is_send(&self) -> bool {
        matches!(self, Self::Send(_))
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

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "task cancelled"),
            Self::Panic(msg) => write!(f, "task panicked: {}", msg),
            Self::Custom(msg) => write!(f, "{}", msg),
            Self::Dropped => write!(f, "task handle dropped"),
            Self::Recv(e) => write!(f, "recv error: {}", e),
            Self::Send(e) => write!(f, "send error: {}", e),
        }
    }
}

impl std::error::Error for TaskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Recv(e) => Some(e),
            Self::Send(e) => Some(e),
            _ => None,
        }
    }
}

impl From<RecvError> for TaskError {
    fn from(err: RecvError) -> Self {
        Self::Recv(err)
    }
}

impl From<SendError> for TaskError {
    fn from(err: SendError) -> Self {
        Self::Send(err)
    }
}
