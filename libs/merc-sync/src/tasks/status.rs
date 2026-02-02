use std::sync::atomic::{AtomicU8, Ordering};

///
/// ## TaskStatus
/// represents the state of a Task
///
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Cancelled,
    Error,
    Ok,
}

impl TaskStatus {
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending => true,
            _ => false,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancelled => true,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Error => true,
            _ => false,
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            _ => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Self::Cancelled | Self::Error | Self::Ok => true,
            _ => false,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Cancelled,
            2 => Self::Error,
            _ => Self::Ok,
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<AtomicU8> for TaskStatus {
    fn from(value: AtomicU8) -> Self {
        match value.load(Ordering::Relaxed) {
            0 => Self::Pending,
            1 => Self::Cancelled,
            2 => Self::Error,
            _ => Self::Ok,
        }
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Error => write!(f, "error"),
            Self::Ok => write!(f, "ok"),
        }
    }
}
