use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    CategoryMetrics, CategoryResult, EvalMetrics, LabelMetrics, LabelResult, SampleResult,
};

/// Raw benchmark results (counts only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub total: usize,
    pub correct: usize,
    pub per_category: HashMap<String, CategoryResult>,
    pub per_label: HashMap<String, LabelResult>,
    pub sample_results: Vec<SampleResult>,
    /// Total evaluation time in milliseconds.
    #[serde(default)]
    pub elapsed_ms: i64,
    /// Throughput in samples per second.
    #[serde(default)]
    pub throughput: f32,
}

impl EvalResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self {
            total: 0,
            correct: 0,
            per_category: HashMap::new(),
            per_label: HashMap::new(),
            sample_results: Vec::new(),
            elapsed_ms: 0,
            throughput: 0.0,
        }
    }

    /// Compute metrics from the collected counts.
    pub fn metrics(&self) -> EvalMetrics {
        let mut metrics = EvalMetrics::default();

        // Overall accuracy
        if self.total > 0 {
            metrics.accuracy = self.correct as f32 / self.total as f32;
        }

        // Per-category accuracy
        for (category, result) in &self.per_category {
            let mut cat_metrics = CategoryMetrics::default();
            if result.total > 0 {
                cat_metrics.accuracy = result.correct as f32 / result.total as f32;
            }
            metrics.per_category.insert(category.clone(), cat_metrics);
        }

        // Per-label precision/recall/F1
        let mut total_precision = 0.0;
        let mut total_recall = 0.0;
        let mut label_count = 0;

        for (label, result) in &self.per_label {
            let mut label_metrics = LabelMetrics::default();

            // Precision = TP / (TP + FP)
            let tp_fp = result.true_positives + result.false_positives;
            if tp_fp > 0 {
                label_metrics.precision = result.true_positives as f32 / tp_fp as f32;
            }

            // Recall = TP / (TP + FN)
            let tp_fn = result.true_positives + result.false_negatives;
            if tp_fn > 0 {
                label_metrics.recall = result.true_positives as f32 / tp_fn as f32;
            }

            // F1 = 2 * (precision * recall) / (precision + recall)
            let pr_sum = label_metrics.precision + label_metrics.recall;
            if pr_sum > 0.0 {
                label_metrics.f1 = 2.0 * label_metrics.precision * label_metrics.recall / pr_sum;
            }

            if result.expected_count > 0 {
                total_precision += label_metrics.precision;
                total_recall += label_metrics.recall;
                label_count += 1;
            }

            metrics.per_label.insert(label.clone(), label_metrics);
        }

        // Macro-averaged precision/recall/F1
        if label_count > 0 {
            metrics.precision = total_precision / label_count as f32;
            metrics.recall = total_recall / label_count as f32;
            let pr_sum = metrics.precision + metrics.recall;
            if pr_sum > 0.0 {
                metrics.f1 = 2.0 * metrics.precision * metrics.recall / pr_sum;
            }
        }

        metrics
    }
}

impl Default for EvalResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_result_computes_accuracy() {
        let mut result = EvalResult::new();
        result.total = 10;
        result.correct = 8;
        let metrics = result.metrics();
        assert!((metrics.accuracy - 0.8).abs() < 0.001);
    }

    #[test]
    fn category_result_computes_accuracy() {
        let mut result = EvalResult::new();
        result.per_category.insert(
            "task".to_string(),
            CategoryResult {
                total: 5,
                correct: 4,
            },
        );
        let metrics = result.metrics();

        let cat = metrics.per_category.get("task").unwrap();
        assert!((cat.accuracy - 0.8).abs() < 0.001);
    }

    #[test]
    fn label_result_computes_precision_recall_f1() {
        let mut result = EvalResult::new();
        result.per_label.insert(
            "Task".to_string(),
            LabelResult {
                expected_count: 10,
                detected_count: 8,
                true_positives: 6,
                false_positives: 2,
                false_negatives: 4,
            },
        );
        let metrics = result.metrics();

        let label = metrics.per_label.get("Task").unwrap();
        assert!((label.precision - 0.75).abs() < 0.001);
        assert!((label.recall - 0.6).abs() < 0.001);
        assert!((label.f1 - 0.667).abs() < 0.01);
    }
}
