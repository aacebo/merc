use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::{Emitter, Level, Signal};

/// An emitter that writes signals to a file in JSON lines format.
///
/// Each signal is written as a single JSON line, making it easy to parse
/// and process with standard tools.
///
/// # Example
/// ```ignore
/// let emitter = FileEmitter::new("signals.jsonl")?;
/// emitter.emit(signal);
/// ```
pub struct FileEmitter {
    writer: Mutex<BufWriter<File>>,
    min_level: Level,
}

impl FileEmitter {
    /// Create a new file emitter that appends to the given path.
    /// Creates the file if it doesn't exist.
    pub fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.into())?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            min_level: Level::Trace,
        })
    }

    /// Set the minimum log level to emit.
    pub fn with_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    fn should_emit(&self, signal: &Signal) -> bool {
        signal.level() as u8 >= self.min_level as u8
    }
}

impl Emitter for FileEmitter {
    fn emit(&self, signal: Signal) {
        if !self.should_emit(&signal) {
            return;
        }

        #[cfg(feature = "json")]
        {
            if let Ok(mut writer) = self.writer.lock() {
                if let Ok(json) = serde_json::to_string(&signal) {
                    let _ = writeln!(writer, "{}", json);
                    let _ = writer.flush();
                }
            }
        }

        #[cfg(not(feature = "json"))]
        {
            // Without JSON feature, write a debug representation
            if let Ok(mut writer) = self.writer.lock() {
                let _ = writeln!(writer, "{:?}", signal);
                let _ = writer.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn cleanup_test_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_file_emitter_creates_file() {
        let path = "/tmp/loom_signal_test_create.jsonl";
        cleanup_test_file(path);

        let emitter = FileEmitter::new(path);
        assert!(emitter.is_ok());
        assert!(Path::new(path).exists());

        cleanup_test_file(path);
    }

    #[test]
    fn test_file_emitter_writes_signal() {
        let path = "/tmp/loom_signal_test_write.jsonl";
        cleanup_test_file(path);

        let emitter = FileEmitter::new(path).unwrap();
        let signal = Signal::new().name("test.event").build();
        emitter.emit(signal);

        // Force flush by dropping
        drop(emitter);

        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("test.event"));

        cleanup_test_file(path);
    }

    #[test]
    fn test_file_emitter_with_level() {
        let path = "/tmp/loom_signal_test_level.jsonl";
        cleanup_test_file(path);

        let emitter = FileEmitter::new(path).unwrap().with_level(Level::Warn);

        // This should be filtered out
        let debug_signal = Signal::new().name("debug").level(Level::Debug).build();
        emitter.emit(debug_signal);

        // This should be written
        let warn_signal = Signal::new().name("warn").level(Level::Warn).build();
        emitter.emit(warn_signal);

        drop(emitter);

        let contents = fs::read_to_string(path).unwrap();
        assert!(!contents.contains("debug"));
        assert!(contents.contains("warn"));

        cleanup_test_file(path);
    }
}
