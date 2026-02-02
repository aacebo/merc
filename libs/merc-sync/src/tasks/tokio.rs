use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::{chan::tokio::alloc, tasks::TaskResult};

use super::{Task, TaskError, TaskResolver};

pub fn new<T: Send + 'static>() -> (Task<T>, TaskResolver<T>) {
    let (sender, receiver) = alloc::<TaskResult<T>>(1);
    let task = Task::new(receiver);
    let handle = TaskResolver::<T>::new(task.id(), sender);

    (task, handle)
}

pub fn spawn<T, F>(future: F) -> Task<T>
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
{
    let (task, handle) = new::<T>();

    tokio::spawn(async move {
        // Catch panics and convert to errors
        let result = AssertUnwindSafe(future).catch_unwind().await;

        match result {
            Ok(value) => {
                let _ = handle.ok(value);
            }
            Err(panic_info) => {
                let msg = panic_payload_to_string(panic_info);
                let _ = handle.error(TaskError::panic(msg));
            }
        }
    });

    task
}

pub fn spawn_result<T, E, F>(future: F) -> Task<T>
where
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
{
    let (task, handle) = new::<T>();

    tokio::spawn(async move {
        let result = AssertUnwindSafe(future).catch_unwind().await;

        match result {
            Ok(Ok(value)) => {
                let _ = handle.ok(value);
            }
            Ok(Err(e)) => {
                let _ = handle.error(TaskError::custom(e));
            }
            Err(panic_info) => {
                let msg = panic_payload_to_string(panic_info);
                let _ = handle.error(TaskError::panic(msg));
            }
        }
    });

    task
}

/// Convert panic payload to a string message
fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}
