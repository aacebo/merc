use std::collections::HashSet;
use std::path::PathBuf;

use clap::Args;
use loom::core::{Format, ident_path};
use loom::io::path::{FilePath, Path};
use loom::runtime::{
    Emitter, FileSystemSource, JsonCodec, Runtime, ScoreConfig, Signal, TomlCodec, YamlCodec, eval,
};

use super::{load_config, resolve_output_path};
use crate::widgets::{self, Widget};

/// Signal emitter that displays progress on stdout.
struct ProgressEmitter;

impl Emitter for ProgressEmitter {
    fn emit(&self, signal: Signal) {
        if signal.name() == "eval.progress" {
            let attrs = signal.attributes();
            let current = attrs.get("current").and_then(|v| v.as_int()).unwrap_or(0);
            let total = attrs.get("total").and_then(|v| v.as_int()).unwrap_or(0);
            let sample_id = attrs
                .get("sample_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let correct = attrs
                .get("correct")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let status = if correct { '✓' } else { '✗' };

            widgets::ProgressBar::new()
                .total(total as usize)
                .current(current as usize)
                .message(sample_id)
                .status(status)
                .render()
                .write();
        }
    }
}

/// Run evaluation against a dataset
#[derive(Debug, Args)]
pub struct RunCommand {
    /// Path to the dataset JSON file
    pub path: PathBuf,

    /// Path to config file (YAML/JSON/TOML)
    #[arg(short, long)]
    pub config: PathBuf,

    /// Output directory for results (default: input file's directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Show detailed per-category and per-label results
    #[arg(short, long)]
    pub verbose: bool,

    /// Number of parallel inference workers (overrides config)
    #[arg(long)]
    pub concurrency: Option<usize>,

    /// Batch size for ML inference (overrides config)
    #[arg(long)]
    pub batch_size: Option<usize>,

    /// Fail if samples have categories/labels not in config (overrides config)
    #[arg(long)]
    pub strict: Option<bool>,
}

impl RunCommand {
    pub async fn exec(self) {
        let path = &self.path;
        let config_path = &self.config;
        let output = self.output.as_ref();
        let verbose = self.verbose;
        let concurrency = self.concurrency;
        let batch_size = self.batch_size;
        let strict = self.strict;

        println!("Loading config from {:?}...", config_path);

        let config = match load_config(config_path.to_str().unwrap_or_default()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                std::process::exit(1);
            }
        };

        println!("Building runtime (this may download model files on first run)...");

        // Build runtime with config in blocking task (scorer building uses rust-bert which conflicts with tokio)
        let runtime = match tokio::task::spawn_blocking(move || {
            Runtime::new()
                .source(FileSystemSource::builder().build())
                .codec(JsonCodec::new())
                .codec(YamlCodec::new())
                .codec(TomlCodec::new())
                .config(config)
                .emitter(ProgressEmitter)
                .build()
        })
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error building runtime: {}", e);
                std::process::exit(1);
            }
        };

        // Get runtime settings
        let loom_config = runtime.config();

        // Merge CLI args with config values (CLI overrides config)
        let output_dir = output.or(loom_config.output.as_ref());
        let output_path =
            resolve_output_path(path, output_dir.map(|p| p.as_path()), "results.json");
        let batch_size = batch_size.unwrap_or(loom_config.batch_size);
        let strict = strict.unwrap_or(loom_config.strict);
        let _ = concurrency; // Reserved for future multi-model parallelism

        // Get score config for validation
        let score_path = ident_path!("layers.score");
        let score_config: ScoreConfig = match runtime.rconfig().get_section(&score_path).bind() {
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

        println!("Loading dataset from {:?}...", path);

        let file_path = Path::File(FilePath::from(path.clone()));
        let mut dataset: eval::SampleDataset = match runtime.load("file_system", &file_path).await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error loading dataset: {}", e);
                std::process::exit(1);
            }
        };

        println!("Loaded {} samples", dataset.samples.len());

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
                let valid_label_set: HashSet<&str> =
                    valid_labels.iter().map(|s| s.as_str()).collect();

                let original_count = dataset.samples.len();
                dataset.samples.retain(|sample| {
                    let category_valid =
                        valid_category_set.contains(sample.primary_category.as_str());
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

        let total = dataset.samples.len();
        println!("\nRunning benchmark with batch size {}...\n", batch_size);

        let result = match runtime.eval_scoring(&dataset, batch_size).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error running evaluation: {}", e);
                std::process::exit(1);
            }
        };

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

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Error creating output directory: {}", e);
                std::process::exit(1);
            }
        }

        // Write results to output file
        let file_path = Path::File(FilePath::from(output_path.clone()));
        if let Err(e) = runtime
            .save("file_system", &file_path, &result, Format::Json)
            .await
        {
            eprintln!("Error writing output file: {}", e);
            std::process::exit(1);
        }

        println!("\nResults written to {:?}", output_path);
    }
}
