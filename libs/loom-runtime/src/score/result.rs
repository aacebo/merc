use crate::score::ScoreLabelConfig;

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

#[derive(Debug, Default, Clone)]
pub struct ScoreResult {
    pub score: f32,
    pub categories: Vec<ScoreCategory>,
}

impl ScoreResult {
    pub fn new(groups: Vec<ScoreCategory>) -> Self {
        let mut categories = groups.clone();
        categories.sort_by(|a, b| b.score.total_cmp(&a.score));

        Self {
            score: categories
                .iter()
                .map(|value| value.score)
                .fold(0.0, f32::max),
            categories,
        }
    }

    pub fn category(&self, name: &str) -> Option<&ScoreCategory> {
        self.categories.iter().find(|v| v.name == name)
    }

    pub fn category_mut(&mut self, name: &str) -> Option<&mut ScoreCategory> {
        self.categories.iter_mut().find(|v| v.name == name)
    }

    pub fn labels(&self) -> Vec<&ScoreLabel> {
        self.categories.iter().flat_map(|v| &v.labels).collect()
    }

    pub fn label(&self, name: &str) -> Option<&ScoreLabel> {
        self.labels().into_iter().find(|l| l.name == name)
    }

    pub fn label_score(&self, name: &str) -> f32 {
        self.label(name).map(|l| l.score).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ScoreCategory {
    pub name: String,
    pub score: f32,
    pub labels: Vec<ScoreLabel>,
}

impl ScoreCategory {
    pub fn topk(name: String, labels: Vec<ScoreLabel>, k: usize) -> Self {
        let take = k.min(labels.len()).max(1);
        let mut list = labels.clone();
        let mut score = 0.0f32;

        list.sort_by(|a, b| b.score.total_cmp(&a.score));
        let top = list
            .iter()
            .take(take)
            .map(|v| v.clone())
            .collect::<Vec<_>>();

        for label in &top {
            score += label.score;
        }

        Self {
            name,
            score: if top.is_empty() {
                0.0
            } else {
                score / take as f32
            },
            labels: list,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreLabel {
    pub name: String,
    pub category: String,
    pub score: f32,
    pub raw_score: f32,
    pub sentence: usize,
}

impl ScoreLabel {
    pub fn new(name: String, category: String, sentence: usize) -> Self {
        Self {
            name,
            category,
            score: 0.0,
            raw_score: 0.0,
            sentence,
        }
    }

    pub fn with_score(mut self, raw_score: f32, config: &ScoreLabelConfig) -> Self {
        self.raw_score = raw_score;
        let calibrated = calibrate(raw_score, config.platt_a, config.platt_b);
        if calibrated >= config.threshold {
            self.score = calibrated * config.weight;
        }
        self
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
            name: "positive".to_string(),
            hypothesis: "test".to_string(),
            weight: 0.30,
            threshold: 0.70,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label = ScoreLabel::new("positive".to_string(), "sentiment".to_string(), 0)
            .with_score(0.8, &config);
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
            name: "positive".to_string(),
            hypothesis: "test".to_string(),
            weight: 0.30,
            threshold: 0.70,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label = ScoreLabel::new("positive".to_string(), "sentiment".to_string(), 0)
            .with_score(0.5, &config);
        assert!(
            (score_label.score - 0.0).abs() < f32::EPSILON,
            "Score below threshold should be 0, got {}",
            score_label.score
        );
    }

    #[test]
    fn score_label_at_threshold_passes() {
        let config = ScoreLabelConfig {
            name: "task".to_string(),
            hypothesis: "test".to_string(),
            weight: 1.00,
            threshold: 0.65,
            platt_a: 1.0,
            platt_b: 0.0,
        };
        let score_label =
            ScoreLabel::new("task".to_string(), "context".to_string(), 0).with_score(0.65, &config);
        let expected = 0.65 * config.weight;
        assert!(
            (score_label.score - expected).abs() < 0.001,
            "Score at threshold should pass: expected {}, got {}",
            expected,
            score_label.score
        );
    }
}
