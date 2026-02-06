use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_valid::Validate;

/// Top-level runtime configuration for Loom.
///
/// This configuration contains runtime settings like concurrency and output paths.
/// Layer configurations (e.g., score) are accessed dynamically via `Config::get_section()`.
///
/// # Example
/// ```ignore
/// use loom_config::Config;
/// use loom_core::path::IdentPath;
///
/// let config = load_config("config.toml")?;
///
/// // Get runtime settings
/// let loom_config: LoomConfig = config.root_section().bind()?;
///
/// // Get layer config dynamically
/// let score_path = IdentPath::parse("layers.score")?;
/// let score_config: ScoreConfig = config.get_section(&score_path).bind()?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoomConfig {
    /// Output path for results
    #[serde(default)]
    pub output: Option<PathBuf>,

    /// Fail on unknown categories/labels instead of skipping
    #[serde(default)]
    pub strict: bool,

    /// Number of parallel inference workers (must be >= 1)
    #[serde(default = "LoomConfig::default_concurrency")]
    #[validate(minimum = 1)]
    pub concurrency: usize,

    /// Batch size for ML inference (must be >= 1)
    #[serde(default = "LoomConfig::default_batch_size")]
    #[validate(minimum = 1)]
    pub batch_size: usize,
}

impl LoomConfig {
    fn default_concurrency() -> usize {
        4
    }

    fn default_batch_size() -> usize {
        8
    }
}

impl Default for LoomConfig {
    fn default() -> Self {
        Self {
            output: None,
            strict: false,
            concurrency: Self::default_concurrency(),
            batch_size: Self::default_batch_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = LoomConfig::default();
        assert_eq!(config.concurrency, 4);
        assert_eq!(config.batch_size, 8);
        assert!(!config.strict);
        assert!(config.output.is_none());
    }

    #[test]
    fn default_config_validates() {
        let config = LoomConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn invalid_concurrency_fails_validation() {
        let mut config = LoomConfig::default();
        config.concurrency = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_batch_size_fails_validation() {
        let mut config = LoomConfig::default();
        config.batch_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_deserializes_with_defaults() {
        let json = r#"{}"#;
        let config: LoomConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.concurrency, 4);
        assert_eq!(config.batch_size, 8);
        assert!(!config.strict);
        assert!(config.output.is_none());
    }

    #[test]
    fn config_deserializes_with_overrides() {
        let json = r#"{
            "output": "results.json",
            "strict": true,
            "concurrency": 8,
            "batch_size": 16
        }"#;
        let config: LoomConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.concurrency, 8);
        assert_eq!(config.batch_size, 16);
        assert!(config.strict);
        assert_eq!(config.output, Some(PathBuf::from("results.json")));
    }

    #[test]
    fn config_ignores_unknown_fields() {
        // LoomConfig should deserialize successfully even with layer configs present
        // Layers are accessed separately via Config::get_section()
        let json = r#"{
            "concurrency": 8,
            "layers": {
                "score": {
                    "threshold": 0.8
                }
            }
        }"#;
        let config: LoomConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.concurrency, 8);
    }
}
