use std::collections::HashMap;

use super::{CategoryResult, LabelResult, SampleResult};
use crate::bench::Category;

/// Overall benchmark results.
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

impl BenchResult {
    /// Create a new empty result.
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

    /// Compute metrics from the collected data.
    pub fn compute_metrics(&mut self) {
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
        assert!((label.precision - 0.75).abs() < 0.001);
        assert!((label.recall - 0.6).abs() < 0.001);
        assert!((label.f1 - 0.667).abs() < 0.01);
    }
}
