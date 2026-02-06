use std::collections::BTreeMap;

use loom_core::{Map, decode, encode, value::Value};
use serde::{Deserialize, Serialize};

use crate::score::ScoreResult;

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
    pub fn score(&self) -> Option<ScoreResult> {
        self.layer("score").and_then(|lr| {
            let json_str = encode!(&lr.output; json).ok()?;
            decode!(&json_str; json).ok()
        })
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
}
