use std::io::{self, Write};

use crate::{Emitter, Level, Signal};

/// An emitter that writes signals to stdout.
///
/// # Example
/// ```ignore
/// let emitter = StdoutEmitter::new()
///     .with_level(Level::Debug)
///     .json();
///
/// emitter.emit(signal);
/// ```
pub struct StdoutEmitter {
    min_level: Level,
    json_format: bool,
}

impl StdoutEmitter {
    /// Create a new stdout emitter with default settings.
    /// Default level is Info, default format is human-readable.
    pub fn new() -> Self {
        Self {
            min_level: Level::Info,
            json_format: false,
        }
    }

    /// Set the minimum log level to emit.
    /// Signals below this level will be filtered out.
    pub fn with_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Enable JSON output format.
    pub fn json(mut self) -> Self {
        self.json_format = true;
        self
    }

    fn should_emit(&self, signal: &Signal) -> bool {
        signal.level() as u8 >= self.min_level as u8
    }

    fn format_human(&self, signal: &Signal) -> String {
        format!(
            "[{}] {} {} {:?}",
            signal.level(),
            signal.otype(),
            signal.name(),
            signal.attributes()
        )
    }
}

impl Default for StdoutEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for StdoutEmitter {
    fn emit(&self, signal: Signal) {
        if !self.should_emit(&signal) {
            return;
        }

        let output = if self.json_format {
            #[cfg(feature = "json")]
            {
                serde_json::to_string(&signal).unwrap_or_else(|_| self.format_human(&signal))
            }
            #[cfg(not(feature = "json"))]
            {
                self.format_human(&signal)
            }
        } else {
            self.format_human(&signal)
        };

        let _ = writeln!(io::stdout(), "{}", output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_emitter_default() {
        let emitter = StdoutEmitter::new();
        assert_eq!(emitter.min_level, Level::Info);
        assert!(!emitter.json_format);
    }

    #[test]
    fn test_stdout_emitter_with_level() {
        let emitter = StdoutEmitter::new().with_level(Level::Debug);
        assert_eq!(emitter.min_level, Level::Debug);
    }

    #[test]
    fn test_stdout_emitter_json() {
        let emitter = StdoutEmitter::new().json();
        assert!(emitter.json_format);
    }

    #[test]
    fn test_should_emit_filters_by_level() {
        let emitter = StdoutEmitter::new().with_level(Level::Warn);

        let debug_signal = Signal::new().level(Level::Debug).build();
        let info_signal = Signal::new().level(Level::Info).build();
        let warn_signal = Signal::new().level(Level::Warn).build();
        let error_signal = Signal::new().level(Level::Error).build();

        assert!(!emitter.should_emit(&debug_signal));
        assert!(!emitter.should_emit(&info_signal));
        assert!(emitter.should_emit(&warn_signal));
        assert!(emitter.should_emit(&error_signal));
    }
}
