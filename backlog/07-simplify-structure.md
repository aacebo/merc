# Phase 7: Simplify Structure

**Status: COMPLETED**

## Overview

Major cleanup and simplification:
1. **Remove coverage** - delete all coverage-related types/commands
2. **Move execution to layers** - `ScoreLayer.eval()` instead of standalone functions
3. **Rename types** - remove "Bench" prefix for cleaner API
4. **Merge modules** - combine bench and score into unified `eval/` module
5. **Flatten CLI** - `loom run` instead of `loom bench run`

## Step 1: Remove Coverage

**Status: COMPLETED**

**Files deleted:**
- `libs/loom-runtime/src/bench/coverage.rs`
- `libs/loom-cli/src/bench/cov.rs`

**Files modified:**
- `libs/loom-runtime/src/bench/mod.rs` - removed coverage module and exports
- `libs/loom-runtime/src/bench/dataset.rs` - removed `coverage()` and `coverage_with_labels()` methods
- `libs/loom-cli/src/bench/mod.rs` - removed `Coverage` variant from `BenchAction` enum

## Step 2: Move Execution to Layers

**Status: COMPLETED**

Removed standalone execution functions. All execution now goes through `ScoreLayer` methods.

**Before:**
```rust
bench::run_batch_async_with_config(&dataset, scorer, config, callback).await
```

**After:**
```rust
let scorer = Arc::new(Mutex::new(score_config.build()?));
ScoreLayer::eval(scorer, &dataset, batch_size, callback).await
```

**Files deleted:**
- `libs/loom-runtime/src/bench/execution.rs`
- `libs/loom-runtime/src/bench/helpers.rs`
- `libs/loom-runtime/src/bench/runner/` (entire directory)

**ScoreLayer methods added:**
- `eval()` - primary entry point for running evaluations
- `eval_with_scores()` - evaluates and captures raw scores
- `export_scores()` - exports raw scores for Platt calibration

## Step 3: Rename Types

**Status: COMPLETED**

| Old Name | New Name |
|----------|----------|
| `BenchSample` | `Sample` |
| `BenchDataset` | `SampleDataset` |
| `BenchResult` | `EvalResult` |
| `BenchMetrics` | `EvalMetrics` |
| `AsyncRunConfig` | (deleted) |

## Step 4: Merge Modules

**Status: COMPLETED**

**Old structure:**
```
libs/loom-runtime/src/
├── bench/
│   ├── mod.rs, sample.rs, dataset.rs, ...
│   └── result/
└── score/
    ├── mod.rs (ScoreLayer)
    └── config/
```

**New structure:**
```
libs/loom-runtime/src/
└── eval/
    ├── mod.rs
    ├── sample.rs (Sample)
    ├── dataset.rs (SampleDataset)
    ├── difficulty.rs
    ├── progress.rs
    ├── validation.rs
    ├── result/
    │   ├── mod.rs
    │   ├── eval.rs (EvalResult, EvalMetrics)
    │   ├── sample.rs (SampleResult)
    │   ├── label.rs
    │   ├── category.rs
    │   ├── metrics.rs
    │   └── export.rs
    └── score/
        ├── mod.rs (ScoreLayer with eval methods)
        ├── result.rs (ScoreResult, ScoreCategory, ScoreLabel)
        └── config/
            ├── mod.rs (ScoreConfig)
            ├── category.rs
            ├── label.rs
            └── modifier.rs
```

**Import changes:**
- `loom::runtime::bench` → `loom::runtime::eval`
- `loom::runtime::score::ScoreLayer` → `loom::runtime::eval::score::ScoreLayer`

## Step 5: Flatten CLI

**Status: COMPLETED**

**Before:**
```
loom bench run ...
loom bench validate ...
loom bench score ...
loom bench train ...
```

**After:**
```
loom run ...
loom validate ...
loom score ...
loom train ...
```

**Changes:**
- Renamed `libs/loom-cli/src/bench/` → `libs/loom-cli/src/commands/`
- Updated `main.rs` to use top-level commands instead of nested `BenchAction`
- Removed `BenchAction` enum, commands defined directly in `main.rs`

## Verification

All tests pass (56 tests):
```bash
cargo test -p loom-runtime -p loom-cli
```

CLI commands verified:
```bash
loom --help
# Commands: run, validate, score, train

loom run --help
# Run evaluation against a dataset
```
