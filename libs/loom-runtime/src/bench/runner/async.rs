use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::stream::{self, StreamExt};
use loom_cortex::bench::platt::{RawScoreExport, SampleScores};
use loom_cortex::bench::{Scorer, ScorerOutput};

use super::config::AsyncRunConfig;
use super::helpers::{build_result, evaluate_sample};
use crate::bench::{BenchDataset, BenchResult, Progress};

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
    let sample_results: Vec<_> = stream::iter(dataset.samples.iter().cloned().enumerate())
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

                (sample, result)
            }
        })
        .collect()
        .await;

    build_result(sample_results)
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

    let sample_scores: Vec<SampleScores> =
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

                    SampleScores {
                        id: sample_id,
                        text: sample.text.clone(),
                        scores,
                        expected_labels,
                    }
                }
            })
            .collect()
            .await;

    RawScoreExport {
        samples: sample_scores,
    }
}
