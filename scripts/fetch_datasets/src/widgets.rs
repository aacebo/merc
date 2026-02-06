use std::fmt;
use std::io::{Write, stdout};
use std::ops::Deref;

use crossterm::{ExecutableCommand, cursor, terminal};

/// Result of rendering a widget, wraps the rendered string
pub struct WidgetResult(String);

impl Deref for WidgetResult {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WidgetResult {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Write to stdout, clearing the current line first
    pub fn write(&self) {
        self.write_to(&mut stdout());
    }

    /// Write to any writer, clearing the line first (for terminal writers)
    pub fn write_to(&self, writer: &mut impl Write) {
        let mut stdout = stdout();
        let _ = stdout.execute(cursor::MoveToColumn(0));
        let _ = stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine));
        let _ = write!(writer, "{}", self.0);
        let _ = writer.flush();
    }
}

/// Trait for all widgets that can be rendered
pub trait Widget: fmt::Display {
    fn render(&self) -> WidgetResult;
}

/// Clear the current line (useful after inline widgets)
pub fn clear_line() {
    let mut stdout = stdout();
    let _ = stdout.execute(cursor::MoveToColumn(0));
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine));
    let _ = stdout.flush();
}

// ============================================================================
// ProgressBar
// ============================================================================

pub struct ProgressBar {
    current: usize,
    total: usize,
    message: String,
    status_icon: Option<char>,
    bar_width: usize,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            current: 0,
            total: 100,
            message: String::new(),
            status_icon: None,
            bar_width: 30,
        }
    }

    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    pub fn total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    #[allow(dead_code)]
    pub fn status(mut self, icon: char) -> Self {
        self.status_icon = Some(icon);
        self
    }

    #[allow(dead_code)]
    pub fn bar_width(mut self, width: usize) -> Self {
        self.bar_width = width;
        self
    }

    pub fn clear() {
        clear_line();
    }
}

impl Widget for ProgressBar {
    fn render(&self) -> WidgetResult {
        let pct = if self.total > 0 {
            self.current as f32 / self.total as f32
        } else {
            0.0
        };
        let filled = (pct * self.bar_width as f32) as usize;
        let empty = self.bar_width.saturating_sub(filled);
        let status = self
            .status_icon
            .map(|c| format!(" {}", c))
            .unwrap_or_default();

        WidgetResult::new(format!(
            "[{}{}] {:3.0}% ({}/{}){}  {}",
            "█".repeat(filled),
            "░".repeat(empty),
            pct * 100.0,
            self.current,
            self.total,
            status,
            self.message
        ))
    }
}

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.render())
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Spinner
// ============================================================================

const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub struct Spinner {
    message: String,
    frame_idx: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            frame_idx: 0,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn frame(mut self, idx: usize) -> Self {
        self.frame_idx = idx % FRAMES.len();
        self
    }

    #[allow(dead_code)]
    pub fn tick(&mut self) {
        self.frame_idx = (self.frame_idx + 1) % FRAMES.len();
    }

    pub fn clear() {
        clear_line();
    }
}

impl Widget for Spinner {
    fn render(&self) -> WidgetResult {
        WidgetResult::new(format!("{} {}", FRAMES[self.frame_idx], self.message))
    }
}

impl fmt::Display for Spinner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.render())
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DownloadProgress - specialized for download tracking with byte sizes
// ============================================================================

pub struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
    message: String,
    bar_width: usize,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            downloaded: 0,
            total: None,
            message: String::new(),
            bar_width: 30,
        }
    }

    pub fn downloaded(mut self, bytes: u64) -> Self {
        self.downloaded = bytes;
        self
    }

    pub fn total(mut self, bytes: Option<u64>) -> Self {
        self.total = bytes;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn clear() {
        clear_line();
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

impl Widget for DownloadProgress {
    fn render(&self) -> WidgetResult {
        match self.total {
            Some(total) if total > 0 => {
                let pct = self.downloaded as f32 / total as f32;
                let filled = (pct * self.bar_width as f32) as usize;
                let empty = self.bar_width.saturating_sub(filled);

                WidgetResult::new(format!(
                    "[{}{}] {:3.0}%  {} / {}  {}",
                    "█".repeat(filled),
                    "░".repeat(empty),
                    pct * 100.0,
                    format_bytes(self.downloaded),
                    format_bytes(total),
                    self.message
                ))
            }
            _ => {
                // Unknown total, show spinner-style with downloaded bytes
                let frame_idx = ((self.downloaded / 10000) % 10) as usize;
                WidgetResult::new(format!(
                    "{} {}  {}",
                    FRAMES[frame_idx],
                    format_bytes(self.downloaded),
                    self.message
                ))
            }
        }
    }
}

impl fmt::Display for DownloadProgress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.render())
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self::new()
    }
}
