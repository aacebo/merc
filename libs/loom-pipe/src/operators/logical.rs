use crate::{Build, Operator, Pipe, Source};

/// And operator - validates a Result value, short-circuits on error
pub struct And<F> {
    validator: F,
}

impl<F> And<F> {
    pub fn new(validator: F) -> Self {
        Self { validator }
    }
}

impl<T, E, F> Operator<Result<T, E>> for And<F>
where
    T: Send + 'static,
    E: Send + 'static,
    F: FnOnce(&T) -> Result<(), E> + Send + 'static,
{
    type Output = Result<T, E>;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || {
            src.build().and_then(|value| {
                (self.validator)(&value)?;
                Ok(value)
            })
        })
    }
}

/// Or operator - provides fallback value on error
pub struct Or<F> {
    fallback: F,
}

impl<F> Or<F> {
    pub fn new(fallback: F) -> Self {
        Self { fallback }
    }
}

impl<T, E, F> Operator<Result<T, E>> for Or<F>
where
    T: Send + 'static,
    E: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap_or_else(|_| (self.fallback)()))
    }
}

/// OrElseMap operator - transforms error into success value
pub struct OrElseMap<F> {
    handler: F,
}

impl<F> OrElseMap<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<T, E, F> Operator<Result<T, E>> for OrElseMap<F>
where
    T: Send + 'static,
    E: Send + 'static,
    F: FnOnce(E) -> T + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || match src.build() {
            Ok(v) => v,
            Err(e) => (self.handler)(e),
        })
    }
}

/// Extension trait for logical operators on Result types
pub trait LogicalPipe<T, E>: Pipe<Result<T, E>> + Sized
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Validate the Ok value, short-circuit with Err if validation fails
    fn and<F>(self, validator: F) -> Source<Result<T, E>>
    where
        F: FnOnce(&T) -> Result<(), E> + Send + 'static,
    {
        self.pipe(And::new(validator))
    }

    /// Provide fallback value on error
    fn or<F>(self, fallback: F) -> Source<T>
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.pipe(Or::new(fallback))
    }

    /// Transform error into success value
    fn or_else_map<F>(self, handler: F) -> Source<T>
    where
        F: FnOnce(E) -> T + Send + 'static,
    {
        self.pipe(OrElseMap::new(handler))
    }
}

impl<T, E, P> LogicalPipe<T, E> for P
where
    T: Send + 'static,
    E: Send + 'static,
    P: Pipe<Result<T, E>> + Sized,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_passes_valid() {
        let result: Result<i32, &str> = Source::from(Ok(10))
            .and(|x| {
                if *x > 0 {
                    Ok(())
                } else {
                    Err("must be positive")
                }
            })
            .build();

        assert_eq!(result, Ok(10));
    }

    #[test]
    fn and_fails_invalid() {
        let result: Result<i32, &str> = Source::from(Ok(-5))
            .and(|x| {
                if *x > 0 {
                    Ok(())
                } else {
                    Err("must be positive")
                }
            })
            .build();

        assert_eq!(result, Err("must be positive"));
    }

    #[test]
    fn and_passes_through_error() {
        let result: Result<i32, &str> = Source::from(Err("already error"))
            .and(|_: &i32| Ok(()))
            .build();

        assert_eq!(result, Err("already error"));
    }

    #[test]
    fn or_uses_ok_value() {
        let result = Source::from(Ok::<i32, &str>(10)).or(|| 0).build();

        assert_eq!(result, 10);
    }

    #[test]
    fn or_uses_fallback_on_error() {
        let result = Source::from(Err::<i32, &str>("error")).or(|| 42).build();

        assert_eq!(result, 42);
    }

    #[test]
    fn or_else_map_uses_ok_value() {
        let result = Source::from(Ok::<i32, i32>(10))
            .or_else_map(|e| e * 2)
            .build();

        assert_eq!(result, 10);
    }

    #[test]
    fn or_else_map_transforms_error() {
        let result = Source::from(Err::<i32, i32>(21))
            .or_else_map(|e| e * 2)
            .build();

        assert_eq!(result, 42);
    }

    #[test]
    fn chained_and_operators() {
        let result: Result<i32, &str> = Source::from(Ok(10))
            .and(|x| {
                if *x > 0 {
                    Ok(())
                } else {
                    Err("must be positive")
                }
            })
            .and(|x| {
                if *x < 100 {
                    Ok(())
                } else {
                    Err("must be less than 100")
                }
            })
            .build();

        assert_eq!(result, Ok(10));
    }

    #[test]
    fn chained_and_fails_on_second() {
        let result: Result<i32, &str> = Source::from(Ok(150))
            .and(|x| {
                if *x > 0 {
                    Ok(())
                } else {
                    Err("must be positive")
                }
            })
            .and(|x| {
                if *x < 100 {
                    Ok(())
                } else {
                    Err("must be less than 100")
                }
            })
            .build();

        assert_eq!(result, Err("must be less than 100"));
    }
}
