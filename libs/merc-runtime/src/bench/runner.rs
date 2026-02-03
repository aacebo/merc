use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Context;
use crate::score::{Label, ScoreLayer, ScoreOptions, ScoreResult};

use super::dataset::{BenchDataset, BenchSample, Category, Decision};

#[derive(Debug, Clone)]
pub struct BenchResult {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub per_category: HashMap<Category, CategoryResult>,
    pub per_label: HashMap<String, LabelResult>,
    pub sample_results: Vec<SampleResult>,
}

#[derive(Debug, Clone, Default)]
pub struct CategoryResult {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LabelResult {
    pub expected_count: usize,
    pub detected_count: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
}

#[derive(Debug, Clone)]
pub struct SampleResult {
    pub id: String,
    pub expected_decision: Decision,
    pub actual_decision: Decision,
    pub correct: bool,
    pub score: f32,
    pub expected_labels: Vec<String>,
    pub detected_labels: Vec<String>,
}

impl BenchResult {
    pub fn new() -> Self {
        Self {
            total: 0,
            correct: 0,
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
            per_category: HashMap::new(),
            per_label: HashMap::new(),
            sample_results: Vec::new(),
        }
    }

    fn compute_metrics(&mut self) {
        // Overall accuracy
        if self.total > 0 {
            self.accuracy = self.correct as f32 / self.total as f32;
        }

        // Per-category accuracy
        for result in self.per_category.values_mut() {
            if result.total > 0 {
                result.accuracy = result.correct as f32 / result.total as f32;
            }
        }

        // Per-label precision/recall/F1
        let mut total_precision = 0.0;
        let mut total_recall = 0.0;
        let mut label_count = 0;

        for result in self.per_label.values_mut() {
            // Precision = TP / (TP + FP)
            let tp_fp = result.true_positives + result.false_positives;
            if tp_fp > 0 {
                result.precision = result.true_positives as f32 / tp_fp as f32;
            }

            // Recall = TP / (TP + FN)
            let tp_fn = result.true_positives + result.false_negatives;
            if tp_fn > 0 {
                result.recall = result.true_positives as f32 / tp_fn as f32;
            }

            // F1 = 2 * (precision * recall) / (precision + recall)
            let pr_sum = result.precision + result.recall;
            if pr_sum > 0.0 {
                result.f1 = 2.0 * result.precision * result.recall / pr_sum;
            }

            if result.expected_count > 0 {
                total_precision += result.precision;
                total_recall += result.recall;
                label_count += 1;
            }
        }

        // Macro-averaged precision/recall/F1
        if label_count > 0 {
            self.precision = total_precision / label_count as f32;
            self.recall = total_recall / label_count as f32;
            let pr_sum = self.precision + self.recall;
            if pr_sum > 0.0 {
                self.f1 = 2.0 * self.precision * self.recall / pr_sum;
            }
        }
    }
}

impl Default for BenchResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress information passed to the callback
#[derive(Debug, Clone)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub sample_id: String,
    pub correct: bool,
}

pub fn run(dataset: &BenchDataset, layer: &ScoreLayer) -> BenchResult {
    run_with_progress(dataset, layer, |_| {})
}

pub fn run_with_progress<F>(
    dataset: &BenchDataset,
    layer: &ScoreLayer,
    on_progress: F,
) -> BenchResult
where
    F: Fn(Progress),
{
    let mut result = BenchResult::new();
    result.total = dataset.samples.len();

    for (i, sample) in dataset.samples.iter().enumerate() {
        let sample_result = evaluate_sample(sample, layer);

        // Report progress
        on_progress(Progress {
            current: i + 1,
            total: result.total,
            sample_id: sample.id.clone(),
            correct: sample_result.correct,
        });

        // Update overall counts
        if sample_result.correct {
            result.correct += 1;
        }

        // Update per-category counts
        let cat_result = result
            .per_category
            .entry(sample.primary_category)
            .or_default();
        cat_result.total += 1;
        if sample_result.correct {
            cat_result.correct += 1;
        }

        // Update per-label counts
        update_label_metrics(&mut result.per_label, sample, &sample_result);

        result.sample_results.push(sample_result);
    }

    result.compute_metrics();
    result
}

pub fn run_with_options(
    dataset: &BenchDataset,
    options: ScoreOptions,
) -> Result<BenchResult, String> {
    run_with_options_and_progress(dataset, options, |_| {})
}

pub fn run_with_options_and_progress<F>(
    dataset: &BenchDataset,
    options: ScoreOptions,
    on_progress: F,
) -> Result<BenchResult, String>
where
    F: Fn(Progress),
{
    let layer = options
        .build()
        .map_err(|e| format!("Failed to build scorer: {}", e))?;
    Ok(run_with_progress(dataset, &layer, on_progress))
}

fn evaluate_sample(sample: &BenchSample, layer: &ScoreLayer) -> SampleResult {
    let ctx = Context::new(&sample.text, ());
    let invoke_result = layer.invoke(ctx);

    let (actual_decision, score, detected_labels) = match invoke_result {
        Ok(layer_result) => {
            let detected = extract_detected_labels(&layer_result.output);
            (Decision::Accept, layer_result.output.score, detected)
        }
        Err(_) => {
            // Rejection (Cancel error code or other error)
            (Decision::Reject, 0.0, vec![])
        }
    };

    SampleResult {
        id: sample.id.clone(),
        expected_decision: sample.expected_decision,
        actual_decision,
        correct: actual_decision == sample.expected_decision,
        score,
        expected_labels: sample.expected_labels.clone(),
        detected_labels,
    }
}

fn extract_detected_labels(score_result: &ScoreResult) -> Vec<String> {
    score_result
        .labels()
        .iter()
        .filter(|l| l.score > 0.0)
        .map(|l| l.label.to_string())
        .collect()
}

fn update_label_metrics(
    per_label: &mut HashMap<String, LabelResult>,
    sample: &BenchSample,
    sample_result: &SampleResult,
) {
    let expected_set: std::collections::HashSet<_> = sample.expected_labels.iter().collect();
    let detected_set: std::collections::HashSet<_> = sample_result.detected_labels.iter().collect();

    // Update expected counts
    for label in &sample.expected_labels {
        let entry = per_label.entry(label.clone()).or_default();
        entry.expected_count += 1;
    }

    // Update detected counts and TP/FP/FN
    for label in &sample_result.detected_labels {
        let entry = per_label.entry(label.clone()).or_default();
        entry.detected_count += 1;

        if expected_set.contains(label) {
            entry.true_positives += 1;
        } else {
            entry.false_positives += 1;
        }
    }

    // Count false negatives (expected but not detected)
    for label in &sample.expected_labels {
        if !detected_set.contains(label) {
            let entry = per_label.entry(label.clone()).or_default();
            entry.false_negatives += 1;
        }
    }
}

// === Raw Score Export for Platt Calibration Training ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawScoreExport {
    pub samples: Vec<SampleScores>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleScores {
    pub id: String,
    pub text: String,
    pub scores: HashMap<String, f32>,
    pub expected_labels: Vec<String>,
}

/// Export raw (uncalibrated) scores for all labels on each sample.
/// Used for training Platt calibration parameters.
pub fn export_raw_scores(
    dataset: &BenchDataset,
    layer: &ScoreLayer,
    on_progress: impl Fn(Progress),
) -> RawScoreExport {
    let mut samples = Vec::with_capacity(dataset.samples.len());
    let total = dataset.samples.len();

    for (i, sample) in dataset.samples.iter().enumerate() {
        let ctx = Context::new(&sample.text, ());
        let mut scores = HashMap::new();

        // Initialize all labels with 0.0
        for label in Label::all() {
            scores.insert(label.to_string(), 0.0);
        }

        // Run scoring and collect raw scores
        if let Ok(layer_result) = layer.invoke(ctx) {
            for score_label in layer_result.output.labels() {
                scores.insert(score_label.label.to_string(), score_label.raw_score);
            }
        }

        on_progress(Progress {
            current: i + 1,
            total,
            sample_id: sample.id.clone(),
            correct: true,
        });

        samples.push(SampleScores {
            id: sample.id.clone(),
            text: sample.text.clone(),
            scores,
            expected_labels: sample.expected_labels.clone(),
        });
    }

    RawScoreExport { samples }
}

/// Export raw scores with options (builds the scorer internally).
pub fn export_raw_scores_with_options(
    dataset: &BenchDataset,
    options: ScoreOptions,
    on_progress: impl Fn(Progress),
) -> Result<RawScoreExport, String> {
    let layer = options
        .build()
        .map_err(|e| format!("Failed to build scorer: {}", e))?;
    Ok(export_raw_scores(dataset, &layer, on_progress))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_result_computes_accuracy() {
        let mut result = BenchResult::new();
        result.total = 10;
        result.correct = 8;
        result.compute_metrics();
        assert!((result.accuracy - 0.8).abs() < 0.001);
    }

    #[test]
    fn category_result_computes_accuracy() {
        let mut result = BenchResult::new();
        result.per_category.insert(
            Category::Task,
            CategoryResult {
                total: 5,
                correct: 4,
                accuracy: 0.0,
            },
        );
        result.compute_metrics();

        let cat = result.per_category.get(&Category::Task).unwrap();
        assert!((cat.accuracy - 0.8).abs() < 0.001);
    }

    #[test]
    fn label_result_computes_precision_recall_f1() {
        let mut result = BenchResult::new();
        result.per_label.insert(
            "Task".to_string(),
            LabelResult {
                expected_count: 10,
                detected_count: 8,
                true_positives: 6,
                false_positives: 2,
                false_negatives: 4,
                ..Default::default()
            },
        );
        result.compute_metrics();

        let label = result.per_label.get("Task").unwrap();
        // Precision = 6 / (6 + 2) = 0.75
        assert!((label.precision - 0.75).abs() < 0.001);
        // Recall = 6 / (6 + 4) = 0.6
        assert!((label.recall - 0.6).abs() < 0.001);
        // F1 = 2 * 0.75 * 0.6 / (0.75 + 0.6) = 0.667
        assert!((label.f1 - 0.667).abs() < 0.01);
    }
}
