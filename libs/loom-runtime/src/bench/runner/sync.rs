use std::collections::HashMap;

use loom_cortex::bench::platt::{RawScoreExport, SampleScores};
use loom_cortex::bench::{Scorer, ScorerOutput};

use super::helpers::{build_result, evaluate_sample};
use crate::bench::{BenchDataset, BenchResult, Progress};

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
    let samples_and_results: Vec<_> = dataset
        .samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            let sample_result = evaluate_sample(sample, scorer);

            on_progress(Progress {
                current: i + 1,
                total: dataset.samples.len(),
                sample_id: sample.id.clone(),
                correct: sample_result.correct,
            });

            (sample.clone(), sample_result)
        })
        .collect();

    build_result(samples_and_results)
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
