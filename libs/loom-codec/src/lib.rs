mod error;

#[cfg(feature = "json")]
mod json_codec;

#[cfg(feature = "yaml")]
mod yaml_codec;

#[cfg(feature = "toml")]
mod toml_codec;

mod text_codec;

pub use error::*;

#[cfg(feature = "json")]
pub use json_codec::*;

#[cfg(feature = "yaml")]
pub use yaml_codec::*;

#[cfg(feature = "toml")]
pub use toml_codec::*;

pub use text_codec::*;

// Re-export types from dependencies
pub use loom_core::{Format, MediaType, path, value};
pub use loom_io::{Document, Entity, Record};

pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn decode(&self, record: Record) -> Result<Document, CodecError>;
    fn encode(&self, document: Document) -> Result<Record, CodecError>;
}
