use std::sync::Arc;

use loom_core::Map;
use loom_io::DataSource;
use loom_pipe::LayerContext;
use loom_signal::Signal;

use crate::Runtime;
use crate::eval::Sample;

/// Single-item context bound to runtime (internal).
pub struct Context<Input> {
    runtime: Option<Arc<Runtime>>,
    pub meta: Map,
    pub step: usize,
    pub text: String,
    pub input: Input,
}

impl<Input> Context<Input> {
    /// Create a new context without a runtime reference.
    /// Used for standalone layer processing.
    pub fn new(text: &str, input: Input) -> Self {
        Self {
            runtime: None,
            meta: Map::default(),
            step: 0,
            text: text.to_string(),
            input,
        }
    }

    /// Emit a signal through the runtime's emitter.
    /// No-op if context was created without a runtime.
    pub fn emit(&self, signal: Signal) {
        if let Some(ref runtime) = self.runtime {
            runtime.emit(signal);
        }
    }

    /// Get a data source by name from the runtime.
    /// Returns None if context was created without a runtime.
    pub fn data_source(&self, name: &str) -> Option<&dyn DataSource> {
        self.runtime.as_ref().and_then(|rt| rt.sources().get(name))
    }

    /// Check if this context has a runtime reference.
    pub fn has_runtime(&self) -> bool {
        self.runtime.is_some()
    }
}

impl<Input: Send + 'static> LayerContext for Context<Input> {
    fn text(&self) -> &str {
        &self.text
    }

    fn step(&self) -> usize {
        self.step
    }

    fn meta(&self) -> &Map {
        &self.meta
    }
}

/// Batch context for processing multiple samples (internal).
pub struct BatchContext {
    runtime: Option<Arc<Runtime>>,
    samples: Vec<Sample>,
    step: usize,
    meta: Map,
}

impl BatchContext {
    /// Create a new batch context without a runtime reference.
    pub fn new(samples: Vec<Sample>) -> Self {
        Self {
            runtime: None,
            samples,
            step: 0,
            meta: Map::default(),
        }
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Emit a signal through the runtime's emitter.
    /// No-op if context was created without a runtime.
    pub fn emit(&self, signal: Signal) {
        if let Some(ref runtime) = self.runtime {
            runtime.emit(signal);
        }
    }

    /// Get a data source by name from the runtime.
    /// Returns None if context was created without a runtime.
    pub fn data_source(&self, name: &str) -> Option<&dyn DataSource> {
        self.runtime.as_ref().and_then(|rt| rt.sources().get(name))
    }
}

impl LayerContext for BatchContext {
    fn text(&self) -> &str {
        self.samples.first().map(|s| s.text.as_str()).unwrap_or("")
    }

    fn step(&self) -> usize {
        self.step
    }

    fn meta(&self) -> &Map {
        &self.meta
    }
}
