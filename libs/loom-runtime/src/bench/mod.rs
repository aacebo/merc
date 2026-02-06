//! Benchmarking module for evaluating scorer performance.
//!
//! This module provides infrastructure for running benchmarks against datasets,
//! validating datasets, computing coverage reports, and exporting scores for
//! Platt calibration training.
//!
//! # Architecture
//!
//! - **Operational types** (dataset, results, runner) live here in runtime
//! - **ML types** (Scorer trait, Platt calibration) live in cortex
//!
//! # Example
//!
//! ```ignore
//! use loom_runtime::{Runtime, FileSystemSource, JsonCodec, bench};
//! use loom_io::path::{Path, FilePath};
//!
//! let runtime = Runtime::new()
//!     .source(FileSystemSource::builder().build())
//!     .codec(JsonCodec::new())
//!     .build();
//!
//! let path = Path::File(FilePath::parse("benchmark.json"));
//! let dataset: bench::BenchDataset = runtime.load("file_system", &path).await?;
//!
//! let errors = dataset.validate();
//! let coverage = dataset.coverage();
//! ```

// Operational types - owned by runtime
mod category;
mod coverage;
mod dataset;
mod difficulty;
mod progress;
pub mod result;
mod runner;
mod sample;
mod validation;

// Public exports - operational types
pub use category::*;
pub use coverage::*;
pub use dataset::*;
pub use difficulty::*;
pub use progress::*;
pub use result::*;
pub use runner::*;
pub use sample::*;
pub use validation::*;

// Re-export ML types from cortex for convenience
pub use loom_cortex::bench::{AsyncScorer, Scorer, ScorerOutput};

// Re-export Platt calibration types and functions from cortex
pub use loom_cortex::bench::platt::{
    PlattParams, PlattTrainingResult, RawScoreExport, SampleScores, generate_rust_code,
    train_platt_params,
};
