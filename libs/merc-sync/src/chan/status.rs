use std::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    /// Senders may still send (subject to capacity).
    Open,

    /// No new sends possible, but receiver may still yield buffered messages.
    Draining,

    /// Closed and empty; receiver will never yield another message.
    Closed,
}

impl Status {
    pub fn is_open(&self) -> bool {
        match self {
            Self::Open => true,
            _ => false,
        }
    }

    pub fn is_draining(&self) -> bool {
        match self {
            Self::Draining => true,
            _ => false,
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Closed => true,
            _ => false,
        }
    }

    pub fn is_closing(&self) -> bool {
        match self {
            Self::Draining | Self::Closed => true,
            _ => false,
        }
    }

    pub fn atomic(&self) -> AtomicU8 {
        match self {
            Self::Open => AtomicU8::new(0),
            Self::Draining => AtomicU8::new(1),
            Self::Closed => AtomicU8::new(2),
        }
    }
}

impl From<&AtomicU8> for Status {
    fn from(value: &AtomicU8) -> Self {
        match value.load(Ordering::Relaxed) {
            0 => Self::Open,
            1 => Self::Draining,
            _ => Self::Closed,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::Closed
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Draining => write!(f, "draining"),
            Self::Closed => write!(f, "closed"),
        }
    }
}
