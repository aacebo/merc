use std::fs;
use std::path::PathBuf;

use loom::cortex::bench::{BenchDataset, Category};

pub fn exec(path: &PathBuf) {
    println!("Analyzing coverage for {:?}...\n", path);

    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading dataset file: {}", e);
            std::process::exit(1);
        }
    };

    let dataset: BenchDataset = match serde_json::from_str(&contents) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing dataset: {}", e);
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
