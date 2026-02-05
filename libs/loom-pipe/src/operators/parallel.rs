use loom_sync::tasks::{Task, TaskError, TaskResult};

use crate::{Build, Operator, Pipe, Source};

/// Parallel: execute multiple operators concurrently using tasks
/// Unlike FanOut which executes sequentially, Parallel spawns tasks for each branch
pub struct Parallel<Input, Output> {
    branches: Vec<Box<dyn FnOnce(Input) -> Output + Send>>,
    _marker: std::marker::PhantomData<fn(Input) -> Output>,
}

impl<Input, Output> Parallel<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Input) -> Output + Send + 'static,
    {
        self.branches.push(Box::new(f));
        self
    }
}

impl<Input, Output> Default for Parallel<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output> Operator<Input> for Parallel<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    type Output = Vec<TaskResult<Output>>;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();

            // Spawn all branches as tasks
            let tasks: Vec<Task<Output>> = self
                .branches
                .into_iter()
                .map(|f| {
                    let cloned = input.clone();
                    loom_sync::spawn!(|| f(cloned))
                })
                .collect();

            // Wait for all tasks to complete
            tasks
                .into_iter()
                .map(|mut t| match t.wait() {
                    Ok(result) => result,
                    Err(recv_err) => TaskResult::Error(TaskError::from(recv_err)),
                })
                .collect()
        })
    }
}

/// Extension trait for starting parallel execution
pub trait ParallelPipe<T>: Pipe<T> + Sized
where
    T: Clone + Send + 'static,
{
    fn parallel<O: Send + 'static>(self) -> ParallelBuilder<T, O, Self> {
        ParallelBuilder::new(self)
    }
}

impl<T: Clone + Send + 'static, P: Pipe<T> + Sized> ParallelPipe<T> for P {}

/// Builder for parallel execution that implements Build and Pipe
pub struct ParallelBuilder<T, O, P> {
    source: P,
    parallel: Parallel<T, O>,
}

impl<T, O, P> ParallelBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    fn new(source: P) -> Self {
        Self {
            source,
            parallel: Parallel::new(),
        }
    }

    /// Spawn a new parallel branch
    pub fn spawn<F>(mut self, f: F) -> Self
    where
        F: FnOnce(T) -> O + Send + 'static,
    {
        self.parallel = self.parallel.add(f);
        self
    }
}

impl<T, O, P> Build for ParallelBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    type Output = Vec<TaskResult<O>>;

    fn build(self) -> Self::Output {
        self.source.pipe(self.parallel).build()
    }
}

impl<T, O, P> Pipe<Vec<TaskResult<O>>> for ParallelBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    fn pipe<Op: Operator<Vec<TaskResult<O>>>>(self, op: Op) -> Source<Op::Output> {
        self.source.pipe(self.parallel).pipe(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn executes_all_branches() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let results = Source::from(10)
            .pipe(Parallel::new().add(|x| x * 2).add(|x| x + 5).add(|x| x - 3))
            .build();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));

        let values: Vec<i32> = results.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![20, 15, 7]);
    }

    #[test]
    fn collects_all_results() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let results = Source::from(5)
            .pipe(Parallel::new().add(|x| x * 2).add(|x| x * 3))
            .build();

        let values: Vec<i32> = results.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![10, 15]);
    }

    #[test]
    fn no_branches() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let results = Source::from(42).pipe(Parallel::<i32, i32>::new()).build();
        assert!(results.is_empty());
    }

    #[test]
    fn default_is_empty() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let parallel: Parallel<i32, i32> = Parallel::default();
        let results = Source::from(42).pipe(parallel).build();
        assert!(results.is_empty());
    }

    #[test]
    fn with_strings() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let results = Source::from("hello".to_string())
            .pipe(
                Parallel::new()
                    .add(|s: String| s.to_uppercase())
                    .add(|s: String| s.len().to_string()),
            )
            .build();

        assert_eq!(results.len(), 2);
        let values: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(values[0], "HELLO");
        assert_eq!(values[1], "5");
    }

    #[test]
    fn concurrent_execution() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        use std::time::Duration;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let results = Source::from(())
            .pipe(
                Parallel::new()
                    .add(move |_| {
                        thread::sleep(Duration::from_millis(10));
                        c1.fetch_add(1, Ordering::SeqCst);
                        1
                    })
                    .add(move |_| {
                        thread::sleep(Duration::from_millis(10));
                        c2.fetch_add(1, Ordering::SeqCst);
                        2
                    })
                    .add(move |_| {
                        thread::sleep(Duration::from_millis(10));
                        c3.fetch_add(1, Ordering::SeqCst);
                        3
                    }),
            )
            .build();

        assert_eq!(results.len(), 3);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn parallel_pipe_trait() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let results = Source::from(10)
            .parallel()
            .spawn(|x| x * 2)
            .spawn(|x| x + 5)
            .build();

        assert_eq!(results.len(), 2);
        let values: Vec<i32> = results.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![20, 15]);
    }
}
