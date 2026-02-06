use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{Sample, ValidationError};

/// A benchmark dataset containing samples for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleDataset {
    pub version: String,
    pub created: String,
    pub samples: Vec<Sample>,
}

impl SampleDataset {
    /// Create a new empty dataset.
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            samples: Vec::new(),
        }
    }

    /// Validate the dataset without label validation.
    pub fn validate(&self) -> Vec<ValidationError> {
        self.validate_with_labels(None)
    }

    /// Validate the dataset with optional label validation.
    pub fn validate_with_labels(&self, valid_labels: Option<&[String]>) -> Vec<ValidationError> {
        self.validate_with_config(None, valid_labels)
    }

    /// Validate the dataset with optional category and label validation.
    ///
    /// This is the most comprehensive validation method that checks:
    /// - Duplicate sample IDs
    /// - Empty text
    /// - Missing expected labels
    /// - Invalid categories (if valid_categories is provided)
    /// - Invalid labels (if valid_labels is provided)
    pub fn validate_with_config(
        &self,
        valid_categories: Option<&[String]>,
        valid_labels: Option<&[String]>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut seen_ids = HashSet::new();

        let valid_category_set: Option<HashSet<&String>> =
            valid_categories.map(|cats| cats.iter().collect());
        let valid_label_set: Option<HashSet<&String>> =
            valid_labels.map(|labels| labels.iter().collect());

        for sample in &self.samples {
            if !seen_ids.insert(&sample.id) {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "Duplicate sample ID".to_string(),
                });
            }

            if sample.text.trim().is_empty() {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "Empty text".to_string(),
                });
            }

            if sample.expected_labels.is_empty() {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "No expected labels".to_string(),
                });
            }

            // Validate category against config
            if let Some(ref valid) = valid_category_set {
                if !valid.contains(&sample.primary_category) {
                    errors.push(ValidationError {
                        sample_id: sample.id.clone(),
                        message: format!("Invalid category: '{}'", sample.primary_category),
                    });
                }
            }

            // Validate labels against config
            if let Some(ref valid) = valid_label_set {
                for label in &sample.expected_labels {
                    if !valid.contains(label) {
                        errors.push(ValidationError {
                            sample_id: sample.id.clone(),
                            message: format!("Invalid label: '{}'", label),
                        });
                    }
                }
            }
        }

        errors
    }
}

impl Default for SampleDataset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::{Decision, Difficulty};

    use super::*;

    #[test]
    fn dataset_new_creates_empty() {
        let dataset = SampleDataset::new();
        assert_eq!(dataset.version, "1.0.0");
        assert!(dataset.samples.is_empty());
    }

    #[test]
    fn dataset_validate_catches_duplicate_ids() {
        let mut dataset = SampleDataset::new();
        dataset.samples.push(Sample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: "emotional".to_string(),
            difficulty: Difficulty::Easy,
            notes: None,
            metadata: None,
        });
        dataset.samples.push(Sample {
            id: "test-001".to_string(),
            text: "World".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: "emotional".to_string(),
            difficulty: Difficulty::Easy,
            notes: None,
            metadata: None,
        });

        let errors = dataset.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Duplicate"));
    }

    #[test]
    fn dataset_validate_catches_empty_text() {
        let mut dataset = SampleDataset::new();
        dataset.samples.push(Sample {
            id: "test-001".to_string(),
            text: "  ".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: "emotional".to_string(),
            difficulty: Difficulty::Easy,
            notes: None,
            metadata: None,
        });

        let errors = dataset.validate();
        assert!(errors.iter().any(|e| e.message.contains("Empty text")));
    }

    #[test]
    fn dataset_validate_catches_invalid_labels() {
        let mut dataset = SampleDataset::new();
        dataset.samples.push(Sample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["NotARealLabel".to_string()],
            primary_category: "emotional".to_string(),
            difficulty: Difficulty::Easy,
            notes: None,
            metadata: None,
        });

        let valid_labels = vec!["positive".to_string(), "negative".to_string()];
        let errors = dataset.validate_with_labels(Some(&valid_labels));
        assert!(errors.iter().any(|e| e.message.contains("Invalid label")));
    }

    #[test]
    fn dataset_validate_catches_invalid_categories() {
        let mut dataset = SampleDataset::new();
        dataset.samples.push(Sample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: "unknown_category".to_string(),
            difficulty: Difficulty::Easy,
            notes: None,
            metadata: None,
        });

        let valid_categories = vec!["sentiment".to_string(), "emotion".to_string()];
        let errors = dataset.validate_with_config(Some(&valid_categories), None);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("Invalid category"))
        );
    }
}
