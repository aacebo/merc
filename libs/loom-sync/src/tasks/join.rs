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
        ::futures::join!($($task),+)
    };
}

/// Blocking wait for multiple tasks concurrently using threads.
///
/// # Example
/// ```ignore
/// let r1 = wait!(task1);
/// let (r1, r2) = wait!(task1, task2);
/// let (r1, r2, r3) = wait!(task1, task2, task3);
/// ```
#[macro_export]
macro_rules! wait {
    // Single task - direct wait
    ($t1:expr) => {{
        let mut t = $t1;
        t.wait()
    }};
    // Two tasks
    ($t1:expr, $t2:expr) => {{
        let (mut t1, mut t2) = ($t1, $t2);
        let h1 = ::std::thread::spawn(move || t1.wait());
        let h2 = ::std::thread::spawn(move || t2.wait());
        (
            h1.join().expect("task panicked"),
            h2.join().expect("task panicked"),
        )
    }};
    // Three tasks
    ($t1:expr, $t2:expr, $t3:expr) => {{
        let (mut t1, mut t2, mut t3) = ($t1, $t2, $t3);
        let h1 = ::std::thread::spawn(move || t1.wait());
        let h2 = ::std::thread::spawn(move || t2.wait());
        let h3 = ::std::thread::spawn(move || t3.wait());
        (
            h1.join().expect("task panicked"),
            h2.join().expect("task panicked"),
            h3.join().expect("task panicked"),
        )
    }};
}

#[cfg(all(test, feature = "tokio"))]
mod tests {
    use crate::spawn;
    use crate::tasks::{Task, TaskResult};

    // ==================== Async tests ====================

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_join_completes_all_tasks() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<i32>, _) = spawn!();
        let (t3, r3): (Task<i32>, _) = spawn!();

        // Complete tasks from separate threads
        std::thread::spawn(move || r1.ok(1).unwrap());
        std::thread::spawn(move || r2.ok(2).unwrap());
        std::thread::spawn(move || r3.ok(3).unwrap());

        let (res1, res2, res3) = join!(t1, t2, t3);

        match res1 {
            TaskResult::Ok(v) => assert_eq!(v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match res2 {
            TaskResult::Ok(v) => assert_eq!(v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match res3 {
            TaskResult::Ok(v) => assert_eq!(v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_join_preserves_order() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<i32>, _) = spawn!();
        let (t3, r3): (Task<i32>, _) = spawn!();

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

        let (res1, res2, res3) = join!(t1, t2, t3);

        // Results should still be in original order
        match res1 {
            TaskResult::Ok(v) => assert_eq!(v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match res2 {
            TaskResult::Ok(v) => assert_eq!(v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match res3 {
            TaskResult::Ok(v) => assert_eq!(v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_join_two_tasks() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<String>, _) = spawn!();

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
    fn test_wait_single_task() {
        let (t1, r1): (Task<i32>, _) = spawn!();

        std::thread::spawn(move || r1.ok(42).unwrap());

        let res = wait!(t1);
        match res.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 42),
            _ => panic!("Expected Ok(42)"),
        }
    }

    #[test]
    fn test_wait_two_tasks() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<i32>, _) = spawn!();

        std::thread::spawn(move || r1.ok(1).unwrap());
        std::thread::spawn(move || r2.ok(2).unwrap());

        let (res1, res2) = wait!(t1, t2);

        match res1.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match res2.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 2),
            _ => panic!("Expected Ok(2)"),
        }
    }

    #[test]
    fn test_wait_three_tasks() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<i32>, _) = spawn!();
        let (t3, r3): (Task<i32>, _) = spawn!();

        std::thread::spawn(move || r1.ok(1).unwrap());
        std::thread::spawn(move || r2.ok(2).unwrap());
        std::thread::spawn(move || r3.ok(3).unwrap());

        let (res1, res2, res3) = wait!(t1, t2, t3);

        match res1.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match res2.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match res3.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }

    #[test]
    fn test_wait_preserves_order() {
        let (t1, r1): (Task<i32>, _) = spawn!();
        let (t2, r2): (Task<i32>, _) = spawn!();
        let (t3, r3): (Task<i32>, _) = spawn!();

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

        let (res1, res2, res3) = wait!(t1, t2, t3);

        // Results should still be in original order
        match res1.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 1),
            _ => panic!("Expected Ok(1)"),
        }
        match res2.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 2),
            _ => panic!("Expected Ok(2)"),
        }
        match res3.unwrap() {
            TaskResult::Ok(v) => assert_eq!(v, 3),
            _ => panic!("Expected Ok(3)"),
        }
    }
}
