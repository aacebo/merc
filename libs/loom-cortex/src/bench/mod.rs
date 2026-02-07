//! ML-specific benchmarking types.
//!
//! This module contains:
//! - `Decision` enum for accept/reject outcomes
//! - `platt` submodule for Platt calibration training
//!
//! For operational types (datasets, results, runner), see `loom_runtime::eval`.

mod decision;
pub mod platt;

pub use decision::*;
