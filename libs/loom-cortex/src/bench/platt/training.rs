use std::collections::HashMap;

use tch::{Kind, Tensor};

use super::{LabelStats, PlattParams, PlattTrainingMetadata, PlattTrainingResult, RawScoreExport};

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

    let all_labels: Vec<String> = export
        .samples
        .first()
        .map(|s| s.scores.keys().cloned().collect())
        .unwrap_or_default();

    for label in &all_labels {
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

        if stats.skipped {
            params.insert(label.clone(), PlattParams::default());
            continue;
        }

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

/// Fit Platt scaling parameters (A, B) using gradient descent.
fn fit_platt_params(raw_scores: &[f32], targets: &[f32]) -> PlattParams {
    let n = raw_scores.len();
    if n == 0 {
        return PlattParams::default();
    }

    let n_pos = targets.iter().filter(|&&t| t > 0.5).count() as f64;
    let n_neg = n as f64 - n_pos;

    let t_pos = (n_pos + 1.0) / (n_pos + 2.0);
    let t_neg = 1.0 / (n_neg + 2.0);

    let corrected_targets: Vec<f64> = targets
        .iter()
        .map(|&t| if t > 0.5 { t_pos } else { t_neg })
        .collect();

    let x = Tensor::from_slice(raw_scores).to_kind(Kind::Double);
    let y = Tensor::from_slice(&corrected_targets);

    let mut a = 1.0f64;
    let mut b = 0.0f64;

    for _ in 0..NUM_ITERATIONS {
        let logits = &x * a + b;
        let p = logits.sigmoid();
        let p_clamped = p.clamp(1e-7, 1.0 - 1e-7);

        let diff = &p_clamped - &y;
        let grad_a = (&diff * &x).sum(Kind::Double).double_value(&[]) / n as f64;
        let grad_b = diff.sum(Kind::Double).double_value(&[]) / n as f64;

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
    code.push_str(
        "// Copy these values into the appropriate platt_a() and platt_b() match arms\n\n",
    );

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
        assert!((params.a - 1.0).abs() > 0.01 || params.b.abs() > 0.01);
    }
}
