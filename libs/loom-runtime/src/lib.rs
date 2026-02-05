pub mod bench;
mod context;
mod layer;
pub mod model;
pub mod score;

pub use context::*;
pub use layer::*;
pub use model::*;

use loom_codec::{CodecRegistry, CodecRegistryBuilder};
use loom_io::{DataSourceRegistry, DataSourceRegistryBuilder};

pub struct Runtime {
    codecs: CodecRegistry,
    sources: DataSourceRegistry,
}

impl Runtime {
    pub fn new() -> Builder {
        Builder::new()
    }

    pub fn codecs(&self) -> &CodecRegistry {
        &self.codecs
    }

    pub fn sources(&self) -> &DataSourceRegistry {
        &self.sources
    }
}

#[derive(Default)]
pub struct Builder {
    codecs: CodecRegistryBuilder,
    sources: DataSourceRegistryBuilder,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec<T: loom_codec::Codec + 'static>(mut self, codec: T) -> Self {
        self.codecs = self.codecs.codec(codec);
        self
    }

    pub fn source<T: loom_io::DataSource + 'static>(mut self, source: T) -> Self {
        self.sources = self.sources.source(source);
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            codecs: self.codecs.build(),
            sources: self.sources.build(),
        }
    }
}
