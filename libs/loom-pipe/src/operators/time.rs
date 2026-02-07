use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::{Build, Operator, Pipe, Source};

// ============================================================================
// TimeoutError
// ============================================================================

/// Error returned when an operation exceeds its timeout duration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutError {
    /// The duration that was exceeded
    pub duration: Duration,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out after {:?}", self.duration)
    }
}

impl std::error::Error for TimeoutError {}

// ============================================================================
// Timeout Operator
// ============================================================================

/// Timeout operator - fails with TimeoutError if operation exceeds duration
///
/// Transforms T -> Result<T, TimeoutError>
pub struct Timeout {
    duration: Duration,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl<T> Operator<T> for Timeout
where
    T: Send + 'static,
{
    type Output = Result<T, TimeoutError>;

    fn apply(self, src: Source<T>) -> Source<Self::Output> {
        let duration = self.duration;
        Source::new(move || {
            let (tx, rx) = mpsc::channel();

            thread::spawn(move || {
                let result = src.build();
                let _ = tx.send(result);
            });

            match rx.recv_timeout(duration) {
                Ok(result) => Ok(result),
                Err(_) => Err(TimeoutError { duration }),
            }
        })
    }
}

// ============================================================================
// Delay Operator
// ============================================================================

/// Delay operator - waits for specified duration before executing
///
/// Does not transform the type: T -> T
pub struct Delay {
    duration: Duration,
}

impl Delay {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl<T> Operator<T> for Delay
where
    T: 'static,
{
    type Output = T;

    fn apply(self, src: Source<T>) -> Source<Self::Output> {
        let duration = self.duration;
        Source::new(move || {
            thread::sleep(duration);
            src.build()
        })
    }
}

// ============================================================================
// Extension Trait
// ============================================================================

/// Extension trait for time-based operations on pipelines
pub trait TimePipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    /// Apply a timeout to this operation
    ///
    /// If the operation takes longer than the specified duration,
    /// returns Err(TimeoutError) instead of the result.
    fn timeout(self, duration: Duration) -> Source<Result<T, TimeoutError>> {
        self.pipe(Timeout::new(duration))
    }

    /// Delay execution by the specified duration
    ///
    /// The delay happens before the source is built.
    fn delay(self, duration: Duration) -> Source<T>
    where
        T: 'static,
    {
        self.pipe(Delay::new(duration))
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> TimePipe<T> for P {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Build;
    use std::time::Instant;

    #[test]
    fn timeout_succeeds_for_fast_operation() {
        let result = Source::from(42)
            .pipe(Timeout::new(Duration::from_secs(1)))
            .build();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn timeout_fails_for_slow_operation() {
        let result = Source::new(|| {
            thread::sleep(Duration::from_millis(200));
            42
        })
        .pipe(Timeout::new(Duration::from_millis(50)))
        .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.duration, Duration::from_millis(50));
    }

    #[test]
    fn timeout_error_display() {
        let err = TimeoutError {
            duration: Duration::from_secs(5),
        };
        assert_eq!(format!("{}", err), "operation timed out after 5s");
    }

    #[test]
    fn timeout_pipe_trait() {
        let result = Source::from(42).timeout(Duration::from_secs(1)).build();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn delay_waits_before_execution() {
        let start = Instant::now();
        let delay_duration = Duration::from_millis(100);

        let result = Source::from(42).pipe(Delay::new(delay_duration)).build();

        let elapsed = start.elapsed();
        assert!(elapsed >= delay_duration);
        assert_eq!(result, 42);
    }

    #[test]
    fn delay_preserves_value() {
        let result = Source::from("hello".to_string())
            .pipe(Delay::new(Duration::from_millis(10)))
            .build();

        assert_eq!(result, "hello");
    }

    #[test]
    fn delay_pipe_trait() {
        let start = Instant::now();
        let delay_duration = Duration::from_millis(50);

        let result = Source::from(42).delay(delay_duration).build();

        assert!(start.elapsed() >= delay_duration);
        assert_eq!(result, 42);
    }

    #[test]
    fn delay_then_timeout_succeeds() {
        let result = Source::from(42)
            .delay(Duration::from_millis(10))
            .timeout(Duration::from_secs(1))
            .build();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn timeout_includes_delay_time() {
        let result = Source::from(42)
            .delay(Duration::from_millis(200))
            .timeout(Duration::from_millis(50))
            .build();

        assert!(result.is_err());
    }
}
