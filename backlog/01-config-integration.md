# Phase 1: loom-config Integration

**Status: COMPLETED**

## Overview
Integrate the unused `loom-config` crate to enable environment variable overrides and multi-source config merging.

## Completed Changes

### Step 1: Enable config feature in loom-cli
**File:** `libs/loom-cli/Cargo.toml`

```toml
loom = { workspace = true, features = ["runtime", "cortex", "core", "io", "json", "yaml", "toml", "config"] }
```

### Step 2: Added load_config helper function
**File:** `libs/loom-cli/src/bench/mod.rs:33-39`

```rust
pub fn load_config<T: DeserializeOwned>(config_path: &str) -> Result<T, ConfigError> {
    Config::new()
        .with_provider(FileProvider::builder(config_path).build())
        .with_provider(EnvProvider::new(Some("LOOM_")))
        .build()?
        .bind()
}
```

### Step 3: Updated CLI commands
- `libs/loom-cli/src/bench/run.rs:34-40` - Now uses `load_config()`
- `libs/loom-cli/src/bench/score.rs:35-41` - Now uses `load_config()`
- `libs/loom-cli/src/bench/validate.rs:31-38` - Now uses `load_config()`

### Step 4: build_runtime() retained
Kept `build_runtime()` as it's still used for dataset loading and file saving operations.

## Environment Variable Mapping

With prefix `LOOM_`:
- Single `_` becomes `.` (hierarchy separator)
- Double `__` becomes literal `_` in key name

| Environment Variable | Config Key |
|---------------------|------------|
| `LOOM_CONCURRENCY=16` | `concurrency: 16` |
| `LOOM_BATCH__SIZE=32` | `batch_size: 32` |
| `LOOM_STRICT=true` | `strict: true` |
| `LOOM_LAYERS_SCORE_THRESHOLD=0.8` | `layers.score.threshold: 0.8` |
| `LOOM_LAYERS_SCORE_TOP__K=3` | `layers.score.top_k: 3` |

## Verification Results
- `cargo build -p loom-cli` - PASSED
- `cargo test -p loom-config` - 55 tests PASSED
