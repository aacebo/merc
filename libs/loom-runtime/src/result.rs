use std::collections::BTreeMap;

use loom_core::{Map, value::Value};
use serde::{Deserialize, Serialize};

use crate::eval::score::ScoreResult;

/// Top-level result aggregating all layer outputs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoomResult {
    /// Pipeline-level metadata (timing, input text, etc.)
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
            // Convert loom_core::Value to serde_json::Value, then deserialize
            let json: serde_json::Value = (&lr.output).into();
            serde_json::from_value(json).ok()
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

/// Generic layer result that can store any layer output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    /// Layer execution metadata (duration_ms, model, etc.)
    #[serde(default)]
    pub meta: Map,
    /// The layer output as a dynamic Value (allows any layer type)
    pub output: Value,
}

impl LayerResult {
    pub fn new(output: impl Into<Value>) -> Self {
        Self {
            meta: Map::new(),
            output: output.into(),
        }
    }

    /// Set metadata
    pub fn with_meta(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.meta.set(key, value.into());
        self
    }
}

impl Default for LayerResult {
    fn default() -> Self {
        Self {
            meta: Map::new(),
            output: Value::Null,
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
}
