use std::collections::BTreeMap;

use loom_core::value::Value;
use serde::{Deserialize, Serialize};

use super::ScoreLabelConfig;

/// Apply Platt scaling to calibrate raw model scores.
/// P(y|x) = 1 / (1 + exp(-Ax - B))
/// With identity params (a=1.0, b=0.0), returns raw score unchanged.
#[inline]
fn calibrate(raw: f32, a: f32, b: f32) -> f32 {
    // Identity: skip calibration
    if (a - 1.0).abs() < f32::EPSILON && b.abs() < f32::EPSILON {
        return raw;
    }
    1.0 / (1.0 + (-a * raw - b).exp())
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    /// Overall score (max of category scores)
    pub score: f32,
    /// Categories keyed by name (mirrors config structure)
    pub categories: BTreeMap<String, ScoreCategory>,
}

impl ScoreResult {
    pub fn new(categories: BTreeMap<String, ScoreCategory>) -> Self {
        let score = categories.values().map(|c| c.score).fold(0.0f32, f32::max);
        Self { score, categories }
    }

    pub fn category(&self, name: &str) -> Option<&ScoreCategory> {
        self.categories.get(name)
    }

    pub fn category_mut(&mut self, name: &str) -> Option<&mut ScoreCategory> {
        self.categories.get_mut(name)
    }

    pub fn label(&self, name: &str) -> Option<&ScoreLabel> {
        self.categories
            .values()
            .flat_map(|c| c.labels.get(name))
            .next()
    }

    pub fn label_score(&self, name: &str) -> f32 {
        self.label(name).map(|l| l.score).unwrap_or_default()
    }

    /// Returns (label_name, raw_score) pairs for external use
    pub fn raw_scores(&self) -> Vec<(String, f32)> {
        self.categories
            .iter()
            .flat_map(|(_, cat)| {
                cat.labels
                    .iter()
                    .map(|(name, label)| (name.clone(), label.raw_score))
            })
            .collect()
    }
}

#[cfg(feature = "json")]
impl From<ScoreResult> for Value {
    fn from(result: ScoreResult) -> Self {
        let json = serde_json::to_value(&result).expect("ScoreResult is serializable");
        Value::from(json)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreCategory {
    /// Category score (avg of top-k labels)
    pub score: f32,
    /// Labels keyed by name (mirrors config structure)
    pub labels: BTreeMap<String, ScoreLabel>,
}

impl ScoreCategory {
    pub fn new(labels: BTreeMap<String, ScoreLabel>) -> Self {
        let score = if labels.is_empty() {
            0.0
        } else {
            labels.values().map(|l| l.score).sum::<f32>() / labels.len() as f32
        };
        Self { score, labels }
    }

    pub fn topk(labels: BTreeMap<String, ScoreLabel>, k: usize) -> Self {
        let take = k.min(labels.len()).max(1);

        // Sort by score for top-k calculation
        let mut sorted: Vec<_> = labels.values().collect();
        sorted.sort_by(|a, b| b.score.total_cmp(&a.score));

        let score = if sorted.is_empty() {
            0.0
        } else {
            sorted.iter().take(take).map(|l| l.score).sum::<f32>() / take as f32
        };

        Self { score, labels }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreLabel {
    /// Calibrated score (raw * weight, if above threshold)
    pub score: f32,
    /// Raw model output before calibration
    pub raw_score: f32,
    /// Sentence index (for multi-sentence inputs)
    pub sentence: usize,
}

impl ScoreLabel {
    pub fn new(raw_score: f32, sentence: usize, config: &ScoreLabelConfig) -> Self {
        let calibrated = calibrate(raw_score, config.platt_a, config.platt_b);
        let score = if calibrated >= config.threshold {
            calibrated * config.weight
        } else {
            0.0
        };
        Self {
            score,
            raw_score,
            sentence,
        }
    }

    pub fn ignore(&self, threshold: f32) -> bool {
        self.score < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Platt Calibration Tests ===

    #[test]
    fn calibrate_identity_params_returns_raw() {
        let raw_scores = [0.0, 0.25, 0.5, 0.75, 1.0];
        for raw in raw_scores {
            let result = calibrate(raw, 1.0, 0.0);
            assert!(
                (result - raw).abs() < f32::EPSILON,
                "Identity calibration failed: expected {}, got {}",
                raw,
                result
            );
        }
    }

    #[test]
    fn calibrate_identity_near_epsilon() {
        let raw = 0.75;
        let near_one = 1.0 + f32::EPSILON * 0.5;
        let near_zero = f32::EPSILON * 0.5;
        let result = calibrate(raw, near_one, near_zero);
        assert!(
            (result - raw).abs() < 0.001,
            "Near-identity should return raw: expected {}, got {}",
            raw,
            result
        );
    }

    #[test]
    fn calibrate_sigmoid_midpoint() {
        // When A*x + B = 0, sigmoid = 0.5
        // With a=2.0, b=-1.0: at x=0.5, we get 2*0.5 - 1 = 0
        let result = calibrate(0.5, 2.0, -1.0);
        assert!(
            (result - 0.5).abs() < 0.001,
            "Sigmoid midpoint should be 0.5, got {}",
            result
        );
    }

    #[test]
    fn calibrate_high_raw_produces_high_output() {
        let result = calibrate(0.95, 2.0, 0.0);
        assert!(
            result > 0.8,
            "High raw should produce high output, got {}",
            result
        );
    }

    #[test]
    fn calibrate_low_raw_produces_low_output() {
        let result = calibrate(0.1, 2.0, 0.0);
        assert!(
            result < 0.6,
            "Low raw should produce lower output, got {}",
            result
        );
    }

    #[test]
    fn calibrate_negative_b_shifts_down() {
        let raw = 0.7;
        let with_zero_b = calibrate(raw, 1.5, 0.0);
        let with_neg_b = calibrate(raw, 1.5, -0.5);
        assert!(
            with_neg_b < with_zero_b,
            "Negative B should shift down: {} should be < {}",
            with_neg_b,
            with_zero_b
        );
    }

    #[test]
    fn calibrate_positive_b_shifts_up() {
        let raw = 0.3;
        let with_zero_b = calibrate(raw, 1.5, 0.0);
        let with_pos_b = calibrate(raw, 1.5, 0.5);
        assert!(
            with_pos_b > with_zero_b,
            "Positive B should shift up: {} should be > {}",
            with_pos_b,
            with_zero_b
        );
    }

    #[test]
    fn calibrate_output_bounded_0_to_1() {
        let extreme_cases = [
            (0.0, 5.0, -10.0),
            (1.0, 5.0, 10.0),
            (0.5, 0.1, 0.0),
            (0.5, 10.0, 0.0),
        ];
        for (raw, a, b) in extreme_cases {
            let result = calibrate(raw, a, b);
            assert!(
                result >= 0.0 && result <= 1.0,
                "Calibrated score must be in [0,1], got {} for ({}, {}, {})",
                result,
                raw,
                a,
                b
            );
        }
    }

    #[test]
    fn calibrate_formula_verification() {
        let raw: f32 = 0.6;
        let a: f32 = 1.5;
        let b: f32 = -0.3;
        let expected: f32 = 1.0 / (1.0 + (-a * raw - b).exp());
        let result = calibrate(raw, a, b);
        assert!(
            (result - expected).abs() < f32::EPSILON,
            "Formula mismatch: expected {}, got {}",
            expected,
            result
        );
    }

    // === ScoreLabel Tests ===

    #[test]
    fn score_label_applies_calibration() {
        let config = ScoreLabelConfig {
            hypothesis: "test".to_string(),
            weight: 0.30,
            threshold: 0.70,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label = ScoreLabel::new(0.8, 0, &config);
        // With identity calibration (a=1.0, b=0.0), raw score passes through
        // Score = calibrated * weight = 0.8 * 0.30 = 0.24 (if above threshold 0.70)
        let expected = 0.8 * config.weight;
        assert!(
            (score_label.score - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            score_label.score
        );
    }

    #[test]
    fn score_label_below_threshold_zeroes_score() {
        let config = ScoreLabelConfig {
            hypothesis: "test".to_string(),
            weight: 0.30,
            threshold: 0.70,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label = ScoreLabel::new(0.5, 0, &config);
        assert!(
            (score_label.score - 0.0).abs() < f32::EPSILON,
            "Score below threshold should be 0, got {}",
            score_label.score
        );
    }

    #[test]
    fn score_label_at_threshold_passes() {
        let config = ScoreLabelConfig {
            hypothesis: "test".to_string(),
            weight: 1.00,
            threshold: 0.65,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label = ScoreLabel::new(0.65, 0, &config);
        let expected = 0.65 * config.weight;
        assert!(
            (score_label.score - expected).abs() < 0.001,
            "Score at threshold should pass: expected {}, got {}",
            expected,
            score_label.score
        );
    }

    // === ScoreCategory Tests ===

    #[test]
    fn score_category_topk() {
        let config = ScoreLabelConfig {
            hypothesis: "test".to_string(),
            weight: 1.0,
            threshold: 0.0,
            platt_a: 1.0,
            platt_b: 0.0,
        };

        let mut labels = BTreeMap::new();
        labels.insert("a".to_string(), ScoreLabel::new(0.9, 0, &config));
        labels.insert("b".to_string(), ScoreLabel::new(0.7, 0, &config));
        labels.insert("c".to_string(), ScoreLabel::new(0.5, 0, &config));

        let category = ScoreCategory::topk(labels, 2);
        // Top 2 are 0.9 and 0.7, avg = 0.8
        assert!(
            (category.score - 0.8).abs() < 0.001,
            "Expected 0.8, got {}",
            category.score
        );
    }

    // === ScoreResult Tests ===

    #[test]
    fn score_result_category_lookup() {
        let mut categories = BTreeMap::new();
        categories.insert(
            "sentiment".to_string(),
            ScoreCategory {
                score: 0.5,
                labels: BTreeMap::new(),
            },
        );

        let result = ScoreResult::new(categories);
        assert!(result.category("sentiment").is_some());
        assert!(result.category("nonexistent").is_none());
    }

    #[test]
    fn score_result_label_lookup() {
        let config = ScoreLabelConfig {
            hypothesis: "test".to_string(),
            weight: 1.0,
            threshold: 0.0,
            platt_a: 1.0,
            platt_b: 0.0,
        };

        let mut labels = BTreeMap::new();
        labels.insert("positive".to_string(), ScoreLabel::new(0.8, 0, &config));

        let mut categories = BTreeMap::new();
        categories.insert("sentiment".to_string(), ScoreCategory::new(labels));

        let result = ScoreResult::new(categories);
        assert!(result.label("positive").is_some());
        assert_eq!(result.label_score("positive"), 0.8);
        assert_eq!(result.label_score("nonexistent"), 0.0);
    }
}
