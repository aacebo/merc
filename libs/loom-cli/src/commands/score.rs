use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use loom::core::Format;
use loom::core::path::IdentPath;
use loom::io::path::{FilePath, Path};
use loom::runtime::eval::score::ScoreLayer;
use loom::runtime::{LoomConfig, ScoreConfig, eval};

use super::{build_runtime, load_config};
use crate::widgets::{self, Widget};

pub async fn exec(
    path: &PathBuf,
    config_path: &PathBuf,
    output: Option<&PathBuf>,
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
    let output = output
        .or(loom_config.output.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Error: Output path required (--output or config 'output' field)");
            std::process::exit(1);
        });
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

    println!(
        "\nExtracting raw scores with batch size {}...\n",
        batch_size
    );

    let total = dataset.samples.len();
    let scorer = Arc::new(Mutex::new(scorer));

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

    // Run benchmark and capture raw scores
    let (result, raw_scores) =
        ScoreLayer::eval_with_scores(scorer, &dataset, batch_size, progress_callback).await;

    // Clear the progress line
    widgets::ProgressBar::clear();
    println!("Scored {} samples", total);

    // Build hierarchical export
    let export = eval::ScoreExport::from_results(&dataset, &result, raw_scores);

    // Display summary
    let metrics = result.metrics();
    println!("\n========================================");
    println!(
        "  SCORE: {}/100 ({:.1}%)",
        (metrics.accuracy * 100.0).round() as u32,
        metrics.accuracy * 100.0
    );
    println!("========================================\n");

    // Write to output file using runtime
    let output_path = Path::File(FilePath::from(output.clone()));
    if let Err(e) = runtime
        .save("file_system", &output_path, &export, Format::Json)
        .await
    {
        eprintln!("Error writing output file: {}", e);
        std::process::exit(1);
    }

    println!("Score export written to {:?}", output);
}
