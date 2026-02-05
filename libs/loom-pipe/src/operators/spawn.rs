use loom_sync::tasks::Task;

use crate::{Build, Operator, Pipe, Source};

/// Spawn: execute work asynchronously, return a Task handle
pub struct Spawn<Input, Output> {
    f: Box<dyn FnOnce(Input) -> Output + Send>,
}

impl<Input, Output> Spawn<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Input) -> Output + Send + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl<Input, Output> Operator<Input> for Spawn<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    type Output = Task<Output>;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();
            let f = self.f;
            loom_sync::spawn!(|| f(input))
        })
    }
}

pub trait SpawnPipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    fn spawn<O, F>(self, f: F) -> Source<Task<O>>
    where
        O: Send + 'static,
        F: FnOnce(T) -> O + Send + 'static,
    {
        self.pipe(Spawn::new(f))
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> SpawnPipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn executes_work() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let mut task = Source::from(5).pipe(Spawn::new(|x| x * 2)).build();
        let result = task.wait().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn with_string() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let mut task = Source::from("hello".to_string())
            .pipe(Spawn::new(|s: String| s.to_uppercase()))
            .build();
        let result = task.wait().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO");
    }

    #[test]
    fn with_computation() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let mut task = Source::from(10)
            .pipe(Spawn::new(|x: i32| (0..x).sum::<i32>()))
            .build();
        let result = task.wait().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 45); // 0+1+2+...+9 = 45
    }

    #[test]
    fn spawn_pipe_trait() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let mut task = Source::from(5).spawn(|x| x * 2).build();
        let result = task.wait().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }
}
