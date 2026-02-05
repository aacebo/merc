use std::fs;
use std::path::PathBuf;

use loom::runtime::bench::RawScoreExport;

pub fn exec(path: &PathBuf, output: &PathBuf, generate_rust: bool) {
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

    let result = loom::runtime::bench::train_platt_params(&export);

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
        let rust_code = loom::runtime::bench::generate_rust_code(&result);
        println!("\n=== Rust Code ===\n");
        println!("{}", rust_code);
    }
}
