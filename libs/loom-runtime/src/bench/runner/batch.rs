use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use loom_cortex::bench::platt::{RawScoreExport, SampleScores};
use loom_cortex::bench::{BatchScorer, ScorerOutput};

use super::config::AsyncRunConfig;
use super::helpers::{build_result, evaluate_batch_output};
use crate::bench::{BenchDataset, BenchResult, BenchSample, Decision, Progress, SampleResult};

/// Run benchmarks using batch inference for improved throughput.
///
/// Unlike the per-sample async runner, this function groups samples into batches
/// and processes them together in a single model forward pass, which is more
/// efficient for ML inference.
pub async fn run_batch_async<S>(dataset: &BenchDataset, scorer: Arc<Mutex<S>>) -> BenchResult
where
    S: BatchScorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
{
    run_batch_async_with_config(dataset, scorer, AsyncRunConfig::default(), |_| {}).await
}

/// Run benchmarks using batch inference with configurable batch size and progress callback.
pub async fn run_batch_async_with_config<S, F>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    config: AsyncRunConfig,
    on_progress: F,
) -> BenchResult
where
    S: BatchScorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    F: Fn(Progress) + Send + Sync + 'static,
{
    let total = dataset.samples.len();
    let on_progress = Arc::new(on_progress);

    // Determine batch size (use config override or scorer's default)
    let batch_size = config.batch_size.unwrap_or_else(|| {
        // Get batch size from scorer (need to lock briefly)
        scorer.lock().expect("scorer lock poisoned").batch_size()
    });

    // Collect all samples with their original indices
    let indexed_samples: Vec<(usize, BenchSample)> =
        dataset.samples.iter().cloned().enumerate().collect();

    // Process samples in batches
    let mut all_results: Vec<(BenchSample, SampleResult)> = Vec::with_capacity(total);
    let mut processed = 0;

    for chunk in indexed_samples.chunks(batch_size) {
        let batch_samples: Vec<(usize, BenchSample)> = chunk.to_vec();
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
                for ((_idx, sample), output) in batch_samples.into_iter().zip(outputs.into_iter()) {
                    let sample_result = evaluate_batch_output(&sample, output);

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

    build_result(all_results)
}

/// Export raw scores using batch inference for improved throughput.
pub async fn export_batch_async<S>(dataset: &BenchDataset, scorer: Arc<Mutex<S>>) -> RawScoreExport
where
    S: BatchScorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
{
    export_batch_async_with_config(dataset, scorer, AsyncRunConfig::default(), |_| {}).await
}

/// Export raw scores using batch inference with configurable batch size and progress callback.
pub async fn export_batch_async_with_config<S, F>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    config: AsyncRunConfig,
    on_progress: F,
) -> RawScoreExport
where
    S: BatchScorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    F: Fn(Progress) + Send + Sync + 'static,
{
    let total = dataset.samples.len();
    let on_progress = Arc::new(on_progress);

    // Determine batch size
    let batch_size = config
        .batch_size
        .unwrap_or_else(|| scorer.lock().expect("scorer lock poisoned").batch_size());

    // Collect samples
    let indexed_samples: Vec<(usize, BenchSample)> =
        dataset.samples.iter().cloned().enumerate().collect();

    // Process in batches
    let mut all_scores: Vec<SampleScores> = Vec::with_capacity(total);
    let mut processed = 0;

    for chunk in indexed_samples.chunks(batch_size) {
        let batch_samples: Vec<(usize, BenchSample)> = chunk.to_vec();
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
                for ((_idx, sample), output) in batch_samples.into_iter().zip(outputs.into_iter()) {
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
