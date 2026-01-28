mod category;
mod label;
mod options;
mod result;

pub use category::*;
pub use label::*;
pub use options::*;
pub use result::*;

use rust_bert::pipelines::zero_shot_classification;

use merc_error::{Error, ErrorCode};

use crate::{Context, Layer, LayerResult};

pub struct ScoreLayer {
    threshold: f32,
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl Layer for ScoreLayer {
    fn invoke(&self, ctx: &Context) -> merc_error::Result<LayerResult> {
        let started_at = chrono::Utc::now();
        let mut result = LayerResult::new(ScoreResult::new(vec![
            LabelCategory::Sentiment.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Emotion.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Outcome.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Context.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
        ]));

        let score = result.data::<ScoreResult>();

        if score.score < self.threshold || score.label_score(ContextLabel::Phatic.into()) >= 0.8 {
            return Err(Error::builder()
                .code(ErrorCode::Cancel)
                .message(&format!(
                    "score {} is less than minimum threshold {}",
                    result.data::<ScoreResult>().score,
                    self.threshold
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
        result.meta.set("step", ctx.step);
        result.meta.set("text", ctx.text.clone());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use merc_error::{ErrorCode, Result};

    use crate::{Context, Layer, score::ScoreOptions};

    #[test]
    fn should_cancel() -> Result<()> {
        let layer = ScoreOptions::new().build()?;
        let mut context = Context::new("hi how are you?");
        let res = layer.invoke(&mut context);

        if let Ok(v) = &res {
            println!("{:#?}", v);
        }

        assert!(res.is_err());
        assert_eq!(*res.unwrap_err().code(), ErrorCode::Cancel);
        Ok(())
    }

    #[test]
    fn should_be_stressed() -> Result<()> {
        let layer = ScoreOptions::new().build()?;
        let mut context = Context::new("oh my god, I'm going to be late for work!");
        let res = layer.invoke(&mut context)?;

        println!("{:#?}", &res);
        Ok(())
    }
}
