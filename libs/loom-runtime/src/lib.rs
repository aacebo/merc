pub mod bench;
mod context;
mod layer;
mod options;
pub mod score;

pub use loom_pipe;

pub use context::*;
pub use layer::*;
pub use options::*;

// Re-export from loom-core
pub use loom_core::{Format, Map, MediaType, path, value};

// Re-export from loom-config
pub use loom_config as config;
pub use loom_config::{ConfigBuilder, ConfigError, ConfigRoot, ConfigSection, ConfigSource};

// Re-export from loom-io
pub use loom_io as data_source;
pub use loom_io::{DataSource, Document, ETag, Entity, Id, ReadError, Record, WriteError};

// Re-export from loom-codec
pub use loom_codec as codec;
pub use loom_codec::{Codec, CodecError};

#[cfg(feature = "json")]
pub use loom_codec::JsonCodec;

#[cfg(feature = "yaml")]
pub use loom_codec::YamlCodec;

#[cfg(feature = "toml")]
pub use loom_codec::TomlCodec;

pub use loom_codec::TextCodec;

pub struct Runtime {
    #[allow(unused)]
    codecs: Vec<Box<dyn Codec>>,

    #[allow(unused)]
    sources: Vec<Box<dyn DataSource>>,
}

#[derive(Default)]
pub struct Builder {
    codecs: Vec<Box<dyn Codec>>,
    sources: Vec<Box<dyn DataSource>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec<T: Codec + 'static>(mut self, codec: T) -> Self {
        self.codecs.push(Box::new(codec));
        self
    }

    pub fn source<T: DataSource + 'static>(mut self, source: T) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            codecs: self.codecs,
            sources: self.sources,
        }
    }
}
