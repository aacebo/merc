use std::sync::{Arc, Mutex};

use crate::{Emitter, Signal};

/// An emitter that collects signals in memory.
///
/// Useful for testing and inspection. Signals can be retrieved
/// after emission for verification.
///
/// # Example
/// ```ignore
/// let emitter = MemoryEmitter::new();
///
/// emitter.emit(signal1);
/// emitter.emit(signal2);
///
/// assert_eq!(emitter.len(), 2);
/// let signals = emitter.signals();
/// ```
#[derive(Clone)]
pub struct MemoryEmitter {
    signals: Arc<Mutex<Vec<Signal>>>,
    capacity: Option<usize>,
}

impl MemoryEmitter {
    /// Create a new memory emitter with unlimited capacity.
    pub fn new() -> Self {
        Self {
            signals: Arc::new(Mutex::new(Vec::new())),
            capacity: None,
        }
    }

    /// Set a maximum capacity. When reached, oldest signals are removed
    /// (ring buffer behavior).
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Get a copy of all collected signals.
    pub fn signals(&self) -> Vec<Signal> {
        self.signals.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get the number of collected signals.
    pub fn len(&self) -> usize {
        self.signals.lock().map(|s| s.len()).unwrap_or(0)
    }

    /// Check if there are no signals.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all collected signals.
    pub fn clear(&self) {
        if let Ok(mut signals) = self.signals.lock() {
            signals.clear();
        }
    }

    /// Get the last emitted signal, if any.
    pub fn last(&self) -> Option<Signal> {
        self.signals.lock().ok().and_then(|s| s.last().cloned())
    }

    /// Find signals by name.
    pub fn find_by_name(&self, name: &str) -> Vec<Signal> {
        self.signals()
            .into_iter()
            .filter(|s| s.name() == name)
            .collect()
    }
}

impl Default for MemoryEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for MemoryEmitter {
    fn emit(&self, signal: Signal) {
        if let Ok(mut signals) = self.signals.lock() {
            if let Some(cap) = self.capacity {
                if signals.len() >= cap {
                    signals.remove(0); // Ring buffer behavior
                }
            }
            signals.push(signal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_emitter_new() {
        let emitter = MemoryEmitter::new();
        assert!(emitter.is_empty());
        assert_eq!(emitter.len(), 0);
    }

    #[test]
    fn test_memory_emitter_emit() {
        let emitter = MemoryEmitter::new();

        emitter.emit(Signal::new().name("test1").build());
        emitter.emit(Signal::new().name("test2").build());

        assert_eq!(emitter.len(), 2);
        assert!(!emitter.is_empty());
    }

    #[test]
    fn test_memory_emitter_signals() {
        let emitter = MemoryEmitter::new();

        emitter.emit(Signal::new().name("test1").build());
        emitter.emit(Signal::new().name("test2").build());

        let signals = emitter.signals();
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].name(), "test1");
        assert_eq!(signals[1].name(), "test2");
    }

    #[test]
    fn test_memory_emitter_clear() {
        let emitter = MemoryEmitter::new();

        emitter.emit(Signal::new().name("test").build());
        assert_eq!(emitter.len(), 1);

        emitter.clear();
        assert!(emitter.is_empty());
    }

    #[test]
    fn test_memory_emitter_last() {
        let emitter = MemoryEmitter::new();

        assert!(emitter.last().is_none());

        emitter.emit(Signal::new().name("first").build());
        emitter.emit(Signal::new().name("last").build());

        let last = emitter.last().unwrap();
        assert_eq!(last.name(), "last");
    }

    #[test]
    fn test_memory_emitter_find_by_name() {
        let emitter = MemoryEmitter::new();

        emitter.emit(Signal::new().name("target").build());
        emitter.emit(Signal::new().name("other").build());
        emitter.emit(Signal::new().name("target").build());

        let found = emitter.find_by_name("target");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_memory_emitter_capacity() {
        let emitter = MemoryEmitter::new().with_capacity(2);

        emitter.emit(Signal::new().name("first").build());
        emitter.emit(Signal::new().name("second").build());
        emitter.emit(Signal::new().name("third").build());

        assert_eq!(emitter.len(), 2);

        let signals = emitter.signals();
        assert_eq!(signals[0].name(), "second");
        assert_eq!(signals[1].name(), "third");
    }

    #[test]
    fn test_memory_emitter_clone() {
        let emitter1 = MemoryEmitter::new();
        let emitter2 = emitter1.clone();

        emitter1.emit(Signal::new().name("test").build());

        // Both should see the same signal (shared Arc)
        assert_eq!(emitter1.len(), 1);
        assert_eq!(emitter2.len(), 1);
    }
}
