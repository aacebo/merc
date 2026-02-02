use std::{collections::HashMap, str::FromStr};

use rust_bert::pipelines::sequence_classification;

use crate::score::{Label, LabelCategory};

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

    pub fn category(&self, label: LabelCategory) -> &ScoreCategory {
        self.categories.iter().find(|v| v.label == label).unwrap()
    }

    pub fn category_mut(&mut self, label: LabelCategory) -> &mut ScoreCategory {
        self.categories
            .iter_mut()
            .find(|v| v.label == label)
            .unwrap()
    }

    pub fn labels(&self) -> Vec<&ScoreLabel> {
        self.categories.iter().flat_map(|v| &v.labels).collect()
    }

    pub fn label(&self, label: Label) -> &ScoreLabel {
        self.labels().iter().find(|l| l.label == label).unwrap()
    }

    pub fn label_score(&self, label: Label) -> f32 {
        self.labels()
            .iter()
            .find(|l| l.label == label)
            .map(|l| l.score)
            .unwrap_or_default()
    }
}

impl From<Vec<Vec<sequence_classification::Label>>> for ScoreResult {
    fn from(lines: Vec<Vec<sequence_classification::Label>>) -> Self {
        let mut categories: HashMap<LabelCategory, Vec<ScoreLabel>> = HashMap::new();

        for line in &lines {
            for class in line {
                if let Ok(label) = Label::from_str(&class.text) {
                    let labels = categories.entry(label.category()).or_insert(vec![]);
                    labels.push(
                        ScoreLabel::new(label, class.sentence).with_score(class.score as f32),
                    );
                }
            }
        }

        let mut arr = vec![];

        for (label, labels) in categories {
            arr.push(ScoreCategory::topk(label, labels, 2));
        }

        Self::new(arr)
    }
}

#[derive(Debug, Clone)]
pub struct ScoreCategory {
    pub label: LabelCategory,
    pub score: f32,
    pub labels: Vec<ScoreLabel>,
}

impl ScoreCategory {
    pub fn topk(label: LabelCategory, labels: Vec<ScoreLabel>, k: usize) -> Self {
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
            label,
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
    pub label: Label,
    pub score: f32,
    pub sentence: usize,
}

impl ScoreLabel {
    pub fn new(label: Label, sentence: usize) -> Self {
        Self {
            label,
            score: 0.0,
            sentence,
        }
    }

    pub fn with_score(mut self, raw_score: f32) -> Self {
        let calibrated = calibrate(raw_score, self.label.platt_a(), self.label.platt_b());
        if calibrated >= self.label.threshold() {
            self.score = calibrated * self.label.weight();
        }

        self
    }

    pub fn ignore(&self) -> bool {
        self.score < self.label.threshold()
    }
}
