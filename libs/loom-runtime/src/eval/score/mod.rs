mod config;
mod result;

pub use config::*;
pub use result::*;

use std::collections::{BTreeMap, HashMap};

use loom_cortex::CortexModel;
use loom_cortex::bench::Decision;
use loom_error::{Error, ErrorCode};
use loom_pipe::Build;

use crate::Context;
use loom_pipe::LayerResult;

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
            .values()
            .flat_map(|c| c.labels.keys().map(|s| s.as_str()))
            .collect();

        // Build a static hypothesis map for the closure
        let hypothesis_map: std::collections::HashMap<String, String> = self
            .config
            .categories
            .values()
            .flat_map(|c| {
                c.labels
                    .iter()
                    .map(|(name, l)| (name.clone(), l.hypothesis.clone()))
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
        let mut categories = BTreeMap::new();

        for (cat_name, cat_config) in &self.config.categories {
            let mut labels = BTreeMap::new();

            for (label_name, label_config) in &cat_config.labels {
                let raw_score = prediction_map
                    .get(label_name.as_str())
                    .copied()
                    .unwrap_or(0.0);

                let score_label = ScoreLabel::new(raw_score, 0, label_config);
                labels.insert(label_name.clone(), score_label);
            }

            let top_k = cat_config.top_k;
            categories.insert(cat_name.clone(), ScoreCategory::topk(labels, top_k));
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
        let elapsed_ms = (chrono::Utc::now() - started_at).num_milliseconds();
        result.meta.set("elapsed_ms", elapsed_ms.into());
        result
            .meta
            .set("start_time", started_at.to_rfc3339().into());
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

impl loom_pipe::Layer for ScoreLayer {
    type Input = Context<()>;
    type Output = ScoreResult;

    fn process(&self, input: Self::Input) -> loom_error::Result<LayerResult<Self::Output>> {
        self.invoke(input)
    }

    fn name(&self) -> &'static str {
        "ScoreLayer"
    }
}

/// Wrapper around ScoreResult for evaluation output.
pub struct ScoreLayerOutput(ScoreResult);

impl ScoreLayerOutput {
    pub fn new(result: ScoreResult) -> Self {
        Self(result)
    }

    /// Get the underlying ScoreResult.
    pub fn inner(&self) -> &ScoreResult {
        &self.0
    }

    /// The decision (Accept/Reject) for this scoring.
    /// If we got a successful result, it's Accept.
    /// (Reject happens when invoke returns an error)
    pub fn decision(&self) -> Decision {
        Decision::Accept
    }

    /// The overall score value.
    pub fn score(&self) -> f32 {
        self.0.score
    }

    /// Labels with their raw (uncalibrated) scores.
    pub fn labels(&self) -> Vec<(String, f32)> {
        self.0.raw_scores()
    }

    /// Labels that were detected (score > 0).
    pub fn detected_labels(&self) -> Vec<String> {
        self.labels()
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(name, _)| name)
            .collect()
    }
}

impl ScoreLayer {
    /// Score a single text.
    pub fn score(&self, text: &str) -> loom_error::Result<ScoreLayerOutput> {
        let ctx = Context::new(text, ());
        self.invoke(ctx).map(|r| ScoreLayerOutput::new(r.output))
    }

    /// Score multiple texts in a single batch.
    /// This is more efficient than scoring texts one at a time.
    pub fn score_batch(&self, texts: &[&str]) -> loom_error::Result<Vec<ScoreLayerOutput>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

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
            .values()
            .flat_map(|c| c.labels.keys().map(|s| s.as_str()))
            .collect();

        // Build a static hypothesis map for the closure
        let hypothesis_map: std::collections::HashMap<String, String> = self
            .config
            .categories
            .values()
            .flat_map(|c| {
                c.labels
                    .iter()
                    .map(|(name, l)| (name.clone(), l.hypothesis.clone()))
            })
            .collect();

        // Create hypothesis function using the cloned map
        let hypothesis_fn = Box::new(move |label: &str| {
            hypothesis_map
                .get(label)
                .cloned()
                .unwrap_or_else(|| format!("This example is {}.", label))
        });

        // Run zero-shot classification on ALL texts at once (batch inference)
        let predictions =
            zs_model.predict_multilabel(texts, &label_names, Some(hypothesis_fn), 128)?;

        // Process predictions for each text
        let mut outputs = Vec::with_capacity(texts.len());

        for sentence_predictions in &predictions {
            // Build a lookup map for this text's predictions by label name
            let mut prediction_map: HashMap<&str, f32> = HashMap::new();
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

            // Build ScoreCategory for each category in config
            let mut categories = BTreeMap::new();

            for (cat_name, cat_config) in &self.config.categories {
                let mut labels = BTreeMap::new();

                for (label_name, label_config) in &cat_config.labels {
                    let raw_score = prediction_map
                        .get(label_name.as_str())
                        .copied()
                        .unwrap_or(0.0);

                    let score_label = ScoreLabel::new(raw_score, 0, label_config);
                    labels.insert(label_name.clone(), score_label);
                }

                let top_k = cat_config.top_k;
                categories.insert(cat_name.clone(), ScoreCategory::topk(labels, top_k));
            }

            outputs.push(ScoreLayerOutput::new(ScoreResult::new(categories)));
        }

        Ok(outputs)
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
        use std::collections::BTreeMap;

        let mut sentiment_labels = BTreeMap::new();
        sentiment_labels.insert(
            "positive".to_string(),
            ScoreLabelConfig {
                hypothesis: "The speaker is expressing a positive, happy, or optimistic sentiment."
                    .to_string(),
                weight: 0.30,
                threshold: 0.70,
                platt_a: 1.0,
                platt_b: 0.0,
            },
        );
        sentiment_labels.insert(
            "negative".to_string(),
            ScoreLabelConfig {
                hypothesis:
                    "The speaker is expressing a negative, unhappy, or pessimistic sentiment."
                        .to_string(),
                weight: 0.35,
                threshold: 0.70,
                platt_a: 1.0,
                platt_b: 0.0,
            },
        );

        let mut emotion_labels = BTreeMap::new();
        emotion_labels.insert(
            "stress".to_string(),
            ScoreLabelConfig {
                hypothesis: "The speaker is feeling stressed, overwhelmed, or under pressure."
                    .to_string(),
                weight: 0.45,
                threshold: 0.70,
                platt_a: 1.0,
                platt_b: 0.0,
            },
        );

        let mut context_labels = BTreeMap::new();
        context_labels.insert("phatic".to_string(), ScoreLabelConfig {
            hypothesis: "This is just social pleasantry, small talk, or acknowledgment with no substantive information.".to_string(),
            weight: 0.40,
            threshold: 0.80,
            platt_a: 1.0,
            platt_b: 0.0,
        });
        context_labels.insert("task".to_string(), ScoreLabelConfig {
            hypothesis: "The speaker is describing something they need to do, remember, or a task to complete.".to_string(),
            weight: 1.00,
            threshold: 0.65,
            platt_a: 1.0,
            platt_b: 0.0,
        });

        let mut categories = BTreeMap::new();
        categories.insert(
            "sentiment".to_string(),
            ScoreCategoryConfig {
                top_k: 2,
                labels: sentiment_labels,
            },
        );
        categories.insert(
            "emotion".to_string(),
            ScoreCategoryConfig {
                top_k: 2,
                labels: emotion_labels,
            },
        );
        categories.insert(
            "context".to_string(),
            ScoreCategoryConfig {
                top_k: 2,
                labels: context_labels,
            },
        );

        ScoreConfig {
            model: CortexModelConfig::ZeroShotClassification(CortexZeroShotConfig::default()),
            threshold: 0.40,
            top_k: 2,
            modifiers: ScoreModifierConfig::default(),
            categories,
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
