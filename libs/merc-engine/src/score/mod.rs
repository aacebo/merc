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
    dynamic_threshold: bool,
    model: zero_shot_classification::ZeroShotClassificationModel,
}

/// Compute effective threshold based on text length.
/// Short text (<= 20 chars) gets a lower threshold (easier to accept).
/// Long text (> 200 chars) gets a higher threshold (harder to accept).
fn compute_threshold(text: &str, base: f32, dynamic: bool) -> f32 {
    if !dynamic {
        return base;
    }
    match text.len() {
        0..=20 => base - 0.05,
        21..=200 => base,
        _ => base + 0.05,
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
        let effective_threshold =
            compute_threshold(&ctx.text, self.threshold, self.dynamic_threshold);

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

    // === MERC-003: Dynamic Threshold Tests ===

    use super::compute_threshold;

    #[test]
    fn compute_threshold_short_text_lowers_threshold() {
        let short = "hi";
        let base = 0.75;
        let result = compute_threshold(short, base, true);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_medium_text_unchanged() {
        let medium = "This is a medium length text that has more than twenty characters.";
        let base = 0.75;
        let result = compute_threshold(medium, base, true);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "Expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_long_text_raises_threshold() {
        let long = "x".repeat(250);
        let base = 0.75;
        let result = compute_threshold(&long, base, true);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "Expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_disabled_returns_base() {
        let short = "hi";
        let medium = "This is medium text here.";
        let long = "x".repeat(250);
        let base = 0.75;

        assert!((compute_threshold(short, base, false) - base).abs() < f32::EPSILON);
        assert!((compute_threshold(medium, base, false) - base).abs() < f32::EPSILON);
        assert!((compute_threshold(&long, base, false) - base).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_threshold_boundary_20_chars() {
        let exactly_20 = "12345678901234567890";
        assert_eq!(exactly_20.len(), 20);
        let result = compute_threshold(exactly_20, 0.75, true);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "20 chars should be short, expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_boundary_21_chars() {
        let exactly_21 = "123456789012345678901";
        assert_eq!(exactly_21.len(), 21);
        let result = compute_threshold(exactly_21, 0.75, true);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "21 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_boundary_200_chars() {
        let exactly_200 = "x".repeat(200);
        let result = compute_threshold(&exactly_200, 0.75, true);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "200 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_boundary_201_chars() {
        let exactly_201 = "x".repeat(201);
        let result = compute_threshold(&exactly_201, 0.75, true);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "201 chars should be long, expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn compute_threshold_empty_text() {
        let empty = "";
        let result = compute_threshold(empty, 0.75, true);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Empty text should be short, expected 0.70, got {}",
            result
        );
    }
}
