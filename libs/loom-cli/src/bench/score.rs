use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use loom::cortex::bench::{BenchDataset, export_with_progress};
use loom::runtime::score::ScoreConfig;

pub fn exec(path: &PathBuf, output: &PathBuf) {
    println!("Loading dataset from {:?}...", path);

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

    println!("Loaded {} samples", dataset.samples.len());
    println!("Building scorer (this may download model files on first run)...");

    let config = ScoreConfig::default();
    let layer = match config.build() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error building scorer: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nExtracting raw scores...\n");

    let total = dataset.samples.len();
    let export = export_with_progress(&dataset, &layer, |p| {
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
    });

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
