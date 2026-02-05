mod config;
mod result;

pub use config::*;
pub use result::*;

use std::collections::HashMap;

use loom_cortex::CortexModel;
use loom_cortex::bench::{Decision, Scorer, ScorerOutput};
use loom_error::{Error, ErrorCode};
use loom_pipe::Build;

use crate::{Context, LayerResult};

pub struct ScoreLayer {
    model: CortexModel,
    config: ScoreConfig,
}

impl ScoreLayer {
    pub(crate) fn new(model: CortexModel, config: ScoreConfig) -> Self {
        Self { model, config }
    }

    /// Get the configuration for this layer
    pub fn config(&self) -> &ScoreConfig {
        &self.config
    }

    /// Invoke the score layer directly with a context reference.
    /// This is useful for benchmarking and other cases where you need to reuse the layer.
    pub fn invoke<Input>(
        &self,
        ctx: Context<Input>,
    ) -> loom_error::Result<LayerResult<ScoreResult>> {
        let started_at = chrono::Utc::now();

        // Extract the zero-shot model
        let zs_model = match &self.model {
            CortexModel::ZeroShotClassification { model, .. } => model,
            _ => {
                return Err(Error::builder()
                    .code(ErrorCode::BadArguments)
                    .message("ScoreLayer requires a ZeroShotClassification model")
                    .build());
            }
        };

        // Get all label names from config
        let label_names: Vec<&str> = self
            .config
            .categories
            .iter()
            .flat_map(|c| c.labels.iter().map(|l| l.name.as_str()))
            .collect();

        // Build a static hypothesis map for the closure
        let hypothesis_map: std::collections::HashMap<String, String> = self
            .config
            .categories
            .iter()
            .flat_map(|c| {
                c.labels
                    .iter()
                    .map(|l| (l.name.clone(), l.hypothesis.clone()))
            })
            .collect();

        // Create hypothesis function using the cloned map
        let hypothesis_fn = Box::new(move |label: &str| {
            hypothesis_map
                .get(label)
                .cloned()
                .unwrap_or_else(|| format!("This example is {}.", label))
        });

        // Run zero-shot classification
        let predictions = zs_model.predict_multilabel(
            &[ctx.text.as_str()],
            &label_names,
            Some(hypothesis_fn),
            128,
        )?;

        // Build a lookup map for predictions by label name
        let mut prediction_map: HashMap<&str, f32> = HashMap::new();
        for sentence_predictions in &predictions {
            for pred in sentence_predictions {
                prediction_map.insert(
                    label_names
                        .iter()
                        .find(|&&n| n == pred.text)
                        .copied()
                        .unwrap_or(&pred.text),
                    pred.score as f32,
                );
            }
        }

        // Build ScoreCategory for each category in config
        let mut categories = Vec::new();

        for cat_config in &self.config.categories {
            let mut labels = Vec::new();

            for label_config in &cat_config.labels {
                let raw_score = prediction_map
                    .get(label_config.name.as_str())
                    .copied()
                    .unwrap_or(0.0);

                let score_label = ScoreLabel::new(
                    label_config.name.clone(),
                    cat_config.name.clone(),
                    0, // sentence index
                )
                .with_score(raw_score, label_config);

                labels.push(score_label);
            }

            let top_k = cat_config.top_k;
            categories.push(ScoreCategory::topk(cat_config.name.clone(), labels, top_k));
        }

        let mut result = LayerResult::new(ScoreResult::new(categories));
        let effective_threshold = self.config.threshold_of(ctx.text.len());
        let phatic_score = result.output.label_score("phatic");
        let phatic_threshold = self
            .config
            .label("phatic")
            .map(|l| l.threshold)
            .unwrap_or(0.80);

        if result.output.score < effective_threshold || phatic_score >= phatic_threshold {
            return Err(Error::builder()
                .code(ErrorCode::Cancel)
                .message(&format!(
                    "score {} is less than minimum threshold {}",
                    result.output.score, effective_threshold
                ))
                .build());
        }

        // Add timing metadata
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

/// Wrapper around ScoreResult that implements ScorerOutput.
pub struct ScoreLayerOutput(ScoreResult);

impl ScoreLayerOutput {
    pub fn new(result: ScoreResult) -> Self {
        Self(result)
    }

    /// Get the underlying ScoreResult.
    pub fn inner(&self) -> &ScoreResult {
        &self.0
    }
}

impl ScorerOutput for ScoreLayerOutput {
    fn decision(&self) -> Decision {
        // If we got a successful result, it's Accept
        // (Reject happens when invoke returns an error)
        Decision::Accept
    }

    fn score(&self) -> f32 {
        self.0.score
    }

    fn labels(&self) -> Vec<(String, f32)> {
        self.0
            .labels()
            .into_iter()
            .map(|l: &ScoreLabel| (l.name.clone(), l.raw_score))
            .collect()
    }
}

impl Scorer for ScoreLayer {
    type Output = ScoreLayerOutput;
    type Error = Error;

    fn score(&self, text: &str) -> Result<Self::Output, Self::Error> {
        let ctx = Context::new(text, ());
        self.invoke(ctx).map(|r| ScoreLayerOutput::new(r.output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ScoreConfig Threshold Tests ===

    #[test]
    fn threshold_short_text_lowers_threshold() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(10);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn threshold_medium_text_unchanged() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(100);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "Expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_long_text_raises_threshold() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(250);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "Expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_20_chars() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(20);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "20 chars should be short, expected 0.70, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_21_chars() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(21);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "21 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_200_chars() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(200);
        assert!(
            (result - 0.75).abs() < f32::EPSILON,
            "200 chars should be medium, expected 0.75, got {}",
            result
        );
    }

    #[test]
    fn threshold_boundary_201_chars() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(201);
        assert!(
            (result - 0.80).abs() < f32::EPSILON,
            "201 chars should be long, expected 0.80, got {}",
            result
        );
    }

    #[test]
    fn threshold_empty_text() {
        let config = ScoreConfig::default();
        let result = config.threshold_of(0);
        assert!(
            (result - 0.70).abs() < f32::EPSILON,
            "Empty text should be short, expected 0.70, got {}",
            result
        );
    }

    // === Integration Tests (require model) ===

    #[cfg(feature = "int")]
    use crate::Context;
    #[cfg(feature = "int")]
    use loom_error::{ErrorCode, Result};
    #[cfg(feature = "int")]
    use loom_pipe::Source;

    #[cfg(feature = "int")]
    fn int_test_config() -> ScoreConfig {
        use loom_cortex::config::{CortexModelConfig, CortexZeroShotConfig};

        ScoreConfig {
            model: CortexModelConfig::ZeroShotClassification(CortexZeroShotConfig::default()),
            threshold: 0.40,
            top_k: 2,
            modifiers: ScoreModifierConfig::default(),
            categories: vec![
                ScoreCategoryConfig {
                    name: "sentiment".to_string(),
                    top_k: 2,
                    labels: vec![
                        ScoreLabelConfig {
                            name: "positive".to_string(),
                            hypothesis: "The speaker is expressing a positive, happy, or optimistic sentiment.".to_string(),
                            weight: 0.30,
                            threshold: 0.70,
                            platt_a: 1.0,
                            platt_b: 0.0,
                        },
                        ScoreLabelConfig {
                            name: "negative".to_string(),
                            hypothesis: "The speaker is expressing a negative, unhappy, or pessimistic sentiment.".to_string(),
                            weight: 0.35,
                            threshold: 0.70,
                            platt_a: 1.0,
                            platt_b: 0.0,
                        },
                    ],
                },
                ScoreCategoryConfig {
                    name: "emotion".to_string(),
                    top_k: 2,
                    labels: vec![
                        ScoreLabelConfig {
                            name: "stress".to_string(),
                            hypothesis: "The speaker is feeling stressed, overwhelmed, or under pressure.".to_string(),
                            weight: 0.45,
                            threshold: 0.70,
                            platt_a: 1.0,
                            platt_b: 0.0,
                        },
                    ],
                },
                ScoreCategoryConfig {
                    name: "context".to_string(),
                    top_k: 2,
                    labels: vec![
                        ScoreLabelConfig {
                            name: "phatic".to_string(),
                            hypothesis: "This is just social pleasantry, small talk, or acknowledgment with no substantive information.".to_string(),
                            weight: 0.40,
                            threshold: 0.80,
                            platt_a: 1.0,
                            platt_b: 0.0,
                        },
                        ScoreLabelConfig {
                            name: "task".to_string(),
                            hypothesis: "The speaker is describing something they need to do, remember, or a task to complete.".to_string(),
                            weight: 1.00,
                            threshold: 0.65,
                            platt_a: 1.0,
                            platt_b: 0.0,
                        },
                    ],
                },
            ],
        }
    }

    #[cfg(feature = "int")]
    #[test]
    fn should_cancel() -> Result<()> {
        use loom_pipe::{Build, Pipe};

        let layer = int_test_config().build()?;
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

        let layer = int_test_config().build()?;
        let context = Context::new("oh my god, I'm going to be late for work!", ());
        let res = Source::from(context).pipe(layer).build()?;

        println!("{:#?}", &res);
        Ok(())
    }
}
