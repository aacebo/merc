mod category;
mod label;
mod options;
mod result;
mod threshold;

pub use category::*;
pub use label::*;
pub use options::*;
pub use result::*;

use crate::threshold;
use loom_pipe::Build;

use rust_bert::pipelines::zero_shot_classification;

use loom_error::{Error, ErrorCode};

use crate::{Context, LayerResult};

pub struct ScoreLayer {
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl ScoreLayer {
    /// Invoke the score layer directly with a context reference.
    /// This is useful for benchmarking and other cases where you need to reuse the layer.
    pub fn invoke<Input>(
        &self,
        ctx: Context<Input>,
    ) -> loom_error::Result<LayerResult<ScoreResult>> {
        let started_at = chrono::Utc::now();
        let mut result = LayerResult::new(ScoreResult::new(vec![
            LabelCategory::Sentiment.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Emotion.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Outcome.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
            LabelCategory::Context.evalute(&vec![ctx.text.as_str()], &self.model, 2)?,
        ]));

        let effective_threshold = threshold!(&ctx.text);

        if result.output.score < effective_threshold
            || result.output.label_score(ContextLabel::Phatic.into()) >= 0.8
        {
            return Err(Error::builder()
                .code(ErrorCode::Cancel)
                .message(&format!(
                    "score {} is less than minimum threshold {}",
                    result.output.score, effective_threshold
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

        result.meta.set("elapse", elapse_message.into());
        result.meta.set("step", ctx.step.into());
        result.meta.set("text", ctx.text.clone().into());
        Ok(result)
    }
}

impl<Input: 'static> loom_pipe::Operator<Context<Input>> for ScoreLayer {
    type Output = loom_error::Result<LayerResult<ScoreResult>>;

    fn apply(self, src: loom_pipe::Source<Context<Input>>) -> loom_pipe::Source<Self::Output> {
        loom_pipe::Source::new(move || self.invoke(src.build()))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "int")]
    use loom_error::{ErrorCode, Result};

    #[cfg(feature = "int")]
    use crate::{Context, score::ScoreOptions};
    #[cfg(feature = "int")]
    use loom_pipe::Source;

    #[cfg(feature = "int")]
    #[test]
    fn should_cancel() -> Result<()> {
        use loom_pipe::{Build, Pipe};

        let layer = ScoreOptions::new().build()?;
        let context = Context::new("hi how are you?", ());
        let res = Source::from(context).pipe(layer).build();

        if let Ok(v) = &res {
            println!("{:#?}", v);
        }

        assert!(res.is_err());
        assert_eq!(*res.unwrap_err().code(), ErrorCode::Cancel);
        Ok(())
    }

    #[cfg(feature = "int")]
    #[test]
    fn should_be_stressed() -> Result<()> {
        use loom_pipe::{Build, Pipe};

        let layer = ScoreOptions::new().build()?;
        let context = Context::new("oh my god, I'm going to be late for work!", ());
        let res = Source::from(context).pipe(layer).build()?;

        println!("{:#?}", &res);
        Ok(())
    }

    // === LOOM-003: Dynamic Threshold Tests ===

    use crate::threshold;

    #[test]
    fn threshold_short_text_lowers_threshold() {
        let result: f32 = threshold!("hi");
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn threshold_medium_text_unchanged() {
        let result: f32 =
            threshold!("This is a medium length text that has more than twenty characters.");
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "Expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_long_text_raises_threshold() {
        let long = "x".repeat(250);
        let result: f32 = threshold!(&long);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "Expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_20_chars() {
        let exactly_20 = "12345678901234567890";
        assert_eq!(exactly_20.len(), 20);
        let result: f32 = threshold!(exactly_20);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "20 chars should be short, expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_21_chars() {
        let exactly_21 = "123456789012345678901";
        assert_eq!(exactly_21.len(), 21);
        let result: f32 = threshold!(exactly_21);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "21 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_200_chars() {
        let exactly_200 = "x".repeat(200);
        let result: f32 = threshold!(&exactly_200);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "200 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_201_chars() {
        let exactly_201 = "x".repeat(201);
        let result: f32 = threshold!(&exactly_201);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "201 chars should be long, expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn threshold_empty_text() {
        let result: f32 = threshold!("");
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Empty text should be short, expected 0.70, got {}",
            result
        );
    }
}
