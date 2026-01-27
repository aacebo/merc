use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use rust_bert::pipelines::sequence_classification;

use crate::{
    Meta,
    score::{Label, LabelCategory},
};

#[derive(Debug, Default, Clone)]
pub struct ScoreResult {
    pub meta: Meta,
    pub score: f64,
    pub categories: Vec<ScoreCategory>,
}

impl ScoreResult {
    pub fn new(categories: Vec<ScoreCategory>) -> Self {
        let mut value = Self {
            meta: Meta::default(),
            score: 0.0,
            categories,
        };

        for category in &value.categories {
            if value.score < category.score {
                value.score = category.score;
            }
        }

        value
            .categories
            .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        value
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
}

impl From<Vec<Vec<sequence_classification::Label>>> for ScoreResult {
    fn from(lines: Vec<Vec<sequence_classification::Label>>) -> Self {
        let mut categories: HashMap<LabelCategory, Vec<ScoreLabel>> = HashMap::new();

        for line in &lines {
            for class in line {
                if let Ok(label) = Label::from_str(&class.text) {
                    let labels = categories.entry(label.category()).or_insert(vec![]);
                    labels.push(ScoreLabel::new(label, class.sentence).with_score(class.score));
                }
            }
        }

        let mut arr = vec![];

        for (label, labels) in categories {
            arr.push(ScoreCategory::new(label, labels));
        }

        Self::new(arr)
    }
}

#[derive(Debug, Clone)]
pub struct ScoreCategory {
    pub label: LabelCategory,
    pub score: f64,
    pub labels: Vec<ScoreLabel>,
}

impl ScoreCategory {
    pub fn new(label: LabelCategory, labels: Vec<ScoreLabel>) -> Self {
        let mut value = Self {
            label,
            score: 0.0,
            labels,
        };

        for label in &value.labels {
            if value.score < label.score {
                value.score = label.score;
            }
        }

        value
            .labels
            .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        value
    }
}

#[derive(Debug, Clone)]
pub struct ScoreLabel {
    pub label: Label,
    pub score: f64,
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

    pub fn with_score(mut self, value: f64) -> Self {
        self.score = value;
        self
    }
}
