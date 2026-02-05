use loom_error::Result;

use crate::{Build, Operator, Pipe, Source};

/// TryMap: transform input with a fallible function
pub struct TryMap<Input, Output> {
    f: Box<dyn FnOnce(Input) -> Result<Output> + Send>,
}

impl<Input, Output> TryMap<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Input) -> Result<Output> + Send + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl<Input, Output> Operator<Input> for TryMap<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    type Output = Result<Output>;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || (self.f)(src.build()))
    }
}

pub trait TryMapPipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    fn try_map<O, F>(self, f: F) -> Source<Result<O>>
    where
        O: Send + 'static,
        F: FnOnce(T) -> Result<O> + Send + 'static,
    {
        self.pipe(TryMap::new(f))
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> TryMapPipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn success() {
        let result = Source::from("42".to_string())
            .pipe(TryMap::new(|s: String| {
                s.parse::<i32>().map_err(|e| e.into())
            }))
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn failure() {
        let result = Source::from("not_a_number".to_string())
            .pipe(TryMap::new(|s: String| {
                s.parse::<i32>().map_err(|e| e.into())
            }))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn with_custom_error() {
        let result = Source::from(10)
            .pipe(TryMap::new(|x: i32| {
                if x > 5 {
                    Ok(x * 2)
                } else {
                    Err(loom_error::Error::builder()
                        .message("value too small")
                        .build())
                }
            }))
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);
    }

    #[test]
    fn with_custom_error_failure() {
        let result = Source::from(3)
            .pipe(TryMap::new(|x: i32| {
                if x > 5 {
                    Ok(x * 2)
                } else {
                    Err(loom_error::Error::builder()
                        .message("value too small")
                        .build())
                }
            }))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn try_map_pipe_trait() {
        let result = Source::from("42".to_string())
            .try_map(|s: String| s.parse::<i32>().map_err(|e| e.into()))
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
