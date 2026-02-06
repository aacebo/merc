use std::path::PathBuf;

use loom::io::path::{FilePath, Path};
use loom::runtime::bench::{self, Category};

use super::build_runtime;

pub async fn exec(path: &PathBuf) {
    println!("Analyzing coverage for {:?}...\n", path);

    let runtime = build_runtime();
    let file_path = Path::File(FilePath::from(path.clone()));

    let dataset: bench::BenchDataset = match runtime.load("file_system", &file_path).await {
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
