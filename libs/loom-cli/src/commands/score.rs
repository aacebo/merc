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

/// Signal emitter that displays scoring progress on stdout.
struct ScoreProgressEmitter {
    total: usize,
}

impl ScoreProgressEmitter {
    fn new(total: usize) -> Self {
        Self { total }
    }
}

impl Emitter for ScoreProgressEmitter {
    fn emit(&self, signal: Signal) {
        if signal.name() == "eval.progress" {
            let attrs = signal.attributes();
            let current = attrs.get("current").and_then(|v| v.as_int()).unwrap_or(0);
            let sample_id = attrs
                .get("sample_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            widgets::ProgressBar::new()
                .total(self.total)
                .current(current as usize)
                .message(sample_id)
                .status('•')
                .render()
                .write();
        }
    }
}

/// Extract raw scores for Platt calibration training
#[derive(Debug, Args)]
pub struct ScoreCommand {
    /// Path to the dataset JSON file
    pub path: PathBuf,

    /// Path to config file (YAML/JSON/TOML)
    #[arg(short, long)]
    pub config: PathBuf,

    /// Output directory for scores (default: input file's directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

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

impl ScoreCommand {
    pub async fn exec(self) {
        let path = &self.path;
        let config_path = &self.config;
        let output = self.output.as_ref();
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
        // Note: We'll add the progress emitter after loading the dataset to know the total count
        let runtime = match tokio::task::spawn_blocking(move || {
            Runtime::new()
                .source(FileSystemSource::builder().build())
                .codec(JsonCodec::new())
                .codec(YamlCodec::new())
                .codec(TomlCodec::new())
                .config(config)
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
        let output_path = resolve_output_path(path, output_dir.map(|p| p.as_path()), "scores.json");
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
        println!(
            "\nExtracting raw scores with batch size {}...\n",
            batch_size
        );

        // Rebuild runtime with progress emitter now that we know the total
        let config = match load_config(config_path.to_str().unwrap_or_default()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reloading config: {}", e);
                std::process::exit(1);
            }
        };

        let runtime = match tokio::task::spawn_blocking(move || {
            Runtime::new()
                .source(FileSystemSource::builder().build())
                .codec(JsonCodec::new())
                .codec(YamlCodec::new())
                .codec(TomlCodec::new())
                .config(config)
                .emitter(ScoreProgressEmitter::new(total))
                .build()
        })
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error rebuilding runtime: {}", e);
                std::process::exit(1);
            }
        };

        // Use runtime.eval_scoring_with_scores() for batch processing
        let (result, raw_scores) =
            match runtime.eval_scoring_with_scores(&dataset, batch_size).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error during scoring: {}", e);
                    std::process::exit(1);
                }
            };

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

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Error creating output directory: {}", e);
                std::process::exit(1);
            }
        }

        // Write to output file using runtime
        let file_path = Path::File(FilePath::from(output_path.clone()));
        if let Err(e) = runtime
            .save("file_system", &file_path, &export, Format::Json)
            .await
        {
            eprintln!("Error writing output file: {}", e);
            std::process::exit(1);
        }

        println!("Score export written to {:?}", output_path);
    }
}
