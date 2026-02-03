use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::score::Label;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchDataset {
    pub version: String,
    pub created: String,
    pub samples: Vec<BenchSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSample {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub expected_decision: Decision,
    pub expected_labels: Vec<String>,
    pub primary_category: Category,
    pub difficulty: Difficulty,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Task,
    Emotional,
    Factual,
    Preference,
    Decision,
    Phatic,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug)]
pub struct ValidationError {
    pub sample_id: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.sample_id, self.message)
    }
}

#[derive(Debug, Default)]
pub struct CoverageReport {
    pub total_samples: usize,
    pub samples_by_category: std::collections::HashMap<Category, usize>,
    pub samples_by_label: std::collections::HashMap<String, usize>,
    pub missing_labels: Vec<String>,
    pub accept_count: usize,
    pub reject_count: usize,
}

impl BenchDataset {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            samples: Vec::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path.as_ref()).map_err(|e| format!("Failed to read file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(path.as_ref(), content).map_err(|e| format!("Failed to write file: {}", e))
    }

    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut seen_ids = HashSet::new();
        let valid_labels: HashSet<String> = Label::all().iter().map(|l| l.to_string()).collect();

        for sample in &self.samples {
            // Check for duplicate IDs
            if !seen_ids.insert(&sample.id) {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "Duplicate sample ID".to_string(),
                });
            }

            // Check for empty text
            if sample.text.trim().is_empty() {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "Empty text".to_string(),
                });
            }

            // Check for empty expected_labels
            if sample.expected_labels.is_empty() {
                errors.push(ValidationError {
                    sample_id: sample.id.clone(),
                    message: "No expected labels".to_string(),
                });
            }

            // Validate label names
            for label in &sample.expected_labels {
                if !valid_labels.contains(label) {
                    errors.push(ValidationError {
                        sample_id: sample.id.clone(),
                        message: format!("Invalid label: '{}'", label),
                    });
                }
            }
        }

        errors
    }

    pub fn coverage(&self) -> CoverageReport {
        let mut report = CoverageReport {
            total_samples: self.samples.len(),
            ..Default::default()
        };

        // Count by category
        for sample in &self.samples {
            *report
                .samples_by_category
                .entry(sample.primary_category)
                .or_insert(0) += 1;

            // Count by label
            for label in &sample.expected_labels {
                *report.samples_by_label.entry(label.clone()).or_insert(0) += 1;
            }

            // Count decisions
            match sample.expected_decision {
                Decision::Accept => report.accept_count += 1,
                Decision::Reject => report.reject_count += 1,
            }
        }

        // Find missing labels
        let all_labels: HashSet<String> = Label::all().iter().map(|l| l.to_string()).collect();

        for label in all_labels {
            if !report.samples_by_label.contains_key(&label) {
                report.missing_labels.push(label);
            }
        }
        report.missing_labels.sort();

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

        let errors = dataset.validate();
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
        assert_eq!(
            *coverage
                .samples_by_category
                .get(&Category::Emotional)
                .unwrap(),
            1
        );
        assert_eq!(
            *coverage.samples_by_category.get(&Category::Phatic).unwrap(),
            1
        );
    }

    #[test]
    fn decision_serializes_lowercase() {
        let accept = serde_json::to_string(&Decision::Accept).unwrap();
        let reject = serde_json::to_string(&Decision::Reject).unwrap();
        assert_eq!(accept, "\"accept\"");
        assert_eq!(reject, "\"reject\"");
    }

    #[test]
    fn category_serializes_lowercase() {
        let task = serde_json::to_string(&Category::Task).unwrap();
        assert_eq!(task, "\"task\"");
    }

    #[test]
    fn difficulty_serializes_lowercase() {
        let easy = serde_json::to_string(&Difficulty::Easy).unwrap();
        assert_eq!(easy, "\"easy\"");
    }
}
