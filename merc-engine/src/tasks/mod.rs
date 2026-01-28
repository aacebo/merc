mod id;
mod status;

pub use id::*;
pub use status::*;

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub fn new<T>() -> Task<T> {
    Task::<T>::new()
}

///
/// ## Execute
/// represents an async runtime that
/// can spawn/track/manage tasks
///
pub trait Execute: Send + Sync + 'static {
    fn spawn<T, F, H>(&self, handler: H) -> Task<T>
    where
        T: Send + 'static,
        F: Future<Output = T> + Send + 'static,
        H: FnOnce() -> F + Send + 'static;
}

type TaskState<T> = (Option<T>, Option<Waker>);

///
/// ## Task
/// represents some unit of async work
///
#[derive(Clone)]
pub struct Task<T> {
    id: TaskId,
    status: TaskStatus,
    inner: Arc<Mutex<TaskState<T>>>,
}

impl<T> Task<T> {
    pub fn new() -> Self {
        Self {
            id: TaskId::new(),
            status: TaskStatus::Pending,
            inner: Arc::new(Mutex::new((None, None))),
        }
    }

    pub fn sender(&self) -> TaskSender<T> {
        TaskSender(Arc::clone(&self.inner))
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

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.status.is_cancelled() {
            return Poll::Pending;
        }

        let mut shared = this.inner.lock().unwrap();

        if let Some(result) = shared.0.take() {
            this.status = TaskStatus::Complete;
            return Poll::Ready(result);
        }

        shared.1 = Some(cx.waker().clone());
        this.status = TaskStatus::Running;
        Poll::Pending
    }
}

///
/// ## TaskSender
/// The sending half used to deliver results to a Task.
///
pub struct TaskSender<T>(Arc<Mutex<TaskState<T>>>);

impl<T> TaskSender<T> {
    pub fn send(self, value: T) {
        let mut shared = self.0.lock().unwrap();
        shared.0 = Some(value);

        if let Some(waker) = shared.1.take() {
            waker.wake();
        }
    }
}
