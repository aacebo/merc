use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use loom::core::path::IdentPath;
use loom::io::path::{FilePath, Path};
use loom::runtime::eval::score::ScoreLayer;
use loom::runtime::{LoomConfig, ScoreConfig, eval};

use super::{build_runtime, load_config};
use crate::widgets::{self, Widget};

pub async fn exec(
    path: &PathBuf,
    config_path: &PathBuf,
    verbose: bool,
    concurrency: Option<usize>,
    batch_size: Option<usize>,
    strict: Option<bool>,
) {
    println!("Loading dataset from {:?}...", path);

    let runtime = build_runtime();
    let file_path = Path::File(FilePath::from(path.clone()));
    let mut dataset: eval::SampleDataset = match runtime.load("file_system", &file_path).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} samples", dataset.samples.len());
    println!("Loading config from {:?}...", config_path);

    let config = match load_config(config_path.to_str().unwrap_or_default()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Bind runtime settings from root
    let loom_config: LoomConfig = match config.root_section().bind() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing runtime config: {}", e);
            std::process::exit(1);
        }
    };

    // Merge CLI args with config values (CLI overrides config)
    let batch_size = batch_size.unwrap_or(loom_config.batch_size);
    let strict = strict.unwrap_or(loom_config.strict);
    let _ = concurrency; // Reserved for future multi-model parallelism

    // Get score config from layers dynamically
    let score_path = IdentPath::parse("layers.score").expect("valid path");
    let score_section = config.get_section(&score_path);
    let score_config: ScoreConfig = match score_section.bind() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing score config: {}", e);
            std::process::exit(1);
        }
    };

    // Extract valid categories and labels from config
    let valid_categories: Vec<String> = score_config.categories.keys().cloned().collect();
    let valid_labels: Vec<String> = score_config
        .categories
        .values()
        .flat_map(|c| c.labels.keys().cloned())
        .collect();

    // Validate dataset against config
    let errors = dataset.validate_with_config(Some(&valid_categories), Some(&valid_labels));

    if !errors.is_empty() {
        if strict {
            eprintln!("Validation failed with {} error(s):", errors.len());
            for error in &errors {
                eprintln!("  - {}", error);
            }
            std::process::exit(1);
        } else {
            // Filter out invalid samples
            let valid_category_set: HashSet<&str> =
                valid_categories.iter().map(|s| s.as_str()).collect();
            let valid_label_set: HashSet<&str> = valid_labels.iter().map(|s| s.as_str()).collect();

            let original_count = dataset.samples.len();
            dataset.samples.retain(|sample| {
                let category_valid = valid_category_set.contains(sample.primary_category.as_str());
                let labels_valid = sample
                    .expected_labels
                    .iter()
                    .all(|l| valid_label_set.contains(l.as_str()));
                category_valid && labels_valid
            });

            let skipped = original_count - dataset.samples.len();
            if skipped > 0 {
                eprintln!(
                    "Warning: Skipping {} samples with unknown categories/labels",
                    skipped
                );
            }
        }
    }

    if dataset.samples.is_empty() {
        eprintln!("Error: No valid samples remaining after filtering");
        std::process::exit(1);
    }

    println!("Building scorer (this may download model files on first run)...");

    // Build scorer in blocking task to avoid tokio runtime conflict with rust-bert
    let scorer = match tokio::task::spawn_blocking(move || score_config.build())
        .await
        .expect("spawn_blocking failed")
    {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error building scorer: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nRunning benchmark with batch size {}...\n", batch_size);

    let scorer = Arc::new(Mutex::new(scorer));
    let total = dataset.samples.len();

    let progress_callback = |p: eval::Progress| {
        let status = if p.correct { '✓' } else { '✗' };
        widgets::ProgressBar::new()
            .total(p.total)
            .current(p.current)
            .message(&p.sample_id)
            .status(status)
            .render()
            .write();
    };

    let result = ScoreLayer::eval(scorer, &dataset, batch_size, progress_callback).await;

    // Clear the progress line
    widgets::ProgressBar::clear();
    println!("Completed {} samples\n", total);

    // Compute metrics from raw counts
    let metrics = result.metrics();

    // Display prominent score summary
    let score_out_of_100 = (metrics.accuracy * 100.0).round() as u32;
    println!("========================================");
    println!(
        "  SCORE: {}/100 ({:.1}%)",
        score_out_of_100,
        metrics.accuracy * 100.0
    );
    println!("========================================\n");

    println!("=== Benchmark Results ===\n");
    println!("Total samples: {}", result.total);
    println!(
        "Correct:       {} ({:.1}%)",
        result.correct,
        metrics.accuracy * 100.0
    );
    println!();
    println!("Precision: {:.3}", metrics.precision);
    println!("Recall:    {:.3}", metrics.recall);
    println!("F1 Score:  {:.3}", metrics.f1);

    if verbose {
        println!("\n=== Per-Category Results ===\n");
        let mut categories: Vec<_> = result.per_category.iter().collect();
        categories.sort_by_key(|(cat, _)| cat.as_str());

        for (category, cat_result) in categories {
            let cat_metrics = metrics.per_category.get(category);
            let accuracy = cat_metrics.map(|m| m.accuracy).unwrap_or(0.0);
            println!(
                "{:20} {:3}/{:3} ({:.1}%)",
                category,
                cat_result.correct,
                cat_result.total,
                accuracy * 100.0
            );
        }

        println!("\n=== Per-Label Results ===\n");

        let mut labels: Vec<_> = result.per_label.iter().collect();
        labels.sort_by_key(|(label, _)| label.as_str());

        let mut table = widgets::Table::new().headers(vec![
            "Label", "Expect", "Detect", "TP", "Prec", "Recall", "F1",
        ]);

        for (label, label_result) in labels {
            if label_result.expected_count > 0 || label_result.detected_count > 0 {
                let label_metrics = metrics.per_label.get(label);
                let (precision, recall, f1) = label_metrics
                    .map(|m| (m.precision, m.recall, m.f1))
                    .unwrap_or((0.0, 0.0, 0.0));
                table = table.row(vec![
                    label.to_string(),
                    label_result.expected_count.to_string(),
                    label_result.detected_count.to_string(),
                    label_result.true_positives.to_string(),
                    format!("{:.3}", precision),
                    format!("{:.3}", recall),
                    format!("{:.3}", f1),
                ]);
            }
        }

        print!("{}", table);

        // Show misclassified samples
        let incorrect: Vec<_> = result
            .sample_results
            .iter()
            .filter(|s| !s.correct)
            .collect();

        if !incorrect.is_empty() {
            println!("\n=== Misclassified Samples ({}) ===\n", incorrect.len());
            for sample in incorrect.iter().take(10) {
                println!("ID: {}", sample.id);
                println!(
                    "  Expected: {:?}, Actual: {:?}",
                    sample.expected_decision, sample.actual_decision
                );
                println!("  Score: {:.3}", sample.score);
                println!("  Expected labels: {:?}", sample.expected_labels);
                println!("  Detected labels: {:?}", sample.detected_labels);
                println!();
            }
            if incorrect.len() > 10 {
                println!("... and {} more", incorrect.len() - 10);
            }
        }
    }
}
