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

use rust_bert::pipelines::zero_shot_classification;

use merc_error::{Error, ErrorCode};

use crate::{Context, Layer, LayerResult};

pub struct ScoreLayer {
    model: zero_shot_classification::ZeroShotClassificationModel,
}

impl ScoreLayer {
    /// Compute effective threshold based on text length.
    /// Short text (<= 20 chars) gets a lower threshold (easier to accept).
    /// Long text (> 200 chars) gets a higher threshold (harder to accept).
    pub fn threshold_of(&self, text: &str) -> f32 {
        threshold!(text)
    }
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
        let effective_threshold = self.threshold_of(&ctx.text);

        if score.score < effective_threshold
            || score.label_score(ContextLabel::Phatic.into()) >= 0.8
        {
            return Err(Error::builder()
                .code(ErrorCode::Cancel)
                .message(&format!(
                    "score {} is less than minimum threshold {}",
                    result.data::<ScoreResult>().score,
                    effective_threshold
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
    #[cfg(feature = "int")]
    use merc_error::{ErrorCode, Result};

    #[cfg(feature = "int")]
    use crate::{Context, Layer, score::ScoreOptions};

    #[cfg(feature = "int")]
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

    #[cfg(feature = "int")]
    #[test]
    fn should_be_stressed() -> Result<()> {
        let layer = ScoreOptions::new().build()?;
        let mut context = Context::new("oh my god, I'm going to be late for work!");
        let res = layer.invoke(&mut context)?;

        println!("{:#?}", &res);
        Ok(())
    }

    // === MERC-003: Dynamic Threshold Tests ===

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
