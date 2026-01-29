#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ChanError {
    Send(SendError),
    Recv(RecvError),
}

impl std::fmt::Display for ChanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Send(v) => write!(f, "{}", v),
            Self::Recv(v) => write!(f, "{}", v),
        }
    }
}

impl std::error::Error for ChanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Send(err) => Some(err),
            Self::Recv(err) => Some(err),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SendError {
    /// the channel is closed
    Closed,

    /// the channel is full
    Full,
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Full => write!(f, "full"),
        }
    }
}

impl std::error::Error for SendError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RecvError {
    /// the channel is closed
    Closed,

    /// the channel is empty (no messages available)
    Empty,
}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Empty => write!(f, "empty"),
        }
    }
}

impl std::error::Error for RecvError {}
