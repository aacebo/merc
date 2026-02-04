use std::task::Poll;

use crate::{
    chan,
    tasks::{TaskId, TaskResult, TaskStatus},
};

///
/// ## Task<T>
/// a reference to some async operation
/// that will eventually resolve to T
///
pub struct Task<T: Send + 'static> {
    id: TaskId,
    status: TaskStatus,
    receiver: Box<dyn chan::Receiver<Item = TaskResult<T>>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new<R: chan::Receiver<Item = TaskResult<T>> + 'static>(receiver: R) -> Self {
        Self {
            id: TaskId::new(),
            status: TaskStatus::Pending,
            receiver: Box::new(receiver),
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn channel(&self) -> &dyn chan::Channel {
        self.receiver.as_ref()
    }

    pub fn wait(&mut self) -> Result<TaskResult<T>, chan::error::RecvError> {
        self.receiver.recv()
    }
}

impl<T: Send + 'static> chan::Channel for Task<T> {
    fn status(&self) -> chan::Status {
        self.receiver.status()
    }

    fn len(&self) -> usize {
        self.receiver.len()
    }

    fn capacity(&self) -> Option<usize> {
        self.receiver.capacity()
    }
}

impl<T: Send + 'static> Future for Task<T> {
    type Output = TaskResult<T>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.receiver.recv_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => match res {
                Ok(value) => {
                    if value.is_cancelled() {
                        self.status = TaskStatus::Cancelled;
                    } else if value.is_error() {
                        self.status = TaskStatus::Error;
                    } else {
                        self.status = TaskStatus::Ok;
                    }

                    Poll::Ready(value)
                }
                Err(err) => {
                    self.status = TaskStatus::Error;
                    Poll::Ready(TaskResult::Error(err.into()))
                }
            },
        }
    }
}

impl<T: Send + 'static> Drop for Task<T> {
    fn drop(&mut self) {
        if !(self.receiver.status().is_closed()) {
            let _ = self.receiver.close();
        }
    }
}

#[cfg(all(test, feature = "tokio"))]
mod tests {
    use super::*;
    use crate::chan::Channel;
    use crate::spawn;
    use futures::future::poll_fn;

    #[tokio::test]
    async fn test_new_task_has_pending_status() {
        let (task, _): (Task<i32>, _) = spawn!();
        assert!(task.status().is_pending());
    }

    #[tokio::test]
    async fn test_tasks_have_unique_ids() {
        let (task1, _r1): (Task<i32>, _) = spawn!();
        let (task2, _r2): (Task<i32>, _) = spawn!();
        let (task3, _r3): (Task<i32>, _) = spawn!();

        assert_ne!(task1.id(), task2.id());
        assert_ne!(task2.id(), task3.id());
        assert_ne!(task1.id(), task3.id());
    }

    #[tokio::test]
    async fn test_task_and_resolver_share_id() {
        let (task, resolver): (Task<i32>, _) = spawn!();
        assert_eq!(task.id(), resolver.id());
    }

    #[tokio::test]
    async fn test_channel_len_and_capacity() {
        let (task, _resolver): (Task<i32>, _) = spawn!();

        // Underlying channel has capacity of 1 (from spawn!())
        assert_eq!(task.capacity(), Some(1));
        assert_eq!(task.len(), 0);
    }

    #[tokio::test]
    async fn test_poll_pending_before_resolve() {
        let (mut task, _resolver): (Task<i32>, _) = spawn!();

        let poll_result = poll_fn(|cx| {
            let pinned = std::pin::Pin::new(&mut task);
            Poll::Ready(pinned.poll(cx))
        })
        .await;

        assert!(poll_result.is_pending());
        assert!(task.status().is_pending());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_complete_sets_status_ok() {
        let (task, resolver): (Task<i32>, _) = spawn!();

        // Use spawn_blocking for blocking send operation
        tokio::task::spawn_blocking(move || {
            resolver.ok(42).unwrap();
        });

        let result = task.await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_status_is_ok_after_complete() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Use spawn_blocking for blocking send operation
        tokio::task::spawn_blocking(move || {
            resolver.ok(42).unwrap();
        });

        // Poll to completion
        let _ = (&mut task).await;
        assert!(task.status().is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_fail_returns_error_result() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Use spawn_blocking for blocking send operation
        tokio::task::spawn_blocking(move || {
            resolver.error("something went wrong").unwrap();
        });

        let result = (&mut task).await;
        let err = result.unwrap_err();
        assert!(err.is_custom());
        assert!(err.to_string().contains("something went wrong"));
        assert!(task.status().is_error());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cancel_sets_status_cancelled() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Use spawn_blocking for blocking send operation
        tokio::task::spawn_blocking(move || {
            resolver.cancel().unwrap();
        });

        let result = (&mut task).await;
        assert!(result.is_cancelled());
        assert!(task.status().is_cancelled());
    }

    #[tokio::test]
    async fn test_dropped_resolver_causes_error() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Drop the resolver without completing
        drop(resolver);

        let result = (&mut task).await;
        assert!(result.is_error());
        assert!(task.status().is_error());
    }

    #[tokio::test]
    async fn test_channel_status_open_initially() {
        use crate::chan::Status;

        let (task, _resolver): (Task<i32>, _) = spawn!();
        assert_eq!(task.channel().status(), Status::Open);
    }

    // ==================== Sync versions (using wait()) ====================

    #[test]
    fn test_new_task_has_pending_status_sync() {
        let (task, _): (Task<i32>, _) = spawn!();
        assert!(task.status().is_pending());
    }

    #[test]
    fn test_tasks_have_unique_ids_sync() {
        let (task1, _r1): (Task<i32>, _) = spawn!();
        let (task2, _r2): (Task<i32>, _) = spawn!();
        let (task3, _r3): (Task<i32>, _) = spawn!();

        assert_ne!(task1.id(), task2.id());
        assert_ne!(task2.id(), task3.id());
        assert_ne!(task1.id(), task3.id());
    }

    #[test]
    fn test_task_and_resolver_share_id_sync() {
        let (task, resolver): (Task<i32>, _) = spawn!();
        assert_eq!(task.id(), resolver.id());
    }

    #[test]
    fn test_channel_len_and_capacity_sync() {
        let (task, _resolver): (Task<i32>, _) = spawn!();

        // Underlying channel has capacity of 1 (from spawn!())
        assert_eq!(task.capacity(), Some(1));
        assert_eq!(task.len(), 0);
    }

    #[test]
    fn test_complete_returns_ok_sync() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Complete from another thread (blocking send)
        std::thread::spawn(move || {
            resolver.ok(42).unwrap();
        });

        let result = task.wait().unwrap();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_fail_returns_error_result_sync() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        std::thread::spawn(move || {
            resolver.error("something went wrong").unwrap();
        });

        let result = task.wait().unwrap();
        let err = result.unwrap_err();
        assert!(err.is_custom());
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn test_cancel_returns_cancelled_sync() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        std::thread::spawn(move || {
            resolver.cancel().unwrap();
        });

        let result = task.wait().unwrap();
        assert!(result.is_cancelled());
    }

    #[test]
    fn test_dropped_resolver_causes_recv_error_sync() {
        let (mut task, resolver): (Task<i32>, _) = spawn!();

        // Drop the resolver without completing
        drop(resolver);

        let result = task.wait();
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_status_open_initially_sync() {
        use crate::chan::Status;

        let (task, _resolver): (Task<i32>, _) = spawn!();
        assert_eq!(task.channel().status(), Status::Open);
    }
}
