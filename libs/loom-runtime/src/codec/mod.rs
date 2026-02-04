mod error;

#[cfg(feature = "json")]
mod json_codec;

#[cfg(feature = "yaml")]
mod yaml_codec;

mod text_codec;

pub use error::*;

#[cfg(feature = "json")]
pub use json_codec::*;

#[cfg(feature = "yaml")]
pub use yaml_codec::*;

pub use text_codec::*;

use crate::{Document, Format, Record};

pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn decode(&self, record: Record) -> Result<Document, CodecError>;
    fn encode(&self, document: Document) -> Result<Record, CodecError>;
}
