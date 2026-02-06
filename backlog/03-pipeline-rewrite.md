# Phase 3: Full Pipeline Infrastructure Rewrite

**Status: COMPLETED**

## Overview
Added pipeline infrastructure and `LoomResult` integration to the runtime, providing a foundation for layer chaining and unified result handling.

## What Was Implemented

### 1. LoomConfig Methods (config.rs)

```rust
impl LoomConfig {
    /// Build a pipeline that outputs ScoreResult
    pub fn build_pipeline(self) -> Result<Pipeline<Context<()>, ScoreResult>>

    /// Build a ScoreLayer for batch processing
    pub fn build_scorer(self) -> Result<ScoreLayer>
}
```

### 2. LoomResult Conversion (result.rs)

```rust
impl LoomResult {
    /// Create from a ScoreResult
    pub fn from_score(score_result: ScoreResult) -> Self

    /// Create from a ScoreResult with metadata
    pub fn from_score_with_meta(score_result: ScoreResult, meta: Map) -> Self

    /// Get typed score result back
    pub fn score(&self) -> Option<ScoreResult>
}

impl From<ScoreResult> for LoomResult { ... }
```

### 3. Pipeline-based Runner Functions (bench/runner/pipeline.rs)

New pipeline-based runner functions for single-item processing:
- `run_pipeline_async` - Run benchmarks using a pipeline
- `run_pipeline_async_with_loom_result` - Run and return `LoomResult` per sample
- `run_pipeline_async_with_scores` - Run and capture raw scores
- `export_pipeline_async` - Export raw scores for Platt calibration

All functions use `Arc<Mutex<Pipeline>>` for thread-safe execution.

## Design Decisions

### Why ScoreResult Instead of LoomResult in Pipeline?

The original plan specified `Pipeline<Context<()>, LoomResult>`, but this was changed to `Pipeline<Context<()>, ScoreResult>` because:

1. **LayerContext constraint** - The `Layer` trait requires `Input: LayerContext`, but intermediate outputs like `ScoreResult` don't implement this trait
2. **Simpler implementation** - Using `LoomResult::from_score()` after pipeline execution is cleaner than chaining layers
3. **Flexibility** - Keeps the pipeline focused on the core ML layer, with `LoomResult` wrapping done at the end

### Why Keep Scorer/BatchScorer?

The batch scorer approach remains the primary method for benchmarking because:
- Batch inference is more efficient for GPU utilization
- The rust-bert models benefit from processing multiple samples together
- The scorer traits provide a simpler interface for batch operations

Pipeline-based runners are available for use cases that prefer single-item processing.

## Files Modified

| File | Changes |
|------|---------|
| `libs/loom-runtime/src/config.rs` | Added `build_pipeline()` and `build_scorer()` methods |
| `libs/loom-runtime/src/result.rs` | Added `from_score()`, `from_score_with_meta()` methods and `From` impl |
| `libs/loom-runtime/src/bench/runner/mod.rs` | Added pipeline module export |
| `libs/loom-runtime/src/bench/runner/pipeline.rs` | **NEW** - Pipeline-based runner functions |

## Files NOT Modified

The following were identified in the original plan but not changed:
- `libs/loom-cli/src/bench/run.rs` - CLI continues using efficient batch scorer approach
- `libs/loom-cli/src/bench/score.rs` - CLI continues using efficient batch scorer approach
- Batch runner functions - Continue using Scorer/BatchScorer traits for efficiency

## Usage Example

```rust
use loom_runtime::{Context, LoomConfig, LoomResult, bench};
use std::sync::{Arc, Mutex};

// Load config
let config: LoomConfig = load_config("config.toml")?;

// Option 1: Use pipeline for single-item processing
let pipeline = Arc::new(Mutex::new(config.clone().build_pipeline()?));
let result = bench::run_pipeline_async(&dataset, pipeline, config, |_| {}).await;

// Option 2: Use scorer for batch processing (more efficient)
let scorer = Arc::new(Mutex::new(config.build_scorer()?));
let result = bench::run_batch_async(&dataset, scorer).await;

// Convert to LoomResult if needed
let loom_result = LoomResult::from_score(score_result);
```

## Verification

1. `cargo build -p loom-runtime -p loom-cli` - ✅ Builds successfully
2. `cargo test -p loom-runtime` - ✅ All 57 tests pass
3. New conversion functions tested with unit tests
4. Pipeline runners compile and export correctly

## Future Enhancements

- Add `LoomResultCollector` layer for true pipeline composition (requires solving `LayerContext` constraint)
- Add batch pipeline operators for GPU-efficient pipeline execution
- Consider deprecating scorer-based runners if pipeline approach becomes preferred
