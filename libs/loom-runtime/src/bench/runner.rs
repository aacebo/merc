use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use futures::stream::{self, StreamExt};

use super::{
    BenchDataset, BenchResult, BenchSample, Decision, LabelResult, Progress, SampleResult,
};

// Import ML types from cortex
pub use loom_cortex::bench::platt::{RawScoreExport, SampleScores};
pub use loom_cortex::bench::{Scorer, ScorerOutput};

/// Configuration for async benchmark execution.
#[derive(Debug, Clone)]
pub struct AsyncRunConfig {
    /// Maximum number of concurrent inference tasks.
    /// Defaults to 4 for CPU-bound ML inference.
    pub concurrency: usize,
}

impl Default for AsyncRunConfig {
    fn default() -> Self {
        Self { concurrency: 4 }
    }
}

/// Run benchmarks on a dataset using a scorer.
pub fn run<S: Scorer>(dataset: &BenchDataset, scorer: &S) -> BenchResult {
    run_with_progress(dataset, scorer, |_| {})
}

/// Run benchmarks with progress callback.
pub fn run_with_progress<S: Scorer>(
    dataset: &BenchDataset,
    scorer: &S,
    on_progress: impl Fn(Progress),
) -> BenchResult {
    let mut result = BenchResult::new();
    result.total = dataset.samples.len();

    for (i, sample) in dataset.samples.iter().enumerate() {
        let sample_result = evaluate_sample(sample, scorer);

        on_progress(Progress {
            current: i + 1,
            total: result.total,
            sample_id: sample.id.clone(),
            correct: sample_result.correct,
        });

        if sample_result.correct {
            result.correct += 1;
        }

        let cat_result = result
            .per_category
            .entry(sample.primary_category)
            .or_default();
        cat_result.total += 1;
        if sample_result.correct {
            cat_result.correct += 1;
        }

        update_label_metrics(&mut result.per_label, sample, &sample_result);

        result.sample_results.push(sample_result);
    }

    result
}

/// Export raw (uncalibrated) scores for all labels on each sample.
/// Used for training Platt calibration parameters.
pub fn export<S: Scorer>(dataset: &BenchDataset, scorer: &S) -> RawScoreExport {
    export_with_progress(dataset, scorer, |_| {})
}

/// Export raw scores with progress callback.
pub fn export_with_progress<S: Scorer>(
    dataset: &BenchDataset,
    scorer: &S,
    on_progress: impl Fn(Progress),
) -> RawScoreExport {
    let mut samples = Vec::with_capacity(dataset.samples.len());
    let total = dataset.samples.len();

    for (i, sample) in dataset.samples.iter().enumerate() {
        let mut scores = HashMap::new();

        if let Ok(output) = scorer.score(&sample.text) {
            for (name, raw_score) in output.labels() {
                scores.insert(name, raw_score);
            }
        }

        on_progress(Progress {
            current: i + 1,
            total,
            sample_id: sample.id.clone(),
            correct: true,
        });

        samples.push(SampleScores {
            id: sample.id.clone(),
            text: sample.text.clone(),
            scores,
            expected_labels: sample.expected_labels.clone(),
        });
    }

    RawScoreExport { samples }
}

// ============================================================================
// Async Runner Functions
// ============================================================================

/// Run benchmarks asynchronously on a blocking thread pool.
///
/// Uses a `Mutex` to serialize access to the scorer since rust-bert models
/// are not thread-safe. This still provides benefits over sync execution:
/// - Non-blocking async runtime
/// - Progress tracking during inference
/// - Foundation for future worker pool parallelism
pub async fn run_async<S>(dataset: &BenchDataset, scorer: Arc<Mutex<S>>) -> BenchResult
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
{
    run_async_with_config(dataset, scorer, AsyncRunConfig::default(), |_| {}).await
}

/// Run benchmarks asynchronously with configurable concurrency and progress callback.
///
/// Note: The `concurrency` config is currently limited by the Mutex serialization.
/// True parallelism requires multiple model instances (future enhancement).
pub async fn run_async_with_config<S, F>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    _config: AsyncRunConfig,
    on_progress: F,
) -> BenchResult
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    F: Fn(Progress) + Send + Sync + 'static,
{
    let total = dataset.samples.len();
    let on_progress = Arc::new(on_progress);

    // Process samples sequentially via spawn_blocking (Mutex serializes access)
    // This keeps the async runtime free while inference runs on blocking pool
    let sample_results: Vec<(usize, BenchSample, SampleResult)> =
        stream::iter(dataset.samples.iter().cloned().enumerate())
            .then(|(i, sample)| {
                let scorer = scorer.clone();
                let sample_clone = sample.clone();
                let on_progress = on_progress.clone();
                async move {
                    // Use spawn_blocking for CPU-bound rust-bert inference
                    let result = tokio::task::spawn_blocking(move || {
                        let scorer = scorer.lock().expect("scorer lock poisoned");
                        evaluate_sample(&sample_clone, &*scorer)
                    })
                    .await
                    .expect("spawn_blocking failed");

                    on_progress(Progress {
                        current: i + 1,
                        total,
                        sample_id: sample.id.clone(),
                        correct: result.correct,
                    });

                    (i, sample, result)
                }
            })
            .collect()
            .await;

    // Build result (same logic as sync version)
    let mut result = BenchResult::new();
    result.total = total;

    for (_i, sample, sample_result) in sample_results {
        if sample_result.correct {
            result.correct += 1;
        }

        let cat_result = result
            .per_category
            .entry(sample.primary_category)
            .or_default();
        cat_result.total += 1;
        if sample_result.correct {
            cat_result.correct += 1;
        }

        update_label_metrics(&mut result.per_label, &sample, &sample_result);
        result.sample_results.push(sample_result);
    }

    result
}

/// Export raw scores asynchronously on a blocking thread pool.
pub async fn export_async<S>(dataset: &BenchDataset, scorer: Arc<Mutex<S>>) -> RawScoreExport
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
{
    export_async_with_config(dataset, scorer, AsyncRunConfig::default(), |_| {}).await
}

/// Export raw scores asynchronously with configurable concurrency and progress callback.
pub async fn export_async_with_config<S, F>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    _config: AsyncRunConfig,
    on_progress: F,
) -> RawScoreExport
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    F: Fn(Progress) + Send + Sync + 'static,
{
    let total = dataset.samples.len();
    let on_progress = Arc::new(on_progress);

    let sample_scores: Vec<(usize, SampleScores)> =
        stream::iter(dataset.samples.iter().cloned().enumerate())
            .then(|(i, sample)| {
                let scorer = scorer.clone();
                let on_progress = on_progress.clone();
                async move {
                    let text = sample.text.clone();
                    let sample_id = sample.id.clone();
                    let expected_labels = sample.expected_labels.clone();

                    let scores = tokio::task::spawn_blocking(move || {
                        let scorer = scorer.lock().expect("scorer lock poisoned");
                        let mut scores = HashMap::new();
                        if let Ok(output) = scorer.score(&text) {
                            for (name, raw_score) in output.labels() {
                                scores.insert(name, raw_score);
                            }
                        }
                        scores
                    })
                    .await
                    .expect("spawn_blocking failed");

                    on_progress(Progress {
                        current: i + 1,
                        total,
                        sample_id: sample_id.clone(),
                        correct: true,
                    });

                    (
                        i,
                        SampleScores {
                            id: sample_id,
                            text: sample.text.clone(),
                            scores,
                            expected_labels,
                        },
                    )
                }
            })
            .collect()
            .await;

    RawScoreExport {
        samples: sample_scores.into_iter().map(|(_, s)| s).collect(),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn evaluate_sample<S: Scorer>(sample: &BenchSample, scorer: &S) -> SampleResult {
    let (actual_decision, score, detected_labels) = match scorer.score(&sample.text) {
        Ok(output) => {
            let detected = output.detected_labels();
            (output.decision(), output.score(), detected)
        }
        Err(_) => (Decision::Reject, 0.0, vec![]),
    };

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

fn update_label_metrics(
    per_label: &mut HashMap<String, LabelResult>,
    sample: &BenchSample,
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
