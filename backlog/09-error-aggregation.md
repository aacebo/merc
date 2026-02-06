# Phase 09: Error Aggregation

**Status:** PENDING

**Crate:** loom-runtime

**Depends on:** Phase 08 (result metadata)

## Goal

Support hierarchical errors in layer results using loom-error.

## Requirements

1. **Child results** - LayerResult can contain sub-results (value or error)
2. **Hierarchical errors** - Use loom-error for proper error chains
3. **Partial failures** - Track which parts of a layer succeeded/failed

## Changes

### LayerResult

```rust
pub struct LayerResult {
    /// Layer execution metadata
    pub meta: Map,
    /// The primary layer output
    pub output: Value,
    /// Child results for sub-operations (NEW)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ChildResult>,
}
```

### ChildResult

```rust
/// Result of a sub-operation within a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildResult {
    /// Successful sub-operation
    Ok {
        name: String,
        output: Value,
    },
    /// Failed sub-operation
    Err {
        name: String,
        error: ErrorInfo,
    },
}

/// Serializable error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub message: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<ErrorInfo>,
}
```

## Implementation

### 1. Convert loom_error::Error to ErrorInfo

```rust
impl From<&loom_error::Error> for ErrorInfo {
    fn from(err: &loom_error::Error) -> Self {
        Self {
            message: err.message().to_string(),
            kind: err.kind().to_string(),
            causes: err.causes().iter().map(ErrorInfo::from).collect(),
        }
    }
}
```

### 2. Use in ScoreLayer

```rust
impl ScoreLayer {
    pub async fn eval(&self, dataset: &SampleDataset, ...) -> EvalResult {
        let mut children = Vec::new();

        for sample in dataset.samples.iter() {
            match self.score_sample(sample).await {
                Ok(result) => children.push(ChildResult::Ok {
                    name: sample.id.clone(),
                    output: result.into(),
                }),
                Err(e) => children.push(ChildResult::Err {
                    name: sample.id.clone(),
                    error: ErrorInfo::from(&e),
                }),
            }
        }

        // ... build result with children ...
    }
}
```

## Benefits

- Track partial failures within a layer
- Error hierarchy preserved via loom-error
- Better debugging for complex pipelines
- JSON-serializable error information

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-runtime/src/result.rs` | Add ChildResult, ErrorInfo types |
| `libs/loom-runtime/src/eval/score/mod.rs` | Collect child results during eval |

## Verification

1. `cargo build -p loom-runtime`
2. `cargo test -p loom-runtime`
3. Test with dataset containing invalid samples
4. Verify error hierarchy is preserved in output
