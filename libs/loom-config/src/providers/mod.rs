mod env_provider;
mod file_provider;
mod memory_provider;

pub use env_provider::*;
pub use file_provider::*;
pub use memory_provider::*;

use crate::Format;
use crate::path::Path;
use crate::value::Value;

use super::ConfigError;

/// Trait for configuration providers
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn load(&self) -> Result<Option<Value>, ConfigError>;
    fn optional(&self) -> bool {
        false
    }

    fn path(&self) -> Option<Path> {
        None
    }

    fn format(&self) -> Option<Format> {
        None
    }
}
