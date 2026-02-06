use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{EvalResult, SampleResult};
use crate::eval::{Decision, Sample, SampleDataset};

/// Comprehensive score export with hierarchical structure.
///
/// Structure: ScoreExport -> Categories -> Labels (summary) + Samples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreExport {
    /// Total samples across all categories
    pub total: usize,
    /// Total correct predictions
    pub correct: usize,
    /// Overall accuracy (correct / total)
    pub accuracy: f32,
    /// Macro-averaged precision across labels
    pub precision: f32,
    /// Macro-averaged recall across labels
    pub recall: f32,
    /// Macro-averaged F1 score
    pub f1: f32,
    /// Categories with their label summaries and samples
    pub categories: Vec<CategoryExport>,
}

/// Category with label summaries and samples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryExport {
    /// Category name (e.g., "task", "conversational", "memory")
    pub name: String,
    /// Total samples in this category
    pub total: usize,
    /// Correct predictions in this category
    pub correct: usize,
    /// Category accuracy (correct / total)
    pub accuracy: f32,
    /// Label metrics for this category (summary only, no nested samples)
    pub labels: Vec<LabelExport>,
    /// All samples in this category
    pub samples: Vec<SampleExport>,
}

/// Label summary metrics (no nested samples).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelExport {
    /// Label name (e.g., "task", "time", "entity")
    pub name: String,
    /// Number of samples in this category that expect this label
    pub expected_count: usize,
    /// Number of samples in this category where this label was detected
    pub detected_count: usize,
    /// True positives (expected AND detected)
    pub true_positives: usize,
    /// False positives (detected but NOT expected)
    pub false_positives: usize,
    /// False negatives (expected but NOT detected)
    pub false_negatives: usize,
    /// Precision for this label in this category
    pub precision: f32,
    /// Recall for this label in this category
    pub recall: f32,
    /// F1 score for this label in this category
    pub f1: f32,
}

/// Sample with full detail including raw scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleExport {
    /// Sample identifier
    pub id: String,
    /// Original text content
    pub text: String,
    /// Final aggregated score (after calibration)
    pub score: f32,
    /// Raw scores per label (before calibration)
    pub raw_scores: HashMap<String, f32>,
    /// Expected decision (accept/reject)
    pub expected_decision: Decision,
    /// Actual decision from scorer
    pub actual_decision: Decision,
    /// Whether the overall decision was correct
    pub correct: bool,
    /// All expected labels for this sample
    pub expected_labels: Vec<String>,
    /// All detected labels for this sample
    pub detected_labels: Vec<String>,
}

impl ScoreExport {
    /// Build a ScoreExport from benchmark results.
    ///
    /// # Arguments
    /// * `dataset` - The original benchmark dataset (for sample text and category)
    /// * `result` - The benchmark results (for sample decisions and correctness)
    /// * `raw_scores` - Raw scores per sample per label (sample_id -> label -> score)
    pub fn from_results(
        dataset: &SampleDataset,
        result: &EvalResult,
        raw_scores: HashMap<String, HashMap<String, f32>>,
    ) -> Self {
        let metrics = result.metrics();

        // Build result lookup: id -> SampleResult
        let result_lookup: HashMap<&str, &SampleResult> = result
            .sample_results
            .iter()
            .map(|r| (r.id.as_str(), r))
            .collect();

        // Group samples by category
        let mut category_samples: HashMap<String, Vec<(&Sample, &SampleResult)>> = HashMap::new();

        for sample in &dataset.samples {
            if let Some(sample_result) = result_lookup.get(sample.id.as_str()) {
                category_samples
                    .entry(sample.primary_category.clone())
                    .or_default()
                    .push((sample, sample_result));
            }
        }

        // Build category exports
        let mut categories: Vec<CategoryExport> = category_samples
            .into_iter()
            .map(|(cat_name, samples)| build_category_export(&cat_name, &samples, &raw_scores))
            .collect();

        // Sort categories by name for consistent output
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        ScoreExport {
            total: result.total,
            correct: result.correct,
            accuracy: metrics.accuracy,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
            categories,
        }
    }
}

/// Build a CategoryExport from samples in that category.
fn build_category_export(
    name: &str,
    samples: &[(&Sample, &SampleResult)],
    raw_scores: &HashMap<String, HashMap<String, f32>>,
) -> CategoryExport {
    let total = samples.len();
    let correct = samples.iter().filter(|(_, r)| r.correct).count();
    let accuracy = if total > 0 {
        correct as f32 / total as f32
    } else {
        0.0
    };

    // Compute per-label metrics scoped to this category
    let mut label_stats: HashMap<String, LabelStats> = HashMap::new();

    for (sample, result) in samples {
        let expected_set: HashSet<&String> = sample.expected_labels.iter().collect();
        let detected_set: HashSet<&String> = result.detected_labels.iter().collect();

        // Track expected labels
        for label in &sample.expected_labels {
            let stats = label_stats.entry(label.clone()).or_default();
            stats.expected_count += 1;
            if detected_set.contains(label) {
                stats.true_positives += 1;
            } else {
                stats.false_negatives += 1;
            }
        }

        // Track detected labels (for false positives)
        for label in &result.detected_labels {
            let stats = label_stats.entry(label.clone()).or_default();
            stats.detected_count += 1;
            if !expected_set.contains(label) {
                stats.false_positives += 1;
            }
        }
    }

    // Convert to LabelExport with computed metrics
    let mut labels: Vec<LabelExport> = label_stats
        .into_iter()
        .map(|(label_name, stats)| {
            let precision = if stats.true_positives + stats.false_positives > 0 {
                stats.true_positives as f32 / (stats.true_positives + stats.false_positives) as f32
            } else {
                0.0
            };
            let recall = if stats.true_positives + stats.false_negatives > 0 {
                stats.true_positives as f32 / (stats.true_positives + stats.false_negatives) as f32
            } else {
                0.0
            };
            let f1 = if precision + recall > 0.0 {
                2.0 * precision * recall / (precision + recall)
            } else {
                0.0
            };

            LabelExport {
                name: label_name,
                expected_count: stats.expected_count,
                detected_count: stats.detected_count,
                true_positives: stats.true_positives,
                false_positives: stats.false_positives,
                false_negatives: stats.false_negatives,
                precision,
                recall,
                f1,
            }
        })
        .collect();

    // Sort labels by name for consistent output
    labels.sort_by(|a, b| a.name.cmp(&b.name));

    // Build sample exports
    let sample_exports: Vec<SampleExport> = samples
        .iter()
        .map(|(sample, result)| {
            let sample_raw_scores = raw_scores.get(&sample.id).cloned().unwrap_or_default();

            SampleExport {
                id: sample.id.clone(),
                text: sample.text.clone(),
                score: result.score,
                raw_scores: sample_raw_scores,
                expected_decision: result.expected_decision,
                actual_decision: result.actual_decision,
                correct: result.correct,
                expected_labels: result.expected_labels.clone(),
                detected_labels: result.detected_labels.clone(),
            }
        })
        .collect();

    CategoryExport {
        name: name.to_string(),
        total,
        correct,
        accuracy,
        labels,
        samples: sample_exports,
    }
}

/// Internal struct for accumulating label statistics.
#[derive(Default)]
struct LabelStats {
    expected_count: usize,
    detected_count: usize,
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
}
