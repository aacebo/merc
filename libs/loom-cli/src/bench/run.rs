use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use loom::io::path::{FilePath, Path};
use loom::runtime::{bench, score::ScoreConfig};

use super::build_runtime;

pub async fn exec(path: &PathBuf, config_path: &PathBuf, verbose: bool, concurrency: usize) {
    println!("Loading dataset from {:?}...", path);

    let runtime = build_runtime();
    let file_path = Path::File(FilePath::from(path.clone()));

    let dataset: bench::BenchDataset = match runtime.load("file_system", &file_path).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading dataset: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} samples", dataset.samples.len());
    println!("Loading config from {:?}...", config_path);

    let config_file_path = Path::File(FilePath::from(config_path.clone()));
    let config: ScoreConfig = match runtime.load("file_system", &config_file_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    println!("Building scorer (this may download model files on first run)...");

    // Build scorer in blocking task to avoid tokio runtime conflict with rust-bert
    let scorer = match tokio::task::spawn_blocking(move || config.build())
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
        "\nRunning benchmark with {} parallel workers...\n",
        concurrency
    );

    let total = dataset.samples.len();
    let scorer = Arc::new(Mutex::new(scorer));
    let config = bench::AsyncRunConfig { concurrency };

    let result = bench::run_async_with_config(&dataset, scorer, config, |p| {
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
    })
    .await;

    // Clear the progress line
    print!("\r\x1B[K");
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
        categories.sort_by_key(|(cat, _)| format!("{:?}", cat));

        for (category, cat_result) in categories {
            let cat_metrics = metrics.per_category.get(category);
            let accuracy = cat_metrics.map(|m| m.accuracy).unwrap_or(0.0);
            println!(
                "{:12} {:3}/{:3} ({:.1}%)",
                format!("{:?}", category),
                cat_result.correct,
                cat_result.total,
                accuracy * 100.0
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
                let label_metrics = metrics.per_label.get(label);
                let (precision, recall, f1) = label_metrics
                    .map(|m| (m.precision, m.recall, m.f1))
                    .unwrap_or((0.0, 0.0, 0.0));
                println!(
                    "{:20} {:>6} {:>6} {:>6} {:>8.3} {:>8.3} {:>8.3}",
                    label,
                    label_result.expected_count,
                    label_result.detected_count,
                    label_result.true_positives,
                    precision,
                    recall,
                    f1
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
