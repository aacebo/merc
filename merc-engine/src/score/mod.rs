mod context;
mod label;
mod options;
mod result;

pub use context::*;
pub use label::*;
pub use options::*;
pub use result::*;

use rust_bert::pipelines::zero_shot_classification;

use crate::{Context, Layer};

const SENTIMENT_LABELS: [&'static str; 2] = ["negative", "positive"];

const EMOTION_LABELS: [&'static str; 7] =
    ["joy", "fear", "shame", "pride", "stress", "anger", "sad"];

const OUTCOME_LABELS: [&'static str; 6] = [
    "success",
    "failure",
    "reward",
    "punishment",
    "decision",
    "response",
];

const CONTEXT_LABELS: [&'static str; 5] = ["fact", "time", "place", "person", "social"];

pub struct ScoreLayer {
    threshold: f32,
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl Layer for ScoreLayer {
    type In = ScoreContext<Context>;
    type Out = f32;

    fn invoke(&self, ctx: &mut Self::In) -> merc_error::Result<Self::Out> {
        let labels = self.model.predict_multilabel(
            ctx.text().split(".").collect::<Vec<_>>(),
            [],
            None,
            128,
        )?;

        Ok(0.0)
    }
}
