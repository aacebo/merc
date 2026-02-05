use std::path::PathBuf;

use loom::runtime::bench::BenchDataset;

pub fn exec(path: &PathBuf) {
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
