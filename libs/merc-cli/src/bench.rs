use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::Subcommand;
use merc_engine::bench::{BenchDataset, Category, RawScoreExport};
use merc_engine::score::ScoreOptions;

#[derive(Subcommand)]
pub enum BenchAction {
    /// Run benchmark against a dataset
    Run {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Show detailed per-category and per-label results
        #[arg(short, long)]
        verbose: bool,
    },
    /// Validate a benchmark dataset
    Validate {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Show label coverage for a dataset
    Coverage {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Extract raw scores for Platt calibration training
    Score {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Output path for raw scores JSON
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Train Platt calibration parameters from raw scores
    Train {
        /// Path to raw scores JSON (from extract-scores)
        path: PathBuf,
        /// Output path for trained parameters JSON
        #[arg(short, long)]
        output: PathBuf,
        /// Also output Rust code for label.rs
        #[arg(long)]
        code: bool,
    },
}

pub fn run(action: BenchAction) {
    match action {
        BenchAction::Run { path, verbose } => run_benchmark(&path, verbose),
        BenchAction::Validate { path } => validate_dataset(&path),
        BenchAction::Coverage { path } => show_coverage(&path),
        BenchAction::Score { path, output } => extract_scores(&path, &output),
        BenchAction::Train { path, output, code } => train_platt(&path, &output, code),
    }
}

fn run_benchmark(path: &PathBuf, verbose: bool) {
    println!("Loading dataset from {:?}...", path);

    let dataset = match BenchDataset::load(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} samples", dataset.samples.len());
    println!("Building scorer (this may download model files on first run)...");

    let options = ScoreOptions::new();

    println!("\nRunning benchmark...\n");

    let total = dataset.samples.len();
    let result = match merc_engine::bench::run_with_options_and_progress(&dataset, options, |p| {
        let pct = (p.current as f32 / p.total as f32 * 100.0) as usize;
        let bar_width = 30;
        let filled = pct * bar_width / 100;
        let empty = bar_width - filled;
        let status = if p.correct { "✓" } else { "✗" };
        print!(
            "\r[{}{}] {:3}% ({:3}/{:3}) {} {}\x1B[K",
            "█".repeat(filled),
            "░".repeat(empty),
            pct,
            p.current,
            p.total,
            status,
            p.sample_id
        );
        let _ = io::stdout().flush();
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\nError running benchmark: {}", e);
            std::process::exit(1);
        }
    };

    // Clear the progress line
    print!("\r\x1B[K");
    println!("Completed {} samples\n", total);

    // Display prominent score summary
    let score_out_of_100 = (result.accuracy * 100.0).round() as u32;
    println!("========================================");
    println!(
        "  SCORE: {}/100 ({:.1}%)",
        score_out_of_100,
        result.accuracy * 100.0
    );
    println!("========================================\n");

    println!("=== Benchmark Results ===\n");
    println!("Total samples: {}", result.total);
    println!(
        "Correct:       {} ({:.1}%)",
        result.correct,
        result.accuracy * 100.0
    );
    println!();
    println!("Precision: {:.3}", result.precision);
    println!("Recall:    {:.3}", result.recall);
    println!("F1 Score:  {:.3}", result.f1);

    if verbose {
        println!("\n=== Per-Category Results ===\n");
        let mut categories: Vec<_> = result.per_category.iter().collect();
        categories.sort_by_key(|(cat, _)| format!("{:?}", cat));
        for (category, cat_result) in categories {
            println!(
                "{:12} {:3}/{:3} ({:.1}%)",
                format!("{:?}", category),
                cat_result.correct,
                cat_result.total,
                cat_result.accuracy * 100.0
            );
        }

        println!("\n=== Per-Label Results ===\n");
        println!(
            "{:20} {:>6} {:>6} {:>6} {:>8} {:>8} {:>8}",
            "Label", "Expect", "Detect", "TP", "Prec", "Recall", "F1"
        );
        println!("{}", "-".repeat(74));

        let mut labels: Vec<_> = result.per_label.iter().collect();
        labels.sort_by_key(|(label, _)| label.as_str());
        for (label, label_result) in labels {
            if label_result.expected_count > 0 || label_result.detected_count > 0 {
                println!(
                    "{:20} {:>6} {:>6} {:>6} {:>8.3} {:>8.3} {:>8.3}",
                    label,
                    label_result.expected_count,
                    label_result.detected_count,
                    label_result.true_positives,
                    label_result.precision,
                    label_result.recall,
                    label_result.f1
                );
            }
        }

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

fn validate_dataset(path: &PathBuf) {
    println!("Validating dataset at {:?}...", path);

    let dataset = match BenchDataset::load(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    let errors = dataset.validate();

    if errors.is_empty() {
        println!("✓ Dataset is valid ({} samples)", dataset.samples.len());
    } else {
        println!("✗ Found {} validation error(s):\n", errors.len());
        for error in &errors {
            println!("  - {}", error);
        }
        std::process::exit(1);
    }
}

fn show_coverage(path: &PathBuf) {
    println!("Analyzing coverage for {:?}...\n", path);

    let dataset = match BenchDataset::load(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    let coverage = dataset.coverage();

    println!("=== Dataset Coverage ===\n");
    println!("Total samples: {}", coverage.total_samples);
    println!(
        "Accept: {}, Reject: {}",
        coverage.accept_count, coverage.reject_count
    );

    println!("\n=== By Category ===\n");
    let categories = [
        Category::Task,
        Category::Emotional,
        Category::Factual,
        Category::Preference,
        Category::Decision,
        Category::Phatic,
        Category::Ambiguous,
    ];
    for cat in categories {
        let count = coverage.samples_by_category.get(&cat).unwrap_or(&0);
        let target = 50;
        let status = if *count >= target { "✓" } else { "○" };
        println!(
            "  {} {:12} {:3}/{}",
            status,
            format!("{:?}", cat),
            count,
            target
        );
    }

    println!("\n=== By Label ===\n");
    let mut labels: Vec<_> = coverage.samples_by_label.iter().collect();
    labels.sort_by_key(|(label, _)| label.as_str());
    for (label, count) in labels {
        let status = if *count >= 3 { "✓" } else { "○" };
        println!("  {} {:20} {}", status, label, count);
    }

    if !coverage.missing_labels.is_empty() {
        println!(
            "\n=== Missing Labels ({}) ===\n",
            coverage.missing_labels.len()
        );
        for label in &coverage.missing_labels {
            println!("  ✗ {}", label);
        }
    }
}

fn extract_scores(path: &PathBuf, output: &PathBuf) {
    println!("Loading dataset from {:?}...", path);

    let dataset = match BenchDataset::load(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} samples", dataset.samples.len());
    println!("Building scorer (this may download model files on first run)...");

    let options = ScoreOptions::new();

    println!("\nExtracting raw scores...\n");

    let total = dataset.samples.len();
    let export = match merc_engine::bench::export_raw_scores_with_options(&dataset, options, |p| {
        let pct = (p.current as f32 / p.total as f32 * 100.0) as usize;
        let bar_width = 30;
        let filled = pct * bar_width / 100;
        let empty = bar_width - filled;
        print!(
            "\r[{}{}] {:3}% ({:3}/{:3}) {}\x1B[K",
            "█".repeat(filled),
            "░".repeat(empty),
            pct,
            p.current,
            p.total,
            p.sample_id
        );
        let _ = io::stdout().flush();
    }) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("\nError extracting scores: {}", e);
            std::process::exit(1);
        }
    };

    // Clear the progress line
    print!("\r\x1B[K");
    println!("Extracted scores for {} samples", total);

    // Write to output file
    let json = match serde_json::to_string_pretty(&export) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error serializing output: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = fs::write(output, json) {
        eprintln!("Error writing output file: {}", e);
        std::process::exit(1);
    }

    println!("Raw scores written to {:?}", output);
}

fn train_platt(path: &PathBuf, output: &PathBuf, generate_rust: bool) {
    println!("Loading raw scores from {:?}...", path);

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let export: RawScoreExport = match serde_json::from_str(&content) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} samples", export.samples.len());
    println!("\nTraining Platt parameters...");

    let result = merc_engine::bench::train_platt_params(&export);

    // Display results
    println!("\n=== Training Results ===\n");

    let mut sorted_labels: Vec<_> = result.params.iter().collect();
    sorted_labels.sort_by_key(|(k, _)| k.as_str());

    for (label, params) in &sorted_labels {
        let stats = result.metadata.samples_per_label.get(*label);
        let status = if let Some(s) = stats {
            if s.skipped {
                format!("SKIPPED (pos={}, neg={})", s.positive, s.negative)
            } else {
                format!("pos={}, neg={}", s.positive, s.negative)
            }
        } else {
            "".to_string()
        };
        println!(
            "{:20} a={:7.4}, b={:7.4}  [{}]",
            label, params.a, params.b, status
        );
    }

    // Write parameters to output file
    let json = match serde_json::to_string_pretty(&result) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("\nError serializing output: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = fs::write(output, json) {
        eprintln!("\nError writing output file: {}", e);
        std::process::exit(1);
    }

    println!("\nParameters written to {:?}", output);

    if generate_rust {
        let rust_code = merc_engine::bench::generate_rust_code(&result);
        println!("\n=== Rust Code ===\n");
        println!("{}", rust_code);
    }
}
