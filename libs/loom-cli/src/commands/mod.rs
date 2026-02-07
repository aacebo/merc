use std::path::{Path, PathBuf};

use loom::config::{Config, ConfigError, EnvProvider, FileProvider};
use loom::runtime::{FileSystemSource, JsonCodec, Runtime, TomlCodec, YamlCodec};

pub mod classify;
pub mod run;
pub mod score;
pub mod train;
pub mod validate;

pub use classify::ClassifyCommand;
pub use run::RunCommand;
pub use score::ScoreCommand;
pub use train::TrainCommand;
pub use validate::ValidateCommand;

/// Resolve the output file path based on input path, optional output directory, and filename.
///
/// # Arguments
/// * `input_path` - Path to the input samples file
/// * `output_dir` - Optional output directory (from config or CLI)
/// * `filename` - The output filename (e.g., "scores.json", "results.json")
///
/// # Returns
/// The resolved output file path
pub fn resolve_output_path(
    input_path: &Path,
    output_dir: Option<&Path>,
    filename: &str,
) -> PathBuf {
    let base_dir = output_dir.unwrap_or_else(|| input_path.parent().unwrap_or(Path::new(".")));
    base_dir.join(filename)
}

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
