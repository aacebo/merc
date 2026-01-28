use merc_error::Result;

///
/// ## Task
/// represents some unit of async work
///
pub trait Task<T>: Send + 'static
where
    T: Send + 'static,
{
    fn id(&self) -> u64;
    fn status(&self) -> &TaskStatus;
    fn cancel(&self);
    fn wait(&self) -> Result<T>;
}

///
/// ## TaskStatus
/// represents the state of a Task
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Running,
    Cancelled,
    Complete,
}

impl TaskStatus {
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending => true,
            _ => false,
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            Self::Running => true,
            _ => false,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancelled => true,
            _ => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Self::Complete => true,
            _ => false,
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
            Self::Running => write!(f, "running"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Complete => write!(f, "complete"),
        }
    }
}
