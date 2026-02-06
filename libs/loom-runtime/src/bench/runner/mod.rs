//! Benchmark runner functions for evaluating scorers on datasets.
//!
//! This module provides various runner implementations:
//! - **sync**: Synchronous execution
//! - **async**: Async execution with spawn_blocking
//! - **batch**: Batch inference for improved throughput
//! - **instrumented**: Signal-instrumented variants for telemetry

mod r#async;
mod batch;
mod config;
mod helpers;
mod instrumented;
mod sync;

pub use r#async::*;
pub use batch::*;
pub use config::*;
pub use helpers::{evaluate_batch_output, evaluate_sample};
pub use instrumented::*;
pub use sync::*;
