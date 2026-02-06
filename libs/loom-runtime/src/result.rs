//! Result types for loom-runtime pipeline output.
//!
//! # Metadata Key Standards
//!
//! The following metadata keys are used across layers and results:
//!
//! | Key | Type | Description |
//! |-----|------|-------------|
//! | `elapsed_ms` | `i64` | Execution time in milliseconds |
//! | `start_time` | `String` | ISO-8601 timestamp of execution start |
//! | `step` | `i64` | Processing step number in pipeline |
//! | `text` | `String` | Input text that was processed |
//! | `inference_ms` | `i64` | Model inference time only (excludes overhead) |
//! | `batch_count` | `i64` | Number of batches processed |
//!
//! # Example
//!
//! ```ignore
//! let result = layer.invoke(ctx)?;
//! let elapsed = result.meta.get("elapsed_ms"); // i64 milliseconds
//! let start = result.meta.get("start_time");   // ISO-8601 string
//! ```

use std::collections::BTreeMap;

use loom_core::{Map, value::Value};
use serde::{Deserialize, Serialize};

use crate::eval::score::ScoreResult;

/// Top-level result aggregating all layer outputs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoomResult {
    /// Pipeline-level metadata.
    ///
    /// Standard keys: `elapsed_ms`, `start_time`, `text`
    #[serde(default)]
    pub meta: Map,

    /// Layer outputs keyed by layer name (e.g., "score", "intent", "entity")
    #[serde(default)]
    pub layers: BTreeMap<String, LayerResult>,
}

impl LoomResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a layer result
    pub fn with_layer(mut self, name: impl Into<String>, result: LayerResult) -> Self {
        self.layers.insert(name.into(), result);
        self
    }

    /// Set metadata
    pub fn with_meta(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.meta.set(key, value.into());
        self
    }

    /// Get a layer result by name
    pub fn layer(&self, name: &str) -> Option<&LayerResult> {
        self.layers.get(name)
    }

    /// Get a mutable layer result by name
    pub fn layer_mut(&mut self, name: &str) -> Option<&mut LayerResult> {
        self.layers.get_mut(name)
    }

    /// Get score layer result (typed)
    #[cfg(feature = "json")]
    pub fn score(&self) -> Option<ScoreResult> {
        self.layer("score").and_then(|lr| {
            // Only extract if output is Ok
            lr.output.as_ref().ok().and_then(|output| {
                // Convert loom_core::Value to serde_json::Value, then deserialize
                let json: serde_json::Value = output.into();
                serde_json::from_value(json).ok()
            })
        })
    }

    /// Create a LoomResult from a ScoreResult.
    ///
    /// This is a convenience method for converting pipeline output to LoomResult.
    /// The score result is stored under the "score" layer key.
    ///
    /// # Example
    /// ```ignore
    /// let pipeline = config.build()?;
    /// let score_result = pipeline.execute(context)?;
    /// let loom_result = LoomResult::from_score(score_result);
    /// ```
    pub fn from_score(score_result: ScoreResult) -> Self {
        Self::from_score_with_meta(score_result, Map::new())
    }

    /// Create a LoomResult from a ScoreResult with additional metadata.
    ///
    /// # Arguments
    /// * `score_result` - The score result to wrap
    /// * `meta` - Pipeline-level metadata (e.g., timing, input text)
    pub fn from_score_with_meta(score_result: ScoreResult, meta: Map) -> Self {
        let layer_result = LayerResult::new(score_result);
        Self::new()
            .with_meta_map(meta)
            .with_layer("score", layer_result)
    }

    /// Set metadata from a Map
    fn with_meta_map(mut self, meta: Map) -> Self {
        self.meta = meta;
        self
    }
}

impl From<ScoreResult> for LoomResult {
    fn from(score_result: ScoreResult) -> Self {
        Self::from_score(score_result)
    }
}

/// Generic layer result that can store any layer output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    /// Layer execution metadata.
    ///
    /// Standard keys: `elapsed_ms`, `start_time`, `step`, `text`, `inference_ms`
    #[serde(default)]
    pub meta: Map,
    /// The layer output (Ok) or error (Err).
    pub output: loom_error::Result<Value>,
    /// Child results for sub-operations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<loom_error::Result<Value>>,
}

impl LayerResult {
    /// Create a new successful result.
    pub fn new(output: impl Into<Value>) -> Self {
        Self {
            meta: Map::new(),
            output: Ok(output.into()),
            children: Vec::new(),
        }
    }

    /// Create a new error result.
    pub fn from_error(error: loom_error::Error) -> Self {
        Self {
            meta: Map::new(),
            output: Err(error),
            children: Vec::new(),
        }
    }

    /// Set metadata
    pub fn with_meta(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.meta.set(key, value.into());
        self
    }

    /// Set the output to a successful value.
    pub fn ok(mut self, output: impl Into<Value>) -> Self {
        self.output = Ok(output.into());
        self
    }

    /// Set the output to an error.
    pub fn error(mut self, error: loom_error::Error) -> Self {
        self.output = Err(error);
        self
    }

    /// Add a child result.
    pub fn add(mut self, child: loom_error::Result<Value>) -> Self {
        self.children.push(child);
        self
    }

    /// Returns true if output is Ok and all children are Ok.
    pub fn is_ok(&self) -> bool {
        self.output.is_ok() && self.children.iter().all(|c| c.is_ok())
    }
}

impl Default for LayerResult {
    fn default() -> Self {
        Self {
            meta: Map::new(),
            output: Ok(Value::Null),
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn loom_result_default() {
        let result = LoomResult::new();
        assert!(result.layers.is_empty());
    }

    #[test]
    fn loom_result_with_layer() {
        let layer = LayerResult::new(Value::String("test".to_string()));
        let result = LoomResult::new().with_layer("score", layer);

        assert!(result.layer("score").is_some());
        assert!(result.layer("nonexistent").is_none());
    }

    #[test]
    fn layer_result_with_meta() {
        let layer = LayerResult::new(Value::Null)
            .with_meta("duration_ms", 100i64)
            .with_meta("model", "bart");

        assert!(layer.meta.exists("duration_ms"));
        assert!(layer.meta.exists("model"));
    }

    #[test]
    fn loom_result_from_score() {
        let score_result = ScoreResult::new(BTreeMap::new());
        let loom_result = LoomResult::from_score(score_result);

        assert!(loom_result.layer("score").is_some());
        assert!(loom_result.score().is_some());
    }

    #[test]
    fn loom_result_from_score_with_meta() {
        let score_result = ScoreResult::new(BTreeMap::new());
        let mut meta = Map::new();
        meta.set("text", "test input".into());

        let loom_result = LoomResult::from_score_with_meta(score_result, meta);

        assert!(loom_result.layer("score").is_some());
        assert!(loom_result.meta.exists("text"));
    }

    #[test]
    fn score_result_into_loom_result() {
        let score_result = ScoreResult::new(BTreeMap::new());
        let loom_result: LoomResult = score_result.into();

        assert!(loom_result.layer("score").is_some());
    }

    #[test]
    fn layer_result_ok() {
        let result = LayerResult::new("success");
        assert!(result.is_ok());
    }

    #[test]
    fn layer_result_error() {
        let err = loom_error::Error::builder()
            .code(loom_error::ErrorCode::NotFound)
            .message("not found")
            .build();
        let result = LayerResult::from_error(err);
        assert!(!result.is_ok());
    }

    #[test]
    fn layer_result_with_children() {
        let result = LayerResult::new(Value::Null)
            .add(Ok("child1".into()))
            .add(Err(loom_error::Error::builder()
                .message("child failed")
                .build()));

        assert!(!result.is_ok()); // has an error child
    }

    #[test]
    fn layer_result_all_children_ok() {
        let result = LayerResult::new(Value::Null)
            .add(Ok("first".into()))
            .add(Ok("second".into()));

        assert!(result.is_ok()); // output ok + all children ok
    }

    #[test]
    fn layer_result_ok_error_methods() {
        let result = LayerResult::default()
            .ok("value")
            .error(loom_error::Error::builder().message("failed").build());

        assert!(!result.is_ok()); // last call was error()
    }
}
