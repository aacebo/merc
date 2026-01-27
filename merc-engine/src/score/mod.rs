mod category;
mod label;
mod options;
mod result;

pub use category::*;
pub use label::*;
use merc_error::{Error, ErrorCode};
pub use options::*;
pub use result::*;

use rust_bert::pipelines::zero_shot_classification;

use crate::{Context, Layer};

pub struct ScoreLayer {
    threshold: f64,
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl Layer for ScoreLayer {
    type In = Context;
    type Out = ScoreResult;

    fn invoke(&self, ctx: &mut Self::In) -> merc_error::Result<Self::Out> {
        let started_at = chrono::Utc::now();
        let labels = self.model.predict_multilabel(
            vec![ctx.text()],
            &Label::all().map(|l| l.as_str()),
            None,
            128,
        )?;

        let mut result = ScoreResult::from(labels);

        if self.threshold > result.score {
            return Err(Error::builder()
                .code(ErrorCode::Cancel)
                .message(&format!(
                    "score {} is less than minimum threshold {}",
                    result.score, self.threshold
                ))
                .build());
        }

        let elapse = chrono::Utc::now() - started_at;
        let mut elapse_message = format!("{}ms", elapse.num_milliseconds());

        if elapse.num_seconds() > 0 {
            elapse_message = format!("{}s", elapse.num_seconds());
        }

        if elapse.num_minutes() > 0 {
            elapse_message = format!("{}m", elapse.num_minutes());
        }

        if elapse.num_hours() > 0 {
            elapse_message = format!("{}h", elapse.num_hours());
        }

        result.meta.set("elapse", elapse_message);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use merc_error::Result;

    use crate::{Context, Layer, score::ScoreOptions};

    #[test]
    fn should_score() -> Result<()> {
        let layer = ScoreOptions::new().build()?;
        let mut context = Context::new("oh my god I forgot to study for exams...");
        let res = layer.invoke(&mut context)?;

        println!("{:#?}", &res);
        Ok(())
    }
}
