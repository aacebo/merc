mod context;
pub mod score;

pub use context::*;

use loom_codec::{CodecRegistry, CodecRegistryBuilder};
use loom_io::{DataSourceRegistry, DataSourceRegistryBuilder};
pub use loom_pipe::{
    Layer, LayerContext, LayerResult, Pipeline, PipelineBuilder,
    operators::{Await, FanOut, Filter, Parallel, Router, Spawn, TryMap},
};

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

    pub fn pipeline<Input: Send + 'static>(&self) -> PipelineBuilder<Input, Input> {
        PipelineBuilder::new()
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
