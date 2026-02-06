use std::io::stdout;
use std::path::PathBuf;

use crossterm::ExecutableCommand;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use loom::core::path::IdentPath;
use loom::io::path::{FilePath, Path};
use loom::runtime::{ScoreConfig, eval};

use super::{build_runtime, load_config};
use crate::widgets::{self, Widget};

pub async fn exec(path: &PathBuf, config_path: Option<&PathBuf>, strict: bool) {
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
        let score_path = IdentPath::parse("layers.score").expect("valid path");
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

    let errors = dataset.validate_with_config(valid_categories.as_deref(), valid_labels.as_deref());
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
}
