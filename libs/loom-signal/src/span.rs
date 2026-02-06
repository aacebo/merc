use std::time::{Duration, Instant};

use loom_core::value::Value;

use crate::{Attributes, Level, Signal, Type};

/// A span represents a timed operation.
///
/// Create a span at the start of an operation and call `finish()` when done
/// to convert it to a Signal with duration information.
///
/// # Example
/// ```ignore
/// let span = Span::new("my.operation")
///     .with_level(Level::Debug)
///     .with_attr("input_size", 100);
///
/// // ... do work ...
///
/// emitter.emit(span.finish());
/// ```
pub struct Span {
    name: String,
    level: Level,
    attributes: Attributes,
    start_time: Instant,
}

impl Span {
    /// Create a new span with the given name.
    /// The start time is captured immediately.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: Level::Info,
            attributes: Attributes::new().build(),
            start_time: Instant::now(),
        }
    }

    /// Set the log level for this span.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Add an attribute to the span.
    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.attributes = Attributes::new()
            .merge(self.attributes)
            .attr(key, value)
            .build();
        self
    }

    /// Get the elapsed duration since the span was created.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the span name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Finish the span and convert it to a Signal.
    /// Adds `duration_ms` attribute automatically.
    pub fn finish(self) -> Signal {
        Signal::new()
            .otype(Type::Span)
            .level(self.level)
            .name(self.name)
            .attributes(self.attributes)
            .attr("duration_ms", self.start_time.elapsed().as_millis() as i64)
            .build()
    }

    /// Finish the span with an error.
    /// Sets level to Error and adds an `error` attribute.
    pub fn finish_with_error(self, error: impl Into<String>) -> Signal {
        Signal::new()
            .otype(Type::Span)
            .level(Level::Error)
            .name(self.name)
            .attributes(self.attributes)
            .attr("duration_ms", self.start_time.elapsed().as_millis() as i64)
            .attr("error", error.into())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("test.span");
        assert_eq!(span.name(), "test.span");
    }

    #[test]
    fn test_span_with_level() {
        let span = Span::new("test.span").with_level(Level::Debug);
        let signal = span.finish();
        assert_eq!(signal.level(), Level::Debug);
    }

    #[test]
    fn test_span_with_attr() {
        let span = Span::new("test.span").with_attr("key", "value");
        let signal = span.finish();
        assert!(signal.attributes().exists("key"));
    }

    #[test]
    fn test_span_finish_has_duration() {
        let span = Span::new("test.span");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let signal = span.finish();

        assert_eq!(signal.otype(), Type::Span);
        assert!(signal.attributes().exists("duration_ms"));
    }

    #[test]
    fn test_span_finish_with_error() {
        let span = Span::new("test.span");
        let signal = span.finish_with_error("something went wrong");

        assert_eq!(signal.level(), Level::Error);
        assert!(signal.attributes().exists("error"));
    }
}
