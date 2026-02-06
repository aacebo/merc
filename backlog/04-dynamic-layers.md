# Phase 4: Dynamic Layer Configuration & Runner Removal

**Status: COMPLETED**

## Overview

Simplify the runtime by:
1. ✅ **Remove the runner module entirely** - Flattened into bench module (execution.rs + helpers.rs)
2. ✅ **Remove `build_scorer()`** - Use `config.layers.score.build()` directly
3. ✅ **Rename `build_pipeline()` to `build()`** - Simpler API
4. ✅ **`load_config()` returns `loom_config::Config` directly** - CLI now works with raw Config
5. ✅ **Layers accessed dynamically via `config.get_section()`** - Using `IdentPath::parse("layers.score")`
6. ✅ **Remove `LayersConfig` struct** - `LoomConfig` now only contains runtime settings

## Prerequisites (from Phase 3)

- `build_pipeline()` method exists on config
- Pipeline infrastructure is in place
- CLI uses pipelines for execution

## Implementation Steps

### Step 1: Remove Runner Module

**Delete entire directory:** `libs/loom-runtime/src/bench/runner/`

Files to delete:
- `async.rs`
- `batch.rs`
- `config.rs`
- `helpers.rs`
- `instrumented.rs`
- `mod.rs`
- `pipeline.rs`
- `sync.rs`

**Update:** `libs/loom-runtime/src/bench/mod.rs` - Remove runner module and exports

### Step 2: Simplify LoomConfig Methods

**File:** `libs/loom-runtime/src/config.rs`

```rust
// Remove build_scorer() method entirely
// Rename build_pipeline() to build()

impl LoomConfig {
    /// Build a pipeline from this configuration.
    pub fn build(self) -> Result<Pipeline<Context<()>, ScoreResult>> {
        let score_layer = self.layers.score.build()?;
        Ok(PipelineBuilder::new().then(score_layer).build())
    }

    // build_scorer() REMOVED - use config.layers.score.build() directly
}
```

### Step 3: Change load_config() Return Type

**File:** `libs/loom-cli/src/bench/mod.rs`

```rust
// Before
pub fn load_config<T: DeserializeOwned>(config_path: &str) -> Result<T, ConfigError> {
    Config::new()
        .with_provider(FileProvider::builder(config_path).build())
        .with_provider(EnvProvider::new(Some("LOOM_")))
        .build()?
        .bind()
}

// After
pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    Config::new()
        .with_provider(FileProvider::builder(config_path).build())
        .with_provider(EnvProvider::new(Some("LOOM_")))
        .build()
}
```

### Step 4: Simplify LoomConfig Struct

**File:** `libs/loom-runtime/src/config.rs`

```rust
// Remove LayersConfig entirely
// Keep only runtime settings in LoomConfig

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoomConfig {
    #[serde(default)]
    pub output: Option<PathBuf>,

    #[serde(default)]
    pub strict: bool,

    #[serde(default = "LoomConfig::default_concurrency")]
    #[validate(minimum = 1)]
    pub concurrency: usize,

    #[serde(default = "LoomConfig::default_batch_size")]
    #[validate(minimum = 1)]
    pub batch_size: usize,

    // Note: layers field removed - accessed dynamically via Config
}
```

### Step 5: Update CLI Commands

**Files:** `libs/loom-cli/src/bench/{run,score,validate}.rs`

```rust
// Load config (returns loom_config::Config)
let config = load_config(config_path.to_str().unwrap_or_default())?;

// Get runtime settings
let runtime_config: LoomConfig = config.bind()?;
runtime_config.validate()?;

// Get score layer config dynamically
let score_path = IdentPath::parse("layers.score")?;
let score_section = config.get_section(&score_path);

if score_section.exists() {
    let score_config: ScoreConfig = score_section.bind()?;
    score_config.validate()?;
    // Build layer or pipeline...
}
```

### Step 6: Optional Layer Registry

**Files to create:**
- `libs/loom-runtime/src/layer/mod.rs`
- `libs/loom-runtime/src/layer/config.rs`
- `libs/loom-runtime/src/layer/registry.rs`

```rust
// libs/loom-runtime/src/layer/config.rs
pub trait LayerConfig: DeserializeOwned + Validate + Send + Sync + 'static {
    type Layer: Layer + Send + Sync + 'static;

    fn name() -> &'static str;
    fn build(self) -> loom_error::Result<Self::Layer>;
}

// Implement for ScoreConfig
impl LayerConfig for ScoreConfig {
    type Layer = ScoreLayer;

    fn name() -> &'static str { "score" }

    fn build(self) -> loom_error::Result<Self::Layer> {
        // Existing build logic
        self.build_layer()
    }
}
```

## Files to Delete

| File | Reason |
|------|--------|
| `libs/loom-runtime/src/bench/runner/async.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/batch.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/config.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/helpers.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/instrumented.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/mod.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/pipeline.rs` | Runner abstraction no longer needed |
| `libs/loom-runtime/src/bench/runner/sync.rs` | Runner abstraction no longer needed |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-cli/src/bench/mod.rs` | Change `load_config()` return type |
| `libs/loom-cli/src/bench/run.rs` | Use `config.get_section()`, manage pipeline directly |
| `libs/loom-cli/src/bench/score.rs` | Use `config.get_section()`, manage pipeline directly |
| `libs/loom-cli/src/bench/validate.rs` | Use `config.get_section()` for layer access |
| `libs/loom-runtime/src/config.rs` | Remove `build_scorer()`, rename `build_pipeline()` → `build()`, remove `LayersConfig` |
| `libs/loom-runtime/src/bench/mod.rs` | Remove runner exports |

## Files to Create (optional registry pattern)

| File | Description |
|------|-------------|
| `libs/loom-runtime/src/layer/mod.rs` | Module exports |
| `libs/loom-runtime/src/layer/config.rs` | `LayerConfig` trait |
| `libs/loom-runtime/src/layer/registry.rs` | `LayerRegistry` type |

## Backwards Compatibility

Existing config files work unchanged:

```yaml
# This still works - layers section just accessed differently in code
concurrency: 8
batch_size: 16
layers:
  score:
    threshold: 0.75
    categories:
      sentiment:
        labels:
          positive: { hypothesis: "..." }
```

## Verification

1. `cargo build -p loom-runtime -p loom-cli -p loom-config`
2. `cargo test -p loom-runtime -p loom-config`
3. Run benchmark with existing config file
4. Test with missing `layers.score` section (should handle gracefully)
5. Test environment variable overrides still work
