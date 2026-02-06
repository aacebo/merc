//! ML-specific benchmarking types.
//!
//! This module contains the core ML abstractions:
//! - `Scorer` trait for synchronous text scoring
//! - `AsyncScorer` trait for async/parallel text scoring
//! - `Decision` enum for accept/reject outcomes
//! - `platt` submodule for Platt calibration training
//!
//! For operational types (datasets, results, runner), see `loom_runtime::bench`.

mod decision;
pub mod platt;
mod scorer;

pub use decision::*;
pub use scorer::*;
