use std::sync::Arc;

use crate::{Emitter, Signal};

/// A composite emitter that broadcasts signals to multiple emitters.
///
/// # Example
/// ```ignore
/// let broadcaster = SignalBroadcaster::new()
///     .add(StdoutEmitter::new())
///     .add(FileEmitter::new("signals.jsonl")?);
///
/// broadcaster.emit(signal); // Sends to both emitters
/// ```
pub struct SignalBroadcaster {
    emitters: Vec<Arc<dyn Emitter + Send + Sync>>,
}

impl SignalBroadcaster {
    /// Create a new empty broadcaster.
    pub fn new() -> Self {
        Self {
            emitters: Vec::new(),
        }
    }

    /// Add an emitter to the broadcaster.
    pub fn add<E: Emitter + Send + Sync + 'static>(mut self, emitter: E) -> Self {
        self.emitters.push(Arc::new(emitter));
        self
    }

    /// Check if the broadcaster has any emitters.
    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }

    /// Get the number of emitters.
    pub fn len(&self) -> usize {
        self.emitters.len()
    }
}

impl Default for SignalBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for SignalBroadcaster {
    fn emit(&self, signal: Signal) {
        for emitter in &self.emitters {
            emitter.emit(signal.clone());
        }
    }
}

/// A no-op emitter that discards all signals.
/// Used as the default when signals are disabled.
pub struct NoopEmitter;

impl Emitter for NoopEmitter {
    fn emit(&self, _signal: Signal) {
        // Intentionally empty - signals are discarded
    }
}

impl Default for NoopEmitter {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Level;
    use std::sync::Mutex;

    struct CountingEmitter {
        count: Arc<Mutex<usize>>,
    }

    impl CountingEmitter {
        fn new() -> (Self, Arc<Mutex<usize>>) {
            let count = Arc::new(Mutex::new(0));
            (
                Self {
                    count: count.clone(),
                },
                count,
            )
        }
    }

    impl Emitter for CountingEmitter {
        fn emit(&self, _signal: Signal) {
            let mut count = self.count.lock().unwrap();
            *count += 1;
        }
    }

    #[test]
    fn test_broadcaster_empty() {
        let broadcaster = SignalBroadcaster::new();
        assert!(broadcaster.is_empty());
        assert_eq!(broadcaster.len(), 0);
    }

    #[test]
    fn test_broadcaster_add() {
        let broadcaster = SignalBroadcaster::new().add(NoopEmitter).add(NoopEmitter);
        assert!(!broadcaster.is_empty());
        assert_eq!(broadcaster.len(), 2);
    }

    #[test]
    fn test_broadcaster_emits_to_all() {
        let (emitter1, count1) = CountingEmitter::new();
        let (emitter2, count2) = CountingEmitter::new();

        let broadcaster = SignalBroadcaster::new().add(emitter1).add(emitter2);

        let signal = Signal::new().name("test").build();
        broadcaster.emit(signal);

        assert_eq!(*count1.lock().unwrap(), 1);
        assert_eq!(*count2.lock().unwrap(), 1);
    }

    #[test]
    fn test_noop_emitter() {
        let emitter = NoopEmitter;
        let signal = Signal::new().name("test").level(Level::Info).build();

        // Should not panic
        emitter.emit(signal);
    }
}
