use std::io::{self, Write};
use std::path::PathBuf;

use loom::runtime::bench::BenchDataset;
use loom::runtime::score::ScoreConfig;

pub fn exec(path: &PathBuf, verbose: bool) {
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

    let config = ScoreConfig::default();

    println!("\nRunning benchmark...\n");

    let total = dataset.samples.len();
    let result = match loom::runtime::bench::run_with_config_and_progress(&dataset, config, |p| {
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
