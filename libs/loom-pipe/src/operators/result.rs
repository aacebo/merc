use std::marker::PhantomData;
use std::time::Duration;

use crate::{Build, Operator, Pipe, Source};

// ============================================================================
// Retry Operator with Builder
// ============================================================================

/// Retry operator - retries a fallible operation with configurable backoff
pub struct Retry<Input, Output, E, F>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
    E: Send + 'static,
{
    operation: F,
    max_attempts: usize,
    initial_delay: Duration,
    backoff_multiplier: f64,
    _marker: PhantomData<fn(Input) -> Result<Output, E>>,
}

impl<Input, Output, E, F> Retry<Input, Output, E, F>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
    E: Send + 'static,
    F: Fn(Input) -> Result<Output, E> + Send + 'static,
{
    pub fn new(
        operation: F,
        max_attempts: usize,
        initial_delay: Duration,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            operation,
            max_attempts,
            initial_delay,
            backoff_multiplier,
            _marker: PhantomData,
        }
    }
}

impl<Input, Output, E, F> Operator<Input> for Retry<Input, Output, E, F>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
    E: Send + 'static,
    F: Fn(Input) -> Result<Output, E> + Send + 'static,
{
    type Output = Result<Output, E>;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();
            let mut attempts = 0;
            let mut delay = self.initial_delay;

            loop {
                match (self.operation)(input.clone()) {
                    Ok(v) => return Ok(v),
                    Err(_) if attempts < self.max_attempts => {
                        attempts += 1;
                        std::thread::sleep(delay);
                        delay =
                            Duration::from_secs_f64(delay.as_secs_f64() * self.backoff_multiplier);
                    }
                    Err(e) => return Err(e),
                }
            }
        })
    }
}

/// Extension trait for retry operations
pub trait RetryPipe<T>: Pipe<T> + Sized
where
    T: Clone + Send + 'static,
{
    fn retry<O, E>(self) -> RetryBuilder<T, O, E, Self>
    where
        O: Send + 'static,
        E: Send + 'static,
    {
        RetryBuilder::new(self)
    }
}

impl<T: Clone + Send + 'static, P: Pipe<T> + Sized> RetryPipe<T> for P {}

/// Builder for retry operations
pub struct RetryBuilder<Input, Output, E, P> {
    source: P,
    max_attempts: usize,
    initial_delay: Duration,
    backoff_multiplier: f64,
    _marker: PhantomData<(Input, Output, E)>,
}

impl<Input, Output, E, P> RetryBuilder<Input, Output, E, P>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
    E: Send + 'static,
    P: Pipe<Input>,
{
    fn new(source: P) -> Self {
        Self {
            source,
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            _marker: PhantomData,
        }
    }

    /// Set maximum number of retry attempts (default: 3)
    pub fn attempts(mut self, n: usize) -> Self {
        self.max_attempts = n;
        self
    }

    /// Set initial delay between retries (default: 100ms)
    pub fn delay(mut self, d: Duration) -> Self {
        self.initial_delay = d;
        self
    }

    /// Set backoff multiplier (default: 2.0)
    pub fn backoff(mut self, m: f64) -> Self {
        self.backoff_multiplier = m;
        self
    }

    /// Run the operation with retry logic
    pub fn run<F>(self, operation: F) -> Source<Result<Output, E>>
    where
        F: Fn(Input) -> Result<Output, E> + Send + 'static,
    {
        self.source.pipe(Retry::new(
            operation,
            self.max_attempts,
            self.initial_delay,
            self.backoff_multiplier,
        ))
    }
}

// ============================================================================
// Result Unwrap Operators
// ============================================================================

/// Unwrap operator - panics on Err with default message
pub struct Unwrap;

impl<T, E> Operator<Result<T, E>> for Unwrap
where
    T: Send + 'static,
    E: std::fmt::Debug + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap())
    }
}

/// Expect operator - panics on Err with custom message
pub struct Expect {
    message: &'static str,
}

impl Expect {
    pub fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl<T, E> Operator<Result<T, E>> for Expect
where
    T: Send + 'static,
    E: std::fmt::Debug + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().expect(self.message))
    }
}

/// UnwrapOr operator - provides default value on Err
pub struct UnwrapOr<T> {
    default: T,
}

impl<T> UnwrapOr<T> {
    pub fn new(default: T) -> Self {
        Self { default }
    }
}

impl<T, E> Operator<Result<T, E>> for UnwrapOr<T>
where
    T: Send + 'static,
    E: Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap_or(self.default))
    }
}

/// UnwrapOrElse operator - provides default via closure on Err
pub struct UnwrapOrElse<F> {
    default_fn: F,
}

impl<F> UnwrapOrElse<F> {
    pub fn new(default_fn: F) -> Self {
        Self { default_fn }
    }
}

impl<T, E, F> Operator<Result<T, E>> for UnwrapOrElse<F>
where
    T: Send + 'static,
    E: Send + 'static,
    F: FnOnce(E) -> T + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap_or_else(self.default_fn))
    }
}

/// ResultOk operator - converts Result<T, E> to Option<T>
pub struct ResultOk;

impl<T, E> Operator<Result<T, E>> for ResultOk
where
    T: Send + 'static,
    E: Send + 'static,
{
    type Output = Option<T>;

    fn apply(self, src: Source<Result<T, E>>) -> Source<Self::Output> {
        Source::new(move || src.build().ok())
    }
}

/// Extension trait for Result operators
pub trait ResultPipe<T, E>: Pipe<Result<T, E>> + Sized
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Unwrap the Result, panicking on Err
    fn unwrap(self) -> Source<T>
    where
        E: std::fmt::Debug,
    {
        self.pipe(Unwrap)
    }

    /// Unwrap the Result, panicking with message on Err
    fn expect(self, message: &'static str) -> Source<T>
    where
        E: std::fmt::Debug,
    {
        self.pipe(Expect::new(message))
    }

    /// Unwrap the Result, using default on Err
    fn unwrap_or(self, default: T) -> Source<T> {
        self.pipe(UnwrapOr::new(default))
    }

    /// Unwrap the Result, computing default from error on Err
    fn unwrap_or_else<F>(self, f: F) -> Source<T>
    where
        F: FnOnce(E) -> T + Send + 'static,
    {
        self.pipe(UnwrapOrElse::new(f))
    }

    /// Convert Result<T, E> to Option<T>, discarding the error
    fn ok(self) -> Source<Option<T>> {
        self.pipe(ResultOk)
    }
}

impl<T, E, P> ResultPipe<T, E> for P
where
    T: Send + 'static,
    E: Send + 'static,
    P: Pipe<Result<T, E>> + Sized,
{
}

// ============================================================================
// Option Unwrap Operators
// ============================================================================

/// OptionUnwrap operator - panics on None with default message
pub struct OptionUnwrap;

impl<T> Operator<Option<T>> for OptionUnwrap
where
    T: Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Option<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap())
    }
}

/// OptionExpect operator - panics on None with custom message
pub struct OptionExpect {
    message: &'static str,
}

impl OptionExpect {
    pub fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl<T> Operator<Option<T>> for OptionExpect
where
    T: Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Option<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().expect(self.message))
    }
}

/// OptionUnwrapOr operator - provides default value on None
pub struct OptionUnwrapOr<T> {
    default: T,
}

impl<T> OptionUnwrapOr<T> {
    pub fn new(default: T) -> Self {
        Self { default }
    }
}

impl<T> Operator<Option<T>> for OptionUnwrapOr<T>
where
    T: Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Option<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap_or(self.default))
    }
}

/// OptionUnwrapOrElse operator - provides default via closure on None
pub struct OptionUnwrapOrElse<F> {
    default_fn: F,
}

impl<F> OptionUnwrapOrElse<F> {
    pub fn new(default_fn: F) -> Self {
        Self { default_fn }
    }
}

impl<T, F> Operator<Option<T>> for OptionUnwrapOrElse<F>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    type Output = T;

    fn apply(self, src: Source<Option<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().unwrap_or_else(self.default_fn))
    }
}

/// OptionOkOr operator - converts Option<T> to Result<T, E>
pub struct OptionOkOr<E> {
    error: E,
}

impl<E> OptionOkOr<E> {
    pub fn new(error: E) -> Self {
        Self { error }
    }
}

impl<T, E> Operator<Option<T>> for OptionOkOr<E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    type Output = Result<T, E>;

    fn apply(self, src: Source<Option<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().ok_or(self.error))
    }
}

/// Extension trait for Option operators
pub trait OptionPipe<T>: Pipe<Option<T>> + Sized
where
    T: Send + 'static,
{
    /// Unwrap the Option, panicking on None
    fn unwrap(self) -> Source<T> {
        self.pipe(OptionUnwrap)
    }

    /// Unwrap the Option, panicking with message on None
    fn expect(self, message: &'static str) -> Source<T> {
        self.pipe(OptionExpect::new(message))
    }

    /// Unwrap the Option, using default on None
    fn unwrap_or(self, default: T) -> Source<T> {
        self.pipe(OptionUnwrapOr::new(default))
    }

    /// Unwrap the Option, computing default on None
    fn unwrap_or_else<F>(self, f: F) -> Source<T>
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.pipe(OptionUnwrapOrElse::new(f))
    }

    /// Convert Option<T> to Result<T, E>
    fn ok_or<E>(self, error: E) -> Source<Result<T, E>>
    where
        E: Send + 'static,
    {
        self.pipe(OptionOkOr::new(error))
    }
}

impl<T, P> OptionPipe<T> for P
where
    T: Send + 'static,
    P: Pipe<Option<T>> + Sized,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Retry tests

    #[test]
    fn retry_succeeds_first_try() {
        let result: Result<i32, &str> = Source::from(10)
            .retry()
            .attempts(3)
            .run(|x| Ok(x * 2))
            .build();

        assert_eq!(result, Ok(20));
    }

    #[test]
    fn retry_succeeds_after_failures() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result: Result<i32, &str> = Source::from(10)
            .retry()
            .attempts(3)
            .delay(Duration::from_millis(1))
            .run(move |x| {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 { Err("not yet") } else { Ok(x * 2) }
            })
            .build();

        assert_eq!(result, Ok(20));
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }

    #[test]
    fn retry_exhausts_attempts() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result: Result<i32, &str> = Source::from(10)
            .retry()
            .attempts(2)
            .delay(Duration::from_millis(1))
            .run(move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err("always fails")
            })
            .build();

        assert_eq!(result, Err("always fails"));
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }

    // Result unwrap tests

    #[test]
    fn result_unwrap_ok() {
        let result = Source::from(Ok::<i32, &str>(42)).unwrap().build();
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic]
    fn result_unwrap_err_panics() {
        let _ = Source::from(Err::<i32, &str>("error")).unwrap().build();
    }

    #[test]
    fn result_expect_ok() {
        let result = Source::from(Ok::<i32, &str>(42))
            .expect("should not fail")
            .build();
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "custom message")]
    fn result_expect_err_panics_with_message() {
        let _ = Source::from(Err::<i32, &str>("error"))
            .expect("custom message")
            .build();
    }

    #[test]
    fn result_unwrap_or_ok() {
        let result = Source::from(Ok::<i32, &str>(42)).unwrap_or(0).build();
        assert_eq!(result, 42);
    }

    #[test]
    fn result_unwrap_or_err() {
        let result = Source::from(Err::<i32, &str>("error")).unwrap_or(0).build();
        assert_eq!(result, 0);
    }

    #[test]
    fn result_unwrap_or_else_ok() {
        let result = Source::from(Ok::<i32, &str>(42))
            .unwrap_or_else(|_| 0)
            .build();
        assert_eq!(result, 42);
    }

    #[test]
    fn result_unwrap_or_else_err() {
        let result = Source::from(Err::<i32, &str>("error"))
            .unwrap_or_else(|e| e.len() as i32)
            .build();
        assert_eq!(result, 5); // "error".len()
    }

    #[test]
    fn result_ok_some() {
        let result = Source::from(Ok::<i32, &str>(42)).ok().build();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn result_ok_none() {
        let result = Source::from(Err::<i32, &str>("error")).ok().build();
        assert_eq!(result, None);
    }

    // Option unwrap tests

    #[test]
    fn option_unwrap_some() {
        let result = Source::from(Some(42)).unwrap().build();
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic]
    fn option_unwrap_none_panics() {
        let _ = Source::from(None::<i32>).unwrap().build();
    }

    #[test]
    fn option_expect_some() {
        let result = Source::from(Some(42)).expect("should not fail").build();
        assert_eq!(result, 42);
    }

    #[test]
    fn option_unwrap_or_some() {
        let result = Source::from(Some(42)).unwrap_or(0).build();
        assert_eq!(result, 42);
    }

    #[test]
    fn option_unwrap_or_none() {
        let result = Source::from(None::<i32>).unwrap_or(0).build();
        assert_eq!(result, 0);
    }

    #[test]
    fn option_unwrap_or_else_some() {
        let result = Source::from(Some(42)).unwrap_or_else(|| 0).build();
        assert_eq!(result, 42);
    }

    #[test]
    fn option_unwrap_or_else_none() {
        let result = Source::from(None::<i32>).unwrap_or_else(|| 100).build();
        assert_eq!(result, 100);
    }

    #[test]
    fn option_ok_or_some() {
        let result: Result<i32, &str> = Source::from(Some(42)).ok_or("missing").build();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn option_ok_or_none() {
        let result: Result<i32, &str> = Source::from(None::<i32>).ok_or("missing").build();
        assert_eq!(result, Err("missing"));
    }
}
