use std::collections::{HashMap, HashSet};

use super::{
    BenchDataset, BenchResult, BenchSample, Decision, LabelResult, Progress, RawScoreExport,
    SampleResult, SampleScores, Scorer, ScorerOutput,
};

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

    result.compute_metrics();
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
