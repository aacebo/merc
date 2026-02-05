use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{BenchSample, CoverageReport, Decision, ValidationError};

/// A benchmark dataset containing samples for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchDataset {
    pub version: String,
    pub created: String,
    pub samples: Vec<BenchSample>,
}

impl BenchDataset {
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
        let mut errors = Vec::new();
        let mut seen_ids = HashSet::new();

        let valid_set: Option<HashSet<&String>> =
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

            if let Some(ref valid) = valid_set {
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

    /// Get coverage report without label coverage.
    pub fn coverage(&self) -> CoverageReport {
        self.coverage_with_labels(None)
    }

    /// Get coverage report with optional label coverage detection.
    pub fn coverage_with_labels(&self, all_labels: Option<&[String]>) -> CoverageReport {
        let mut report = CoverageReport {
            total_samples: self.samples.len(),
            ..Default::default()
        };

        for sample in &self.samples {
            *report
                .samples_by_category
                .entry(sample.primary_category)
                .or_insert(0) += 1;

            for label in &sample.expected_labels {
                *report.samples_by_label.entry(label.clone()).or_insert(0) += 1;
            }

            match sample.expected_decision {
                Decision::Accept => report.accept_count += 1,
                Decision::Reject => report.reject_count += 1,
            }
        }

        if let Some(labels) = all_labels {
            for label in labels {
                if !report.samples_by_label.contains_key(label) {
                    report.missing_labels.push(label.clone());
                }
            }
            report.missing_labels.sort();
        }

        report
    }
}

impl Default for BenchDataset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::{Category, Difficulty};

    #[test]
    fn dataset_new_creates_empty() {
        let dataset = BenchDataset::new();
        assert_eq!(dataset.version, "1.0.0");
        assert!(dataset.samples.is_empty());
    }

    #[test]
    fn dataset_validate_catches_duplicate_ids() {
        let mut dataset = BenchDataset::new();
        dataset.samples.push(BenchSample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: Category::Emotional,
            difficulty: Difficulty::Easy,
            notes: None,
        });
        dataset.samples.push(BenchSample {
            id: "test-001".to_string(),
            text: "World".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: Category::Emotional,
            difficulty: Difficulty::Easy,
            notes: None,
        });

        let errors = dataset.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Duplicate"));
    }

    #[test]
    fn dataset_validate_catches_empty_text() {
        let mut dataset = BenchDataset::new();
        dataset.samples.push(BenchSample {
            id: "test-001".to_string(),
            text: "  ".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: Category::Emotional,
            difficulty: Difficulty::Easy,
            notes: None,
        });

        let errors = dataset.validate();
        assert!(errors.iter().any(|e| e.message.contains("Empty text")));
    }

    #[test]
    fn dataset_validate_catches_invalid_labels() {
        let mut dataset = BenchDataset::new();
        dataset.samples.push(BenchSample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["NotARealLabel".to_string()],
            primary_category: Category::Emotional,
            difficulty: Difficulty::Easy,
            notes: None,
        });

        let valid_labels = vec!["positive".to_string(), "negative".to_string()];
        let errors = dataset.validate_with_labels(Some(&valid_labels));
        assert!(errors.iter().any(|e| e.message.contains("Invalid label")));
    }

    #[test]
    fn dataset_coverage_counts_correctly() {
        let mut dataset = BenchDataset::new();
        dataset.samples.push(BenchSample {
            id: "test-001".to_string(),
            text: "Hello".to_string(),
            context: None,
            expected_decision: Decision::Accept,
            expected_labels: vec!["positive".to_string()],
            primary_category: Category::Emotional,
            difficulty: Difficulty::Easy,
            notes: None,
        });
        dataset.samples.push(BenchSample {
            id: "test-002".to_string(),
            text: "Hi".to_string(),
            context: None,
            expected_decision: Decision::Reject,
            expected_labels: vec!["phatic".to_string()],
            primary_category: Category::Phatic,
            difficulty: Difficulty::Easy,
            notes: None,
        });

        let coverage = dataset.coverage();
        assert_eq!(coverage.total_samples, 2);
        assert_eq!(coverage.accept_count, 1);
        assert_eq!(coverage.reject_count, 1);
    }
}
