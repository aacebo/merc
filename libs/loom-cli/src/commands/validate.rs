use std::io::stdout;
use std::path::PathBuf;

use clap::Args;
use crossterm::ExecutableCommand;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use loom::core::ident_path;
use loom::io::path::{FilePath, Path};
use loom::runtime::{
    FileSystemSource, JsonCodec, Runtime, ScoreConfig, TomlCodec, YamlCodec, eval,
};

use super::{build_runtime, load_config};
use crate::widgets::{self, Widget};

/// Validate a dataset
#[derive(Debug, Args)]
pub struct ValidateCommand {
    /// Path to the dataset JSON file
    pub path: PathBuf,

    /// Path to config file (YAML/JSON/TOML) for category/label validation
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Fail if samples have categories/labels not in config (default: report errors)
    #[arg(long)]
    pub strict: bool,

    /// Test N samples by running them through the scorer (requires --config)
    #[arg(long)]
    pub test_samples: Option<usize>,
}

impl ValidateCommand {
    pub async fn exec(self) {
        let path = &self.path;
        let config_path = self.config.as_ref();
        let strict = self.strict;
        let test_samples = self.test_samples;

        widgets::Spinner::new()
            .message(format!("Validating dataset at {:?}...", path))
            .render()
            .write();

        let runtime = build_runtime();
        let file_path = Path::File(FilePath::from(path.clone()));
        let dataset: eval::SampleDataset = match runtime.load("file_system", &file_path).await {
            Ok(d) => d,
            Err(e) => {
                widgets::Spinner::clear();
                eprintln!("Error loading dataset: {}", e);
                std::process::exit(1);
            }
        };

        // Load config if provided for category/label validation
        let (valid_categories, valid_labels) = if let Some(cfg_path) = config_path {
            let config = match load_config(cfg_path.to_str().unwrap_or_default()) {
                Ok(c) => c,
                Err(e) => {
                    widgets::Spinner::clear();
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            };

            // Get score config from layers dynamically
            let score_path = ident_path!("layers.score");
            let score_section = config.get_section(&score_path);
            let score_config: ScoreConfig = match score_section.bind() {
                Ok(c) => c,
                Err(e) => {
                    widgets::Spinner::clear();
                    eprintln!("Error parsing score config: {}", e);
                    std::process::exit(1);
                }
            };

            let categories: Vec<String> = score_config.categories.keys().cloned().collect();
            let labels: Vec<String> = score_config
                .categories
                .values()
                .flat_map(|c| c.labels.keys().cloned())
                .collect();

            (Some(categories), Some(labels))
        } else {
            (None, None)
        };

        widgets::Spinner::clear();

        let errors =
            dataset.validate_with_config(valid_categories.as_deref(), valid_labels.as_deref());
        let mut stdout = stdout();

        if errors.is_empty() {
            let _ = stdout.execute(SetForegroundColor(Color::Green));
            print!("✓ ");
            let _ = stdout.execute(ResetColor);
            println!("Dataset is valid ({} samples)", dataset.samples.len());
        } else {
            let _ = stdout.execute(SetForegroundColor(Color::Red));
            print!("✗ ");
            let _ = stdout.execute(ResetColor);
            println!("Found {} validation error(s):\n", errors.len());
            for error in &errors {
                println!("  - {}", error);
            }
            if strict {
                std::process::exit(1);
            }
        }

        // Test samples if requested
        if let Some(n) = test_samples {
            if config_path.is_none() {
                eprintln!("\nError: --test-samples requires --config to be specified");
                std::process::exit(1);
            }

            println!("\nTesting {} sample(s)...", n.min(dataset.samples.len()));

            // Build runtime with scorer config
            let config = match load_config(config_path.unwrap().to_str().unwrap_or_default()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config for testing: {}", e);
                    std::process::exit(1);
                }
            };

            let scoring_runtime = match tokio::task::spawn_blocking(move || {
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
                    eprintln!("Error building runtime for testing: {}", e);
                    std::process::exit(1);
                }
            };

            println!();
            for sample in dataset.samples.iter().take(n) {
                // Use runtime.score() for single-text scoring
                match scoring_runtime.score(&sample.text) {
                    Ok(result) => {
                        let _ = stdout.execute(SetForegroundColor(Color::Green));
                        print!("  ✓ ");
                        let _ = stdout.execute(ResetColor);
                        println!(
                            "{} → Accept (score: {:.3}, expected: {:?})",
                            sample.id, result.score, sample.expected_decision
                        );
                    }
                    Err(_) => {
                        let _ = stdout.execute(SetForegroundColor(Color::Yellow));
                        print!("  ○ ");
                        let _ = stdout.execute(ResetColor);
                        println!(
                            "{} → Reject (expected: {:?})",
                            sample.id, sample.expected_decision
                        );
                    }
                }
            }
        }
    }
}
