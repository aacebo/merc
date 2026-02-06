mod config;
mod result;

pub use config::*;
pub use result::*;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use loom_cortex::CortexModel;
use loom_cortex::bench::platt::{RawScoreExport, SampleScores};
use loom_cortex::bench::{BatchScorer, Decision, Scorer, ScorerOutput};
use loom_error::{Error, ErrorCode};
use loom_pipe::Build;

use super::{EvalResult, LabelResult, Progress, Sample, SampleDataset, SampleResult};
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
        self.0.raw_scores()
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

impl BatchScorer for ScoreLayer {
    type Output = ScoreLayerOutput;
    type Error = Error;

    fn score_batch(&self, texts: &[&str]) -> Result<Vec<Self::Output>, Self::Error> {
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

    fn batch_size(&self) -> usize {
        16 // Optimal batch size for zero-shot classification
    }
}

// =============================================================================
// Dataset Evaluation Methods
// =============================================================================

impl ScoreLayer {
    /// Evaluate a dataset using batch inference.
    ///
    /// This is the primary entry point for running evaluations. The scorer must be
    /// wrapped in `Arc<Mutex<_>>` for thread-safe async execution.
    ///
    /// # Example
    /// ```ignore
    /// let scorer = Arc::new(Mutex::new(score_config.build()?));
    /// let result = ScoreLayer::eval(scorer, &dataset, 16, |p| println!("{}/{}", p.current, p.total)).await;
    /// ```
    pub async fn eval<F>(
        scorer: Arc<Mutex<Self>>,
        dataset: &SampleDataset,
        batch_size: usize,
        on_progress: F,
    ) -> EvalResult
    where
        F: Fn(Progress) + Send + Sync + 'static,
    {
        let total = dataset.samples.len();
        let on_progress = Arc::new(on_progress);

        // Collect all samples with their original indices
        let indexed_samples: Vec<(usize, Sample)> =
            dataset.samples.iter().cloned().enumerate().collect();

        // Process samples in batches
        let mut all_results: Vec<(Sample, SampleResult)> = Vec::with_capacity(total);
        let mut processed = 0;

        for chunk in indexed_samples.chunks(batch_size) {
            let batch_samples: Vec<(usize, Sample)> = chunk.to_vec();
            let texts: Vec<String> = batch_samples.iter().map(|(_, s)| s.text.clone()).collect();
            let scorer = scorer.clone();
            let on_progress = on_progress.clone();

            // Process batch in spawn_blocking
            let batch_outputs = tokio::task::spawn_blocking(move || {
                let scorer = scorer.lock().expect("scorer lock poisoned");
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                scorer.score_batch(&text_refs)
            })
            .await
            .expect("spawn_blocking failed");

            // Evaluate each sample in the batch
            match batch_outputs {
                Ok(outputs) => {
                    for ((_idx, sample), output) in
                        batch_samples.into_iter().zip(outputs.into_iter())
                    {
                        let sample_result = Self::evaluate_output(&sample, output);

                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: sample_result.correct,
                        });

                        all_results.push((sample, sample_result));
                    }
                }
                Err(_) => {
                    // On batch error, mark all samples as rejected
                    for (_idx, sample) in batch_samples {
                        let sample_result = SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision: Decision::Reject,
                            correct: sample.expected_decision == Decision::Reject,
                            score: 0.0,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels: vec![],
                        };

                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: sample_result.correct,
                        });

                        all_results.push((sample, sample_result));
                    }
                }
            }
        }

        Self::build_result(all_results)
    }

    /// Evaluate a dataset and capture raw scores.
    ///
    /// Returns both the evaluation result and a map of sample_id -> label -> raw_score.
    /// This is useful for score extraction and Platt calibration.
    pub async fn eval_with_scores<F>(
        scorer: Arc<Mutex<Self>>,
        dataset: &SampleDataset,
        batch_size: usize,
        on_progress: F,
    ) -> (EvalResult, HashMap<String, HashMap<String, f32>>)
    where
        F: Fn(Progress) + Send + Sync + 'static,
    {
        let total = dataset.samples.len();
        let on_progress = Arc::new(on_progress);

        let indexed_samples: Vec<(usize, Sample)> =
            dataset.samples.iter().cloned().enumerate().collect();

        let mut all_results: Vec<(Sample, SampleResult, HashMap<String, f32>)> =
            Vec::with_capacity(total);
        let mut processed = 0;

        for chunk in indexed_samples.chunks(batch_size) {
            let batch_samples: Vec<(usize, Sample)> = chunk.to_vec();
            let texts: Vec<String> = batch_samples.iter().map(|(_, s)| s.text.clone()).collect();
            let scorer = scorer.clone();
            let on_progress = on_progress.clone();

            let batch_outputs = tokio::task::spawn_blocking(move || {
                let scorer = scorer.lock().expect("scorer lock poisoned");
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                scorer.score_batch(&text_refs)
            })
            .await
            .expect("spawn_blocking failed");

            match batch_outputs {
                Ok(outputs) => {
                    for ((_idx, sample), output) in
                        batch_samples.into_iter().zip(outputs.into_iter())
                    {
                        let raw_scores: HashMap<String, f32> =
                            output.labels().into_iter().collect();
                        let sample_result = Self::evaluate_output(&sample, output);

                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: sample_result.correct,
                        });

                        all_results.push((sample, sample_result, raw_scores));
                    }
                }
                Err(_) => {
                    for (_idx, sample) in batch_samples {
                        let sample_result = SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision: Decision::Reject,
                            correct: sample.expected_decision == Decision::Reject,
                            score: 0.0,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels: vec![],
                        };

                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: sample_result.correct,
                        });

                        all_results.push((sample, sample_result, HashMap::new()));
                    }
                }
            }
        }

        Self::build_result_with_scores(all_results)
    }

    /// Export raw scores for Platt calibration training.
    ///
    /// This exports the raw model scores for each sample, which can be used
    /// to train Platt calibration parameters.
    pub async fn export_scores<F>(
        scorer: Arc<Mutex<Self>>,
        dataset: &SampleDataset,
        batch_size: usize,
        on_progress: F,
    ) -> RawScoreExport
    where
        F: Fn(Progress) + Send + Sync + 'static,
    {
        let total = dataset.samples.len();
        let on_progress = Arc::new(on_progress);

        // Collect samples
        let indexed_samples: Vec<(usize, Sample)> =
            dataset.samples.iter().cloned().enumerate().collect();

        // Process in batches
        let mut all_scores: Vec<SampleScores> = Vec::with_capacity(total);
        let mut processed = 0;

        for chunk in indexed_samples.chunks(batch_size) {
            let batch_samples: Vec<(usize, Sample)> = chunk.to_vec();
            let texts: Vec<String> = batch_samples.iter().map(|(_, s)| s.text.clone()).collect();
            let scorer = scorer.clone();
            let on_progress = on_progress.clone();

            // Process batch
            let batch_outputs = tokio::task::spawn_blocking(move || {
                let scorer = scorer.lock().expect("scorer lock poisoned");
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                scorer.score_batch(&text_refs)
            })
            .await
            .expect("spawn_blocking failed");

            match batch_outputs {
                Ok(outputs) => {
                    for ((_idx, sample), output) in
                        batch_samples.into_iter().zip(outputs.into_iter())
                    {
                        let mut scores = HashMap::new();
                        for (name, raw_score) in output.labels() {
                            scores.insert(name, raw_score);
                        }

                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: true,
                        });

                        all_scores.push(SampleScores {
                            id: sample.id.clone(),
                            text: sample.text.clone(),
                            scores,
                            expected_labels: sample.expected_labels.clone(),
                        });
                    }
                }
                Err(_) => {
                    // On batch error, push empty scores
                    for (_idx, sample) in batch_samples {
                        processed += 1;
                        on_progress(Progress {
                            current: processed,
                            total,
                            sample_id: sample.id.clone(),
                            correct: true,
                        });

                        all_scores.push(SampleScores {
                            id: sample.id.clone(),
                            text: sample.text.clone(),
                            scores: HashMap::new(),
                            expected_labels: sample.expected_labels.clone(),
                        });
                    }
                }
            }
        }

        RawScoreExport {
            samples: all_scores,
        }
    }

    // === Private helper methods ===

    /// Evaluate a batch output for a sample.
    fn evaluate_output<O: ScorerOutput>(sample: &Sample, output: O) -> SampleResult {
        let detected_labels = output.detected_labels();
        let actual_decision = output.decision();
        let score = output.score();

        SampleResult {
            id: sample.id.clone(),
            expected_decision: sample.expected_decision,
            actual_decision,
            correct: actual_decision == sample.expected_decision,
            score,
            expected_labels: sample.expected_labels.clone(),
            detected_labels,
        }
    }

    /// Update per-label metrics based on sample results.
    fn update_label_metrics(
        per_label: &mut HashMap<String, LabelResult>,
        sample: &Sample,
        sample_result: &SampleResult,
    ) {
        let expected_set: HashSet<_> = sample.expected_labels.iter().collect();
        let detected_set: HashSet<_> = sample_result.detected_labels.iter().collect();

        for label in &sample.expected_labels {
            let entry = per_label.entry(label.clone()).or_default();
            entry.expected_count += 1;
        }

        for label in &sample_result.detected_labels {
            let entry = per_label.entry(label.clone()).or_default();
            entry.detected_count += 1;

            if expected_set.contains(label) {
                entry.true_positives += 1;
            } else {
                entry.false_positives += 1;
            }
        }

        for label in &sample.expected_labels {
            if !detected_set.contains(label) {
                let entry = per_label.entry(label.clone()).or_default();
                entry.false_negatives += 1;
            }
        }
    }

    /// Build a EvalResult from sample results.
    fn build_result(samples_and_results: Vec<(Sample, SampleResult)>) -> EvalResult {
        let mut result = EvalResult::new();
        result.total = samples_and_results.len();

        for (sample, sample_result) in samples_and_results {
            if sample_result.correct {
                result.correct += 1;
            }

            let cat_result = result
                .per_category
                .entry(sample.primary_category.clone())
                .or_default();
            cat_result.total += 1;
            if sample_result.correct {
                cat_result.correct += 1;
            }

            Self::update_label_metrics(&mut result.per_label, &sample, &sample_result);
            result.sample_results.push(sample_result);
        }

        result
    }

    /// Build a EvalResult from sample results with raw scores.
    fn build_result_with_scores(
        samples_and_results: Vec<(Sample, SampleResult, HashMap<String, f32>)>,
    ) -> (EvalResult, HashMap<String, HashMap<String, f32>>) {
        let mut result = EvalResult::new();
        let mut raw_scores_map: HashMap<String, HashMap<String, f32>> = HashMap::new();
        result.total = samples_and_results.len();

        for (sample, sample_result, raw_scores) in samples_and_results {
            if sample_result.correct {
                result.correct += 1;
            }

            let cat_result = result
                .per_category
                .entry(sample.primary_category.clone())
                .or_default();
            cat_result.total += 1;
            if sample_result.correct {
                cat_result.correct += 1;
            }

            Self::update_label_metrics(&mut result.per_label, &sample, &sample_result);
            raw_scores_map.insert(sample_result.id.clone(), raw_scores);
            result.sample_results.push(sample_result);
        }

        (result, raw_scores_map)
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
