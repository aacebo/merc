use crate::tasks::TaskError;

pub enum TaskResult<T: Send + 'static> {
    Cancelled,
    Error(TaskError),
    Ok(T),
}

impl<T: Send + 'static> TaskResult<T> {
    pub fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancelled => true,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _ => false,
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok(_) => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(value) => value,
            Self::Cancelled => panic!("called `TaskResult::unwrap()` on a `Cancelled` value"),
            Self::Error(err) => panic!("called `TaskResult::unwrap()` on an `Error` value: {err}"),
        }
    }

    pub fn unwrap_err(self) -> TaskError {
        match self {
            Self::Error(err) => err,
            Self::Ok(_) => panic!("called `TaskResult::unwrap_err()` on an `Ok` value"),
            Self::Cancelled => panic!("called `TaskResult::unwrap_err()` on a `Cancelled` value"),
        }
    }
}

impl<T: Send + 'static> std::fmt::Debug for TaskResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "TaskResult::Cancelled"),
            Self::Error(err) => f.debug_tuple("TaskResult::Err").field(err).finish(),
            Self::Ok(_) => f.debug_tuple("TaskResult::Ok").field(&"<value>").finish(),
        }
    }
}
