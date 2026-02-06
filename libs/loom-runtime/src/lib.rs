pub mod bench;
mod context;
pub mod score;

pub use context::*;

use std::sync::Arc;

use loom_codec::{CodecRegistry, CodecRegistryBuilder};
use loom_core::{Format, MediaType, decode, encode};
use loom_error::Result;
use loom_io::{DataSourceRegistry, DataSourceRegistryBuilder, path::Path};
pub use loom_pipe::{
    Layer, LayerContext, LayerResult, Pipeline, PipelineBuilder,
    operators::{Await, FanOut, Filter, Parallel, Router, Spawn, TryMap},
};
use serde::{Serialize, de::DeserializeOwned};

// Re-export commonly used types for convenience
#[cfg(feature = "toml")]
pub use loom_codec::TomlCodec;
#[cfg(feature = "yaml")]
pub use loom_codec::YamlCodec;
pub use loom_codec::{JsonCodec, TextCodec};
pub use loom_io::Record;
pub use loom_io::sources::FileSystemSource;

// Re-export signal types for convenience
pub use loom_signal::{
    Emitter, Level, NoopEmitter, Signal, SignalBroadcaster, Span, Type as SignalType,
    consumers::{FileEmitter, MemoryEmitter, StdoutEmitter},
};

pub struct Runtime {
    codecs: CodecRegistry,
    sources: DataSourceRegistry,
    signals: Arc<dyn Emitter + Send + Sync>,
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

    /// Get a reference to the signal emitter.
    pub fn emitter(&self) -> &dyn Emitter {
        self.signals.as_ref()
    }

    /// Emit a signal through the runtime's emitter.
    pub fn emit(&self, signal: Signal) {
        self.signals.emit(signal);
    }

    pub fn pipeline<Input: Send + 'static>(&self) -> PipelineBuilder<Input, Input> {
        PipelineBuilder::new()
    }

    /// Load and deserialize data from a DataSource.
    ///
    /// # Arguments
    /// * `source` - The name of the registered DataSource (e.g., "file_system")
    /// * `path` - The path to load from
    ///
    /// # Example
    /// ```ignore
    /// let dataset: BenchDataset = runtime.load("file_system", &path).await?;
    /// ```
    pub async fn load<T: DeserializeOwned>(&self, source: &str, path: &Path) -> Result<T> {
        let source = self.sources.get(source).ok_or_else(|| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::NotFound)
                .message(format!("DataSource '{}' not found", source))
                .build()
        })?;

        let record = source.find_one(path).await.map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Failed to load from path '{}': {}", path, e))
                .build()
        })?;

        let content = record.content_str().map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Invalid UTF-8 content: {}", e))
                .build()
        })?;

        decode!(content, record.media_type.format()).map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Deserialization failed: {}", e))
                .build()
        })
    }

    /// Save and serialize data to a DataSource.
    ///
    /// # Arguments
    /// * `source` - The name of the registered DataSource (e.g., "file_system")
    /// * `path` - The path to save to
    /// * `data` - The data to serialize and save
    /// * `format` - The format to serialize as
    ///
    /// # Example
    /// ```ignore
    /// runtime.save("file_system", &path, &export, Format::Json).await?;
    /// ```
    pub async fn save<T: Serialize>(
        &self,
        source: &str,
        path: &Path,
        data: &T,
        format: Format,
    ) -> Result<()> {
        let source = self.sources.get(source).ok_or_else(|| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::NotFound)
                .message(format!("DataSource '{}' not found", source))
                .build()
        })?;

        let content = encode!(data, format).map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Serialization failed: {}", e))
                .build()
        })?;

        let media_type = match format {
            Format::Json => MediaType::TextJson,
            Format::Yaml => MediaType::TextYaml,
            Format::Toml => MediaType::TextToml,
            _ => MediaType::TextPlain,
        };

        let record = loom_io::Record::from_str(path.clone(), media_type, &content);

        source.upsert(record).await.map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Failed to save to path '{}': {}", path, e))
                .build()
        })?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    codecs: CodecRegistryBuilder,
    sources: DataSourceRegistryBuilder,
    signals: SignalBroadcaster,
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

    /// Add a signal emitter to the runtime.
    /// Multiple emitters can be added and signals will be broadcast to all of them.
    pub fn emitter<E: Emitter + Send + Sync + 'static>(mut self, emitter: E) -> Self {
        self.signals = self.signals.add(emitter);
        self
    }

    pub fn build(self) -> Runtime {
        let signals: Arc<dyn Emitter + Send + Sync> = if self.signals.is_empty() {
            Arc::new(NoopEmitter)
        } else {
            Arc::new(self.signals)
        };

        Runtime {
            codecs: self.codecs.build(),
            sources: self.sources.build(),
            signals,
        }
    }
}
