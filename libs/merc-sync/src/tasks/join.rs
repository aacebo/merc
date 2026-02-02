use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::chan::error::RecvError;

use super::{Task, TaskResult};

/// Join multiple tasks concurrently (heterogeneous types).
/// Re-exports futures::join! since Task<T> implements Future.
///
/// # Example
/// ```ignore
/// let (r1, r2, r3) = join!(task1, task2, task3).await;
/// ```
#[macro_export]
macro_rules! join {
    ($($task:expr),+ $(,)?) => {
        futures::join!($($task),+)
    };
}

/// Join all tasks of the same type, returning results in order.
///
/// # Example
/// ```ignore
/// let results: Vec<TaskResult<i32>> = join_all(vec![task1, task2, task3]).await;
/// ```
pub fn join_all<T>(tasks: Vec<Task<T>>) -> JoinAll<T>
where
    T: Send + 'static,
{
    let len = tasks.len();
    JoinAll {
        tasks: tasks.into_iter().map(Some).collect(),
        results: (0..len).map(|_| None).collect(),
    }
}

/// Future that completes when all tasks complete.
pub struct JoinAll<T: Send + 'static> {
    tasks: Vec<Option<Task<T>>>,
    results: Vec<Option<TaskResult<T>>>,
}

// JoinAll is Unpin because we never move the inner tasks after creation
impl<T: Send + 'static> Unpin for JoinAll<T> {}

impl<T: Send + 'static> Future for JoinAll<T> {
    type Output = Vec<TaskResult<T>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        let mut all_done = true;

        for i in 0..this.tasks.len() {
            if this.results[i].is_some() {
                continue; // Already done
            }

            if let Some(ref mut task) = this.tasks[i] {
                match Pin::new(task).poll(cx) {
                    Poll::Ready(result) => {
                        this.results[i] = Some(result);
                    }
                    Poll::Pending => {
                        all_done = false;
                    }
                }
            }
        }

        if all_done {
            let results = this.results.iter_mut().map(|r| r.take().unwrap()).collect();
            Poll::Ready(results)
        } else {
            Poll::Pending
        }
    }
}

/// Blocking wait for all tasks using threads.
///
/// # Example
/// ```ignore
/// let results = wait_all(vec![task1, task2, task3]);
/// ```
pub fn wait_all<T>(tasks: Vec<Task<T>>) -> Vec<Result<TaskResult<T>, RecvError>>
where
    T: Send + 'static,
{
    let handles: Vec<_> = tasks
        .into_iter()
        .map(|mut task| std::thread::spawn(move || task.wait()))
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().expect("task thread panicked"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::tokio::new;

    // ==================== Async tests ====================

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_join_all_completes_all_tasks() {
        let (t1, r1) = new::<i32>();
        let (t2, r2) = new::<i32>();
        let (t3, r3) = new::<i32>();

        // Complete tasks from separate threads
        std::thread::spawn(move || r1.ok(1).unwrap());
        std::thread::spawn(move || r2.ok(2).unwrap());
        std::thread::spawn(move || r3.ok(3).unwrap());

        let results = join_all(vec![t1, t2, t3]).await;

        assert_eq!(results.len(), 3);
        match &results[0] {
            TaskResult::Ok(v) => assert_eq!(*v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match &results[1] {
            TaskResult::Ok(v) => assert_eq!(*v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match &results[2] {
            TaskResult::Ok(v) => assert_eq!(*v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_join_all_preserves_order() {
        let (t1, r1) = new::<i32>();
        let (t2, r2) = new::<i32>();
        let (t3, r3) = new::<i32>();

        // Complete in reverse order
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(30));
            r1.ok(1).unwrap();
        });
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            r2.ok(2).unwrap();
        });
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            r3.ok(3).unwrap();
        });

        let results = join_all(vec![t1, t2, t3]).await;

        // Results should still be in original order
        match &results[0] {
            TaskResult::Ok(v) => assert_eq!(*v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match &results[1] {
            TaskResult::Ok(v) => assert_eq!(*v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match &results[2] {
            TaskResult::Ok(v) => assert_eq!(*v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[tokio::test]
    async fn test_join_all_empty_vec() {
        let results: Vec<TaskResult<i32>> = join_all(vec![]).await;
        assert!(results.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_join_macro_two_tasks() {
        let (t1, r1) = new::<i32>();
        let (t2, r2) = new::<String>();

        std::thread::spawn(move || r1.ok(42).unwrap());
        std::thread::spawn(move || r2.ok("hello".to_string()).unwrap());

        let (res1, res2) = join!(t1, t2);

        match res1 {
            TaskResult::Ok(v) => assert_eq!(v, 42),
            _ => panic!("Expected Ok(42)"),
        }
        match res2 {
            TaskResult::Ok(v) => assert_eq!(v, "hello"),
            _ => panic!("Expected Ok(\"hello\")"),
        }
    }

    // ==================== Sync tests ====================

    #[test]
    fn test_wait_all_completes_all_tasks_sync() {
        let (t1, r1) = new::<i32>();
        let (t2, r2) = new::<i32>();
        let (t3, r3) = new::<i32>();

        std::thread::spawn(move || r1.ok(1).unwrap());
        std::thread::spawn(move || r2.ok(2).unwrap());
        std::thread::spawn(move || r3.ok(3).unwrap());

        let results = wait_all(vec![t1, t2, t3]);

        assert_eq!(results.len(), 3);
        match results[0].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match results[1].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match results[2].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[test]
    fn test_wait_all_preserves_order_sync() {
        let (t1, r1) = new::<i32>();
        let (t2, r2) = new::<i32>();
        let (t3, r3) = new::<i32>();

        // Complete in reverse order
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(30));
            r1.ok(1).unwrap();
        });
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            r2.ok(2).unwrap();
        });
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            r3.ok(3).unwrap();
        });

        let results = wait_all(vec![t1, t2, t3]);

        // Results should still be in original order
        match results[0].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match results[1].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match results[2].as_ref().unwrap() {
            TaskResult::Ok(v) => assert_eq!(*v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[test]
    fn test_wait_all_empty_vec_sync() {
        let results: Vec<Result<TaskResult<i32>, RecvError>> = wait_all(vec![]);
        assert!(results.is_empty());
    }
}
