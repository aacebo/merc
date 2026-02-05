use loom_sync::tasks::{Task, TaskError, TaskResult};

use crate::{Build, Operator, Pipe, Source};

/// Await: wait for a Task to complete and extract its result
pub struct Await;

impl Await {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Await {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Operator<Task<T>> for Await
where
    T: Send + 'static,
{
    type Output = TaskResult<T>;

    fn apply(self, src: Source<Task<T>>) -> Source<Self::Output> {
        Source::new(move || {
            let mut task = src.build();
            match task.wait() {
                Ok(result) => result,
                Err(recv_err) => TaskResult::Error(TaskError::from(recv_err)),
            }
        })
    }
}

pub trait AwaitPipe<T>: Pipe<Task<T>> + Sized
where
    T: Send + 'static,
{
    fn wait(self) -> Source<TaskResult<T>> {
        self.pipe(Await::new())
    }
}

impl<T: Send + 'static, P: Pipe<Task<T>> + Sized> AwaitPipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;
    use crate::operators::Spawn;

    #[test]
    fn waits_for_task() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let result = Source::from(5)
            .pipe(Spawn::new(|x| x * 3))
            .pipe(Await::new())
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 15);
    }

    #[test]
    fn with_chained_spawn() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let result = Source::from("test".to_string())
            .pipe(Spawn::new(|s: String| s.len()))
            .pipe(Await::new())
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
    }

    #[test]
    fn default_impl() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let result = Source::from(10)
            .pipe(Spawn::new(|x| x + 5))
            .pipe(Await::default())
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 15);
    }

    #[test]
    fn await_pipe_trait() {
        use super::AwaitPipe;
        use crate::operators::SpawnPipe;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let result = Source::from(5).spawn(|x| x * 3).wait().build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 15);
    }
}
