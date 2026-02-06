use loom::config::{Config, ConfigError, EnvProvider, FileProvider};
use loom::runtime::{FileSystemSource, JsonCodec, Runtime, TomlCodec, YamlCodec};

pub mod run;
pub mod score;
pub mod train;
pub mod validate;

/// Build a Runtime configured with standard sources and codecs.
pub fn build_runtime() -> Runtime {
    Runtime::new()
        .source(FileSystemSource::builder().build())
        .codec(JsonCodec::new())
        .codec(YamlCodec::new())
        .codec(TomlCodec::new())
        .build()
}

/// Load configuration from file with environment variable overrides.
///
/// Returns the raw `Config` object for dynamic section access.
///
/// Environment variables with prefix `LOOM_` override file values.
/// Mapping rules (after prefix removal):
/// - Single `_` becomes `.` (hierarchy separator)
/// - Double `__` becomes literal `_` in key name
///
/// Examples:
/// - `LOOM_CONCURRENCY=16` -> `concurrency: 16`
/// - `LOOM_BATCH__SIZE=32` -> `batch_size: 32`
/// - `LOOM_LAYERS_SCORE_THRESHOLD=0.8` -> `layers.score.threshold: 0.8`
pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    Config::new()
        .with_provider(FileProvider::builder(config_path).build())
        .with_provider(EnvProvider::new(Some("LOOM_")))
        .build()
}
