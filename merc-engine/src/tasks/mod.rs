mod id;
mod status;

pub use id::*;
use merc_error::{Error, Result};
pub use status::*;

use std::{
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
};

type TaskState<T> = (Option<T>, Option<Waker>);

pub trait Spawn: Send + Sync + 'static {
    fn spawn<T, H>(&self, handler: H) -> Task<T>
    where
        T: Send + 'static,
        H: FnOnce() -> T + Send + 'static;
}

///
/// ## Task
/// represents some unit of async work
///
#[derive(Clone)]
pub struct Task<T> {
    id: TaskId,
    status: TaskStatus,
    result: TaskResult<T>,
}

impl<T> Task<T> {
    pub fn new() -> Self {
        Self {
            id: TaskId::new(),
            status: TaskStatus::Pending,
            result: TaskResult(Arc::new(Mutex::new((None, None)))),
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

    pub(crate) fn result(&self) -> TaskResult<T> {
        self.result.clone()
    }
}

impl<T> Future for Task<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let task = self.get_mut();

        if task.status.is_cancelled() {
            return Poll::Pending;
        }

        let mut mutex = task.result.lock();

        if let Some(result) = mutex.0.take() {
            task.status = TaskStatus::Complete;
            return Poll::Ready(result);
        }

        mutex.1 = Some(cx.waker().clone());
        task.status = TaskStatus::Running;
        Poll::Pending
    }
}

///
/// ## TaskResult
/// holds the tasks result state (value or error)
/// and exposes methods for completing the task.
///
pub struct TaskResult<T>(Arc<Mutex<TaskState<Result<T>>>>);

impl<T> Clone for TaskResult<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> TaskResult<T> {
    pub fn ok(&self, value: T) {
        let mut mutex = self.lock();
        mutex.0 = Some(Ok(value));

        if let Some(waker) = mutex.1.take() {
            waker.wake();
        }
    }

    pub fn throw(&self, error: Error) {
        let mut mutex = self.lock();
        mutex.0 = Some(Err(error));

        if let Some(waker) = mutex.1.take() {
            waker.wake();
        }
    }

    fn lock(&self) -> MutexGuard<'_, TaskState<Result<T>>> {
        self.0.lock().unwrap()
    }
}
