mod error;
mod registry;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "yaml")]
mod yaml;

#[cfg(feature = "toml")]
mod toml;

mod text;

pub use error::*;
pub use registry::*;

#[cfg(feature = "json")]
pub use json::*;

#[cfg(feature = "yaml")]
pub use yaml::*;

#[cfg(feature = "toml")]
pub use toml::*;

pub use text::*;

// Re-export types from dependencies
pub use loom_core::{Format, MediaType, path, value};
pub use loom_io::{Document, Entity, Record};

pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn decode(&self, record: Record) -> Result<Document, CodecError>;
    fn encode(&self, document: Document) -> Result<Record, CodecError>;
}
