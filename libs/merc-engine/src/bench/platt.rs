use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tch::{Kind, Tensor};

use super::RawScoreExport;

/// Trained Platt scaling parameters for a single label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattParams {
    pub a: f32,
    pub b: f32,
}

impl Default for PlattParams {
    fn default() -> Self {
        Self { a: 1.0, b: 0.0 }
    }
}

/// Result of training Platt parameters for all labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattTrainingResult {
    pub params: HashMap<String, PlattParams>,
    pub metadata: PlattTrainingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattTrainingMetadata {
    pub total_samples: usize,
    pub samples_per_label: HashMap<String, LabelStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStats {
    pub positive: usize,
    pub negative: usize,
    pub skipped: bool,
}

/// Minimum number of positive samples required to train parameters.
const MIN_POSITIVE_SAMPLES: usize = 5;

/// Learning rate for gradient descent.
const LEARNING_RATE: f64 = 0.1;

/// Number of optimization iterations.
const NUM_ITERATIONS: usize = 100;

/// Train Platt scaling parameters for all labels in the dataset.
pub fn train_platt_params(export: &RawScoreExport) -> PlattTrainingResult {
    let mut params = HashMap::new();
    let mut samples_per_label = HashMap::new();

    // Collect all label names from first sample
    let all_labels: Vec<String> = export
        .samples
        .first()
        .map(|s| s.scores.keys().cloned().collect())
        .unwrap_or_default();

    for label in &all_labels {
        // Collect (raw_score, is_present) pairs for this label
        let mut raw_scores = Vec::new();
        let mut targets = Vec::new();

        for sample in &export.samples {
            if let Some(&score) = sample.scores.get(label) {
                let is_present = sample.expected_labels.contains(label);
                raw_scores.push(score);
                targets.push(if is_present { 1.0f32 } else { 0.0f32 });
            }
        }

        let positive_count = targets.iter().filter(|&&t| t > 0.5).count();
        let negative_count = targets.len() - positive_count;

        let stats = LabelStats {
            positive: positive_count,
            negative: negative_count,
            skipped: positive_count < MIN_POSITIVE_SAMPLES || negative_count < MIN_POSITIVE_SAMPLES,
        };
        samples_per_label.insert(label.clone(), stats.clone());

        // Skip if insufficient data
        if stats.skipped {
            params.insert(label.clone(), PlattParams::default());
            continue;
        }

        // Fit parameters using gradient descent
        let fitted = fit_platt_params(&raw_scores, &targets);
        params.insert(label.clone(), fitted);
    }

    PlattTrainingResult {
        params,
        metadata: PlattTrainingMetadata {
            total_samples: export.samples.len(),
            samples_per_label,
        },
    }
}

/// Fit Platt scaling parameters (A, B) using gradient descent on binary cross-entropy loss.
/// P(y|x) = 1 / (1 + exp(-A*x - B))
fn fit_platt_params(raw_scores: &[f32], targets: &[f32]) -> PlattParams {
    let n = raw_scores.len();
    if n == 0 {
        return PlattParams::default();
    }

    // Apply Platt's target correction to avoid overfitting
    let n_pos = targets.iter().filter(|&&t| t > 0.5).count() as f64;
    let n_neg = n as f64 - n_pos;

    let t_pos = (n_pos + 1.0) / (n_pos + 2.0);
    let t_neg = 1.0 / (n_neg + 2.0);

    let corrected_targets: Vec<f64> = targets
        .iter()
        .map(|&t| if t > 0.5 { t_pos } else { t_neg })
        .collect();

    // Convert to tensors
    let x = Tensor::from_slice(raw_scores).to_kind(Kind::Double);
    let y = Tensor::from_slice(&corrected_targets);

    // Initialize parameters
    let mut a = 1.0f64;
    let mut b = 0.0f64;

    // Gradient descent
    for _ in 0..NUM_ITERATIONS {
        // Compute sigmoid: p = 1 / (1 + exp(-a*x - b))
        let logits = &x * a + b;
        let p = logits.sigmoid();

        // Clamp for numerical stability
        let p_clamped = p.clamp(1e-7, 1.0 - 1e-7);

        // Compute gradients of negative log-likelihood
        // NLL = -sum(y * log(p) + (1-y) * log(1-p))
        // d(NLL)/da = sum((p - y) * x)
        // d(NLL)/db = sum(p - y)
        let diff = &p_clamped - &y;
        let grad_a = (&diff * &x).sum(Kind::Double).double_value(&[]) / n as f64;
        let grad_b = diff.sum(Kind::Double).double_value(&[]) / n as f64;

        // Update parameters
        a -= LEARNING_RATE * grad_a;
        b -= LEARNING_RATE * grad_b;
    }

    PlattParams {
        a: a as f32,
        b: b as f32,
    }
}

/// Generate Rust code for updating label.rs with trained parameters.
pub fn generate_rust_code(result: &PlattTrainingResult) -> String {
    let mut code = String::new();
    code.push_str("// Generated Platt calibration parameters\n");
    code.push_str(&format!(
        "// Trained on {} samples\n\n",
        result.metadata.total_samples
    ));

    // Group by label category (assuming naming convention like "positive", "negative", etc.)
    code.push_str("// Copy these values into the appropriate platt_a() and platt_b() match arms in label.rs\n\n");

    let mut sorted_labels: Vec<_> = result.params.iter().collect();
    sorted_labels.sort_by_key(|(k, _)| k.as_str());

    for (label, params) in sorted_labels {
        let stats = result.metadata.samples_per_label.get(label);
        let comment = if let Some(s) = stats {
            if s.skipped {
                " // SKIPPED: insufficient data"
            } else {
                ""
            }
        } else {
            ""
        };
        code.push_str(&format!(
            "// {}: a={:.4}, b={:.4}{}\n",
            label, params.a, params.b, comment
        ));
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platt_params_default_is_identity() {
        let params = PlattParams::default();
        assert!((params.a - 1.0).abs() < f32::EPSILON);
        assert!(params.b.abs() < f32::EPSILON);
    }

    #[test]
    fn fit_platt_params_with_empty_data_returns_identity() {
        let params = fit_platt_params(&[], &[]);
        assert!((params.a - 1.0).abs() < f32::EPSILON);
        assert!(params.b.abs() < f32::EPSILON);
    }

    #[test]
    fn fit_platt_params_with_balanced_data() {
        // Create synthetic data: scores near 0.8 for positives, near 0.2 for negatives
        let raw_scores: Vec<f32> = (0..20)
            .map(|i| {
                if i < 10 {
                    0.8 + (i as f32) * 0.01
                } else {
                    0.2 + ((i - 10) as f32) * 0.01
                }
            })
            .collect();
        let targets: Vec<f32> = (0..20).map(|i| if i < 10 { 1.0 } else { 0.0 }).collect();

        let params = fit_platt_params(&raw_scores, &targets);

        // Parameters should be non-identity
        assert!((params.a - 1.0).abs() > 0.01 || params.b.abs() > 0.01);
    }
}
