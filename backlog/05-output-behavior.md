# Phase 5: Output Path Behavior

**Status: PENDING**

## Overview

Improve CLI output path handling so that:
1. `output` (config or CLI) is a **relative path to a directory**
2. Default directory = directory where the input sample file is located
3. Output files are automatically named based on the operation

## Current State

```bash
# Current: must specify full file path
loom bench score -p datasets/basic/samples.json -c config.toml -o output.json
```

## Goal State

```bash
# Input: datasets/basic/samples.json
# Output (score command): datasets/basic/scores.json (default)
# Output (run command): datasets/basic/results.json (default)

# Default behavior - output to input file's directory
loom bench score -p datasets/basic/samples.json -c config.toml
# → outputs to datasets/basic/scores.json

# Custom output directory
loom bench score -p datasets/basic/samples.json -c config.toml -o custom/
# → outputs to custom/scores.json

# Run command
loom bench run -p datasets/basic/samples.json -c config.toml
# → outputs to datasets/basic/results.json
```

## Implementation Steps

### Step 1: Create Output Path Helper

**File:** `libs/loom-cli/src/bench/mod.rs`

```rust
use std::path::{Path, PathBuf};

/// Resolve the output file path based on input path, optional output directory, and filename.
///
/// # Arguments
/// * `input_path` - Path to the input samples file
/// * `output_dir` - Optional output directory (from config or CLI)
/// * `filename` - The output filename (e.g., "scores.json", "results.json")
///
/// # Returns
/// The resolved output file path
pub fn resolve_output_path(
    input_path: &Path,
    output_dir: Option<&Path>,
    filename: &str,
) -> PathBuf {
    let base_dir = output_dir.unwrap_or_else(|| {
        input_path.parent().unwrap_or(Path::new("."))
    });
    base_dir.join(filename)
}
```

### Step 2: Update bench/score.rs

**File:** `libs/loom-cli/src/bench/score.rs`

```rust
// Determine output path
let output_path = resolve_output_path(
    path,
    output.as_ref().map(|p| p.as_path()),
    "scores.json",
);

// Ensure output directory exists
if let Some(parent) = output_path.parent() {
    std::fs::create_dir_all(parent)?;
}

// Write output
runtime
    .save("file_system", &output_path.into(), &export, Format::Json)
    .await?;

println!("Score export written to {:?}", output_path);
```

### Step 3: Update bench/run.rs

**File:** `libs/loom-cli/src/bench/run.rs`

```rust
// Determine output path (only if results need to be saved)
let output_path = resolve_output_path(
    path,
    output.as_ref().map(|p| p.as_path()),
    "results.json",
);

// Write results if needed
if save_results {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // ... save logic
}
```

### Step 4: Update Config Documentation

**File:** `libs/loom-runtime/src/config.rs`

```rust
/// Top-level configuration for Loom.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoomConfig {
    /// Output directory for results.
    /// If relative, resolved against the current working directory.
    /// If not specified, defaults to the input file's directory.
    #[serde(default)]
    pub output: Option<PathBuf>,

    // ... other fields
}
```

## File Naming Convention

| Command | Default Filename |
|---------|-----------------|
| `bench score` | `scores.json` |
| `bench run` | `results.json` |
| `bench validate` | (no output file - console only) |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-cli/src/bench/mod.rs` | Add `resolve_output_path()` helper |
| `libs/loom-cli/src/bench/run.rs` | Use new output path logic |
| `libs/loom-cli/src/bench/score.rs` | Use new output path logic |
| `libs/loom-runtime/src/config.rs` | Update `output` field documentation |

## Edge Cases

1. **Input path is just a filename** (no directory):
   - Default to current directory

2. **Output directory doesn't exist**:
   - Create it with `create_dir_all`

3. **Output is an absolute path**:
   - Use as-is (directory, not file)

## Verification

1. `cargo build -p loom-cli`
2. Test default behavior:
   ```bash
   loom bench score -p datasets/test/samples.json -c config.toml
   # Should output to datasets/test/scores.json
   ```
3. Test custom output directory:
   ```bash
   loom bench score -p datasets/test/samples.json -c config.toml -o custom/
   # Should output to custom/scores.json
   ```
4. Test run command:
   ```bash
   loom bench run -p datasets/test/samples.json -c config.toml
   # Should output to datasets/test/results.json
   ```
5. Test with input file in current directory:
   ```bash
   loom bench score -p samples.json -c config.toml
   # Should output to ./scores.json
   ```
