use std::fs;
use std::path::PathBuf;

use loom::cortex::bench::BenchDataset;

pub fn exec(path: &PathBuf) {
    println!("Validating dataset at {:?}...", path);

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
