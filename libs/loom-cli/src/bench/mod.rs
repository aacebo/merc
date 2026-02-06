use std::path::PathBuf;

use clap::Subcommand;
use loom::runtime::{FileSystemSource, JsonCodec, Runtime, TomlCodec, YamlCodec};

mod cov;
mod run;
mod score;
mod train;
mod validate;

/// Build a Runtime configured with standard sources and codecs.
pub fn build_runtime() -> Runtime {
    Runtime::new()
        .source(FileSystemSource::builder().build())
        .codec(JsonCodec::new())
        .codec(YamlCodec::new())
        .codec(TomlCodec::new())
        .build()
}

#[derive(Subcommand)]
pub enum BenchAction {
    /// Run benchmark against a dataset
    Run {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Path to score config file (YAML/JSON/TOML)
        #[arg(short, long)]
        config: PathBuf,
        /// Show detailed per-category and per-label results
        #[arg(short, long)]
        verbose: bool,
        /// Number of parallel inference workers (default: 4)
        #[arg(long, default_value = "4")]
        concurrency: usize,
    },
    /// Validate a benchmark dataset
    Validate {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Show label coverage for a dataset
    Coverage {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Extract raw scores for Platt calibration training
    Score {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Path to score config file (YAML/JSON/TOML)
        #[arg(short, long)]
        config: PathBuf,
        /// Output path for raw scores JSON
        #[arg(short, long)]
        output: PathBuf,
        /// Number of parallel inference workers (default: 4)
        #[arg(long, default_value = "4")]
        concurrency: usize,
    },
    /// Train Platt calibration parameters from raw scores
    Train {
        /// Path to raw scores JSON (from extract-scores)
        path: PathBuf,
        /// Output path for trained parameters JSON
        #[arg(short, long)]
        output: PathBuf,
        /// Also output Rust code for label.rs
        #[arg(long)]
        code: bool,
    },
}

pub async fn run(action: BenchAction) {
    match action {
        BenchAction::Run {
            path,
            config,
            verbose,
            concurrency,
        } => run::exec(&path, &config, verbose, concurrency).await,
        BenchAction::Validate { path } => validate::exec(&path).await,
        BenchAction::Coverage { path } => cov::exec(&path).await,
        BenchAction::Score {
            path,
            config,
            output,
            concurrency,
        } => score::exec(&path, &config, &output, concurrency).await,
        BenchAction::Train { path, output, code } => train::exec(&path, &output, code).await,
    }
}
