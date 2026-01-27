mod category;
mod label;
mod options;
mod result;

pub use category::*;
pub use label::*;
use merc_error::{Error, ErrorCode};
pub use options::*;
pub use result::*;

use std::str::FromStr;

use rust_bert::pipelines::zero_shot_classification;

use crate::{Context, Layer, LayerResult};

pub struct ScoreLayer {
    threshold: f64,
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl Layer for ScoreLayer {
    fn invoke(&self, ctx: &Context) -> merc_error::Result<LayerResult> {
        let started_at = chrono::Utc::now();
        let labels = self.model.predict_multilabel(
            vec![ctx.text.as_str()],
            &Label::all().map(|l| l.as_str()),
            Some(Box::new(|label: &str| {
                Label::from_str(label)
                    .map(|l| l.hypothesis().to_string())
                    .unwrap_or_else(|_| format!("This example is {}.", label))
            })),
            128,
        )?;

        let mut result = LayerResult::new(ScoreResult::from(labels));

        if self.threshold > result.data::<ScoreResult>().score {
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

        assert!(res.is_err());
        assert_eq!(*res.unwrap_err().code(), ErrorCode::Cancel);
        Ok(())
    }
}
