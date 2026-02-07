use crate::{Build, Operator, Pipe, Source};

pub struct Map<Input, Output> {
    handler: Box<dyn FnOnce(Input) -> Output + Send>,
}

impl<Input, Output> Map<Input, Output> {
    pub fn new<Handler: FnOnce(Input) -> Output + Send + 'static>(handler: Handler) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl<Input: Send + 'static, Output: Send + 'static> Operator<Input> for Map<Input, Output> {
    type Output = Output;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(|| (self.handler)(src.build()))
    }
}

pub trait MapPipe<T>: Pipe<T> + Sized {
    fn map<O: Send + 'static, F: FnOnce(T) -> O + Send + 'static>(self, f: F) -> Source<O>
    where
        T: Send + 'static,
    {
        self.pipe(Map::new(f))
    }
}

impl<T, P: Pipe<T> + Sized> MapPipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn transforms_value() {
        let result = Source::from(5).pipe(Map::new(|x| x * 2)).build();
        assert_eq!(result, 10);
    }

    #[test]
    fn changes_type() {
        let result = Source::from(42)
            .pipe(Map::new(|x: i32| x.to_string()))
            .build();
        assert_eq!(result, "42");
    }

    #[test]
    fn with_closure() {
        let multiplier = 3;
        let result = Source::from(7)
            .pipe(Map::new(move |x| x * multiplier))
            .build();
        assert_eq!(result, 21);
    }

    #[test]
    fn chained() {
        let result = Source::from(2)
            .map(|x| x + 1)
            .map(|x| x * 2)
            .map(|x| x.to_string())
            .build();
        assert_eq!(result, "6");
    }

    #[test]
    fn map_pipe_trait() {
        let result = Source::from(10).map(|x| x * 3).build();
        assert_eq!(result, 30);
    }
}
