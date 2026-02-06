use std::sync::{Arc, Mutex};
use std::time::Instant;

use futures::stream::{self, StreamExt};
use loom_cortex::bench::Scorer;
use loom_signal::{Emitter, Level, Signal, Span, Type};

use super::config::AsyncRunConfig;
use super::helpers::{build_result, evaluate_sample};
use crate::bench::{BenchDataset, BenchResult, Progress};

/// Run benchmarks with signal instrumentation.
///
/// Emits signals for the overall benchmark run and each sample evaluation.
/// - `bench.run` span: emitted at start/end of benchmark with total samples and accuracy
/// - `bench.sample` span: emitted for each sample with sample_id, correct, and score
pub fn run_with_signals<S: Scorer, E: Emitter>(
    dataset: &BenchDataset,
    scorer: &S,
    emitter: &E,
) -> BenchResult {
    run_with_signals_and_progress(dataset, scorer, emitter, |_| {})
}

/// Run benchmarks with signal instrumentation and progress callback.
pub fn run_with_signals_and_progress<S: Scorer, E: Emitter>(
    dataset: &BenchDataset,
    scorer: &S,
    emitter: &E,
    on_progress: impl Fn(Progress),
) -> BenchResult {
    let run_span = Span::new("bench.run")
        .with_level(Level::Info)
        .with_attr("total_samples", dataset.samples.len() as i64);

    let samples_and_results: Vec<_> = dataset
        .samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            let sample_start = Instant::now();
            let sample_result = evaluate_sample(sample, scorer);
            let sample_duration = sample_start.elapsed();

            // Emit sample signal
            emitter.emit(
                Signal::new()
                    .otype(Type::Span)
                    .level(Level::Debug)
                    .name("bench.sample")
                    .attr("sample_id", sample.id.clone())
                    .attr("sample_index", i as i64)
                    .attr("correct", sample_result.correct)
                    .attr("score", sample_result.score)
                    .attr("duration_ms", sample_duration.as_millis() as i64)
                    .build(),
            );

            on_progress(Progress {
                current: i + 1,
                total: dataset.samples.len(),
                sample_id: sample.id.clone(),
                correct: sample_result.correct,
            });

            (sample.clone(), sample_result)
        })
        .collect();

    let result = build_result(samples_and_results);

    // Emit completion signal
    let accuracy = if result.total > 0 {
        result.correct as f64 / result.total as f64
    } else {
        0.0
    };

    emitter.emit(
        run_span
            .with_attr("correct", result.correct as i64)
            .with_attr("accuracy", accuracy)
            .finish(),
    );

    result
}

/// Run benchmarks asynchronously with signal instrumentation.
pub async fn run_async_with_signals<S, E>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    emitter: Arc<E>,
) -> BenchResult
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    E: Emitter + Send + Sync + 'static,
{
    run_async_with_signals_and_config(dataset, scorer, emitter, AsyncRunConfig::default(), |_| {})
        .await
}

/// Run benchmarks asynchronously with signal instrumentation and config.
pub async fn run_async_with_signals_and_config<S, E, F>(
    dataset: &BenchDataset,
    scorer: Arc<Mutex<S>>,
    emitter: Arc<E>,
    _config: AsyncRunConfig,
    on_progress: F,
) -> BenchResult
where
    S: Scorer + Send + 'static,
    S::Output: Send + 'static,
    S::Error: Send + 'static,
    E: Emitter + Send + Sync + 'static,
    F: Fn(Progress) + Send + Sync + 'static,
{
    let run_span = Span::new("bench.run")
        .with_level(Level::Info)
        .with_attr("total_samples", dataset.samples.len() as i64);

    let total = dataset.samples.len();
    let on_progress = Arc::new(on_progress);

    let sample_results: Vec<_> = stream::iter(dataset.samples.iter().cloned().enumerate())
        .then(|(i, sample)| {
            let scorer = scorer.clone();
            let sample_clone = sample.clone();
            let on_progress = on_progress.clone();
            let emitter = emitter.clone();
            async move {
                let sample_start = Instant::now();

                let result = tokio::task::spawn_blocking(move || {
                    let scorer = scorer.lock().expect("scorer lock poisoned");
                    evaluate_sample(&sample_clone, &*scorer)
                })
                .await
                .expect("spawn_blocking failed");

                let sample_duration = sample_start.elapsed();

                // Emit sample signal
                emitter.emit(
                    Signal::new()
                        .otype(Type::Span)
                        .level(Level::Debug)
                        .name("bench.sample")
                        .attr("sample_id", sample.id.clone())
                        .attr("sample_index", i as i64)
                        .attr("correct", result.correct)
                        .attr("score", result.score)
                        .attr("duration_ms", sample_duration.as_millis() as i64)
                        .build(),
                );

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

    let result = build_result(sample_results);

    // Emit completion signal
    let accuracy = if result.total > 0 {
        result.correct as f64 / result.total as f64
    } else {
        0.0
    };

    emitter.emit(
        run_span
            .with_attr("correct", result.correct as i64)
            .with_attr("accuracy", accuracy)
            .finish(),
    );

    result
}
