use std::str::FromStr;

use merc_error::Result;
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;

use crate::score::{
    ContextLabel, EmotionLabel, Label, OutcomeLabel, ScoreCategory, ScoreLabel, SentimentLabel,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LabelCategory {
    Sentiment,
    Emotion,
    Outcome,
    Context,
}

impl LabelCategory {
    pub fn labels(&self) -> &'static [Label] {
        match self {
            Self::Sentiment => &[
                Label::Sentiment(SentimentLabel::Positive),
                Label::Sentiment(SentimentLabel::Negative),
                Label::Sentiment(SentimentLabel::Neutral),
            ],
            Self::Emotion => &[
                Label::Emotion(EmotionLabel::Joy),
                Label::Emotion(EmotionLabel::Fear),
                Label::Emotion(EmotionLabel::Shame),
                Label::Emotion(EmotionLabel::Pride),
                Label::Emotion(EmotionLabel::Stress),
                Label::Emotion(EmotionLabel::Anger),
                Label::Emotion(EmotionLabel::Sad),
            ],
            Self::Outcome => &[
                Label::Outcome(OutcomeLabel::Success),
                Label::Outcome(OutcomeLabel::Failure),
                Label::Outcome(OutcomeLabel::Reward),
                Label::Outcome(OutcomeLabel::Punishment),
                Label::Outcome(OutcomeLabel::Decision),
                Label::Outcome(OutcomeLabel::Progress),
                Label::Outcome(OutcomeLabel::Conflict),
            ],
            Self::Context => &[
                Label::Context(ContextLabel::Fact),
                Label::Context(ContextLabel::Time),
                Label::Context(ContextLabel::Place),
                Label::Context(ContextLabel::Entity),
                Label::Context(ContextLabel::Phatic),
                Label::Context(ContextLabel::Preference),
                Label::Context(ContextLabel::Plan),
                Label::Context(ContextLabel::Goal),
                Label::Context(ContextLabel::Task),
            ],
        }
    }

    pub fn evalute(
        &self,
        text: &[&str],
        model: &ZeroShotClassificationModel,
        k: usize,
    ) -> Result<ScoreCategory> {
        let labels = model
            .predict_multilabel(
                text,
                &Label::all().map(|l| l.as_str()),
                Some(Box::new(|label: &str| {
                    Label::from_str(label)
                        .map(|l| l.hypothesis().to_string())
                        .unwrap_or_else(|_| format!("This example is {}.", label))
                })),
                128,
            )?
            .iter()
            .flat_map(|c| c.iter().map(|l| l.clone()))
            .filter_map(|l| {
                if let Ok(label) = Label::from_str(&l.text) {
                    if label.category() == *self {
                        return Some(ScoreLabel::new(label, l.sentence).with_score(l.score as f32));
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        Ok(ScoreCategory::topk(*self, labels, k))
    }
}

impl std::fmt::Display for LabelCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentiment => write!(f, "sentiment"),
            Self::Emotion => write!(f, "emotion"),
            Self::Outcome => write!(f, "outcome"),
            Self::Context => write!(f, "context"),
        }
    }
}
