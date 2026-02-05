mod category;
mod label;
mod modifier;

pub use category::*;
pub use label::*;
pub use modifier::*;

use serde::{Deserialize, Serialize};
use serde_valid::Validate;

/// Root configuration for the scoring engine
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ScoreConfig {
    /// Baseline threshold for overall score acceptance
    #[serde(default = "ScoreConfig::threshold")]
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub threshold: f32,

    /// Number of top labels to consider per category (default)
    #[serde(default = "ScoreConfig::top_k")]
    #[validate(minimum = 1)]
    pub top_k: usize,

    /// Dynamic threshold adjustments based on text length
    #[serde(default)]
    #[validate]
    pub modifiers: ScoreModifierConfig,

    /// Category definitions with their labels
    #[validate]
    #[validate(min_items = 1)]
    pub categories: Vec<ScoreCategoryConfig>,
}

impl ScoreConfig {
    fn threshold() -> f32 {
        0.75
    }

    fn top_k() -> usize {
        2
    }

    /// Compute effective threshold based on text length
    pub fn threshold_of(&self, text_len: usize) -> f32 {
        match text_len {
            len if len <= self.modifiers.short_text_limit => {
                self.threshold - self.modifiers.short_text_delta
            }
            len if len > self.modifiers.long_text_limit => {
                self.threshold + self.modifiers.long_text_delta
            }
            _ => self.threshold,
        }
    }

    /// Get a category by name
    pub fn category(&self, name: &str) -> Option<&ScoreCategoryConfig> {
        self.categories.iter().find(|c| c.name == name)
    }

    /// Get a label by name across all categories
    pub fn label(&self, name: &str) -> Option<&ScoreLabelConfig> {
        self.categories
            .iter()
            .flat_map(|c| &c.labels)
            .find(|l| l.name == name)
    }

    /// Get all labels across all categories
    pub fn labels(&self) -> Vec<ScoreLabelConfig> {
        self.categories
            .iter()
            .flat_map(|c| c.labels.clone())
            .collect()
    }

    /// Get hypothesis for a label by name
    pub fn hypothesis(&self, label_name: &str) -> String {
        self.label(label_name)
            .map(|l| l.hypothesis.clone())
            .unwrap_or_else(|| format!("This example is {}.", label_name))
    }
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            threshold: Self::threshold(),
            top_k: Self::top_k(),
            modifiers: ScoreModifierConfig::default(),
            categories: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ScoreConfig {
        ScoreConfig {
            threshold: 0.75,
            top_k: 2,
            modifiers: ScoreModifierConfig::default(),
            categories: vec![ScoreCategoryConfig {
                name: "test".to_string(),
                top_k: 2,
                labels: vec![
                    ScoreLabelConfig {
                        name: "label1".to_string(),
                        hypothesis: "Test hypothesis 1".to_string(),
                        weight: 0.50,
                        threshold: 0.70,
                        platt_a: 1.0,
                        platt_b: 0.0,
                    },
                    ScoreLabelConfig {
                        name: "label2".to_string(),
                        hypothesis: "Test hypothesis 2".to_string(),
                        weight: 0.80,
                        threshold: 0.65,
                        platt_a: 1.0,
                        platt_b: 0.0,
                    },
                ],
            }],
        }
    }

    #[test]
    fn default_config_has_empty_categories() {
        let config = ScoreConfig::default();
        assert!(config.categories.is_empty());
        assert_eq!(config.threshold, 0.75);
        assert_eq!(config.top_k, 2);
    }

    #[test]
    fn threshold_of_short_text() {
        let config = test_config();
        let threshold = config.threshold_of(10);
        assert!((threshold - 0.70).abs() < f32::EPSILON);
    }

    #[test]
    fn threshold_of_medium_text() {
        let config = test_config();
        let threshold = config.threshold_of(100);
        assert!((threshold - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn threshold_of_long_text() {
        let config = test_config();
        let threshold = config.threshold_of(250);
        assert!((threshold - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn label_lookup_works() {
        let config = test_config();
        let label = config.label("label2");
        assert!(label.is_some());
        assert_eq!(label.unwrap().weight, 0.80);
    }

    #[test]
    fn category_lookup_works() {
        let config = test_config();
        let category = config.category("test");
        assert!(category.is_some());
        assert_eq!(category.unwrap().labels.len(), 2);
    }

    #[test]
    fn invalid_threshold_fails_validation() {
        let mut config = test_config();
        config.threshold = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_weight_fails_validation() {
        let mut config = test_config();
        config.categories[0].labels[0].weight = -0.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_categories_fails_validation() {
        let config = ScoreConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_labels_fails_validation() {
        let mut config = test_config();
        config.categories[0].labels.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn short_name_fails_validation() {
        let mut config = test_config();
        config.categories[0].name = "ab".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn label_config_uses_defaults() {
        let json = r#"{"name": "test", "hypothesis": "Test hypothesis"}"#;
        let label: ScoreLabelConfig = serde_json::from_str(json).unwrap();

        assert_eq!(label.name, "test");
        assert_eq!(label.hypothesis, "Test hypothesis");
        assert_eq!(label.weight, 0.50);
        assert_eq!(label.threshold, 0.70);
        assert_eq!(label.platt_a, 1.0);
        assert_eq!(label.platt_b, 0.0);
    }

    #[test]
    fn score_config_uses_defaults() {
        let json = r#"{
            "categories": [{
                "name": "test",
                "labels": [{"name": "label1", "hypothesis": "Test"}]
            }]
        }"#;
        let config: ScoreConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.threshold, 0.75);
        assert_eq!(config.top_k, 2);
        assert_eq!(config.modifiers.short_text_delta, 0.05);
        assert_eq!(config.modifiers.long_text_delta, 0.05);
        assert!(config.validate().is_ok());
    }
}
