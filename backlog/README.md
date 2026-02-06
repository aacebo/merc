# Loom Backlog

This backlog tracks the refactoring and feature development for the loom project.

## Status Overview

| Phase | Description | Crate | Status |
|-------|-------------|-------|--------|
| [01-config-integration](01-config-integration.md) | Integrate loom-config crate | cli/runtime | **COMPLETED** |
| [02-validation](02-validation.md) | Add config validation | cli/runtime | **COMPLETED** |
| [03-pipeline-rewrite](03-pipeline-rewrite.md) | Pipeline infrastructure | runtime | **COMPLETED** |
| [04-dynamic-layers](04-dynamic-layers.md) | Runner removal, config simplification | cli/runtime | **COMPLETED** |
| [07-simplify-structure](07-simplify-structure.md) | Merge modules, flatten CLI | cli/runtime | **COMPLETED** |
| [05-output-behavior](05-output-behavior.md) | CLI output path handling | cli | **COMPLETED** |
| [06-fork-join](06-fork-join.md) | Rename spawn→fork, add .join() | pipe | PENDING |
| [08-result-metadata](08-result-metadata.md) | Add timing & resource metrics | runtime | PENDING |
| [09-error-aggregation](09-error-aggregation.md) | Hierarchical layer errors | runtime | PENDING |
| [10-control-flow-ops](10-control-flow-ops.md) | if/then/else, and/or | pipe | PENDING |
| [11-result-operators](11-result-operators.md) | retry, expect/unwrap | pipe | PENDING |
| [12-collection-ops](12-collection-ops.md) | flatten | pipe | PENDING |
| [13-time-operators](13-time-operators.md) | timeout, debounce | pipe | PENDING |

## Priority Tiers

### Tier 1: Pending Core Work
- **Phase 06**: Fork/Join - Foundation for pipe operators

### Tier 2: Runtime Infrastructure
- **Phase 08**: Result metadata - Timing and resource metrics
- **Phase 09**: Error aggregation - Hierarchical error support

### Tier 3: Pipe Operators - Foundation
- **Phase 10**: Control flow - if/then/else, and/or
- **Phase 11**: Result operators - retry, expect/unwrap

### Tier 4: Pipe Operators - Advanced
- **Phase 12**: Collection operators - flatten
- **Phase 13**: Time operators - timeout, debounce

## Dependencies

```
COMPLETED                        PENDING
─────────────────────────────────────────────────────
Phase 01-05, 07 ────────────────► Phase 08 (Metadata)
                                  │
                                  ▼
                             Phase 09 (Errors)


Phase 01-05, 07 ────────────────► Phase 06 (Fork/Join)
                                  │
                       ┌──────────┼──────────┐
                       ▼          ▼          ▼
                  Phase 10    Phase 11   Phase 12
                (Control)    (Result)   (Collection)
                       │          │          │
                       └──────────┼──────────┘
                                  ▼
                             Phase 13 (Time)
```

## Completed Work

### Phase 1: loom-config Integration
- Added `config` feature to loom-cli
- Created `load_config()` helper with env var override support
- Updated run.rs, score.rs, validate.rs to use new config loading

### Phase 2: Config Validation
- Added `Validate` derive to `LoomConfig`, `LayersConfig`
- Added `#[validate(minimum = 1)]` to `concurrency`, `batch_size`, `short_text_limit`, `long_text_limit`
- Added manual nested BTreeMap validation in `ScoreConfig::build()`
- Added `short_text_limit < long_text_limit` constraint

### Phase 3: Pipeline Infrastructure Rewrite
- Added `build_pipeline()` method to `LoomConfig` - returns `Pipeline<Context<()>, ScoreResult>`
- Added `build_scorer()` method to `LoomConfig` - returns `ScoreLayer` for batch processing
- Added `LoomResult::from_score()` conversion helper
- Created new pipeline-based runner functions in `bench/runner/pipeline.rs`
- Kept batch scorer approach for CLI (more efficient for GPU inference)

### Phase 4: Runner Removal & Config Simplification
- Flattened runner module into bench (removed `bench/runner/` subdirectory)
- Created `bench/execution.rs` - consolidated async/batch execution functions
- Created `bench/helpers.rs` - evaluation and result-building helpers
- Removed `build_scorer()` method - use `config.layers.score.build()` directly
- Renamed `build_pipeline()` to `build()` - simpler API
- Deleted 8 runner files
- `load_config()` now returns `loom_config::Config` directly (not typed)
- Dynamic layer access via `config.get_section(&IdentPath::parse("layers.score"))`
- Removed `LayersConfig` struct - `LoomConfig` now only contains runtime settings
- Re-exported `ScoreConfig` from `loom_runtime` for convenience

### Phase 7: Simplify Structure
- Removed all coverage-related types/commands
- Moved execution logic to `ScoreLayer.eval()` method
- Renamed types: `BenchSample` → `Sample`, `BenchDataset` → `SampleDataset`, `BenchResult` → `EvalResult`, `BenchMetrics` → `EvalMetrics`
- Merged bench and score modules into unified `eval/` module
- Flattened CLI: `loom bench run` → `loom run`

### Phase 5: Output Path Behavior
- Added `resolve_output_path()` helper to CLI commands module
- Output path is now a directory, auto-naming files by command:
  - `loom score` → `scores.json`
  - `loom run` → `results.json`
- Default output directory is the input file's parent directory
- Added `--output` flag to `run` command
- Output directory is created automatically if it doesn't exist

## Environment Variable Support

Users can override config via environment variables:

```bash
# Override concurrency
LOOM_CONCURRENCY=16 loom run -c config.toml ...

# Override nested config (use __ for literal underscore)
LOOM_BATCH__SIZE=32 loom run -c config.toml ...
LOOM_LAYERS_SCORE_THRESHOLD=0.8 loom run -c config.toml ...
```
