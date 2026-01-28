use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

static COUNTER: AtomicU64 = AtomicU64::new(0);

///
/// ## Task
/// represents some unit of async work
///
pub struct Task<T>
where
    T: Future + Send + 'static,
{
    id: TaskId,
    status: TaskStatus,
    inner: Option<T>,
}

impl<T> Task<T>
where
    T: Future + Send + 'static,
{
    pub fn new(future: T) -> Self {
        Self {
            id: TaskId::new(),
            status: TaskStatus::Pending,
            inner: Some(future),
        }
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
    }
}

impl<T> Future for Task<T>
where
    T: Future + Send + 'static,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let task = unsafe { self.get_unchecked_mut() };

        if task.status.is_cancelled() {
            return Poll::Pending;
        }

        task.status = TaskStatus::Running;

        match &mut task.inner {
            None => Poll::Pending,
            Some(fut) => {
                let pinned = unsafe { Pin::new_unchecked(fut) };

                match pinned.poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(output) => {
                        task.status = TaskStatus::Complete;
                        Poll::Ready(output)
                    }
                }
            }
        }
    }
}

///
/// ## TaskId
/// an auto incrementing atomic
/// task identifier
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> &u64 {
        &self.0
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
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
