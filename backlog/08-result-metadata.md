# Phase 08: Result Metadata

**Status:** PENDING

**Crate:** loom-runtime

**Depends on:** Phase 07 (eval module structure)

## Goal

Add timing and resource metrics to LoomResult and LayerResult by default.

## Requirements

1. **Elapsed time** - Track execution duration for each layer and overall pipeline
2. **Resource metrics** - Memory usage, CPU time (where available)
3. **Layer-specific metrics** - Model inference time, batch counts, etc.

## Changes

### LoomResult

```rust
pub struct LoomResult {
    /// Pipeline-level metadata (timing, input text, etc.)
    pub meta: Map,  // Add: elapsed_ms, total_memory_bytes, etc.
    /// Layer outputs keyed by layer name
    pub layers: BTreeMap<String, LayerResult>,
}
```

### LayerResult

```rust
pub struct LayerResult {
    /// Layer execution metadata
    pub meta: Map,  // Add: elapsed_ms, layer-specific metrics
    /// The layer output as a dynamic Value
    pub output: Value,
}
```

## Implementation

### 1. Timing Instrumentation

Wrap layer execution with timing:

```rust
impl ScoreLayer {
    pub async fn eval(&self, dataset: &SampleDataset, ...) -> EvalResult {
        let start = Instant::now();

        // ... execution logic ...

        let elapsed_ms = start.elapsed().as_millis() as u64;
        result.meta.set("elapsed_ms", elapsed_ms);
        result
    }
}
```

### 2. Resource Metrics (Optional)

Platform-specific resource collection:

```rust
#[cfg(target_os = "linux")]
fn collect_memory_stats() -> Option<MemoryStats> { ... }

#[cfg(target_os = "macos")]
fn collect_memory_stats() -> Option<MemoryStats> { ... }
```

### 3. Layer-specific Metrics

For ScoreLayer:
- `batch_count` - Number of inference batches
- `samples_per_second` - Throughput
- `model_inference_ms` - Time spent in model inference vs overhead

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-runtime/src/result.rs` | Document meta fields, add helper methods |
| `libs/loom-runtime/src/eval/score/mod.rs` | Add timing to `eval()` method |

## Verification

1. `cargo build -p loom-runtime`
2. `cargo test -p loom-runtime`
3. Run `loom run` and verify output includes timing info
4. Check `result.meta.get("elapsed_ms")` returns valid value
