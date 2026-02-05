use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use loom::runtime::bench::BenchDataset;
use loom::runtime::score::ScoreConfig;

pub fn exec(path: &PathBuf, output: &PathBuf) {
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

    println!("\nExtracting raw scores...\n");

    let total = dataset.samples.len();
    let export = match loom::runtime::bench::export_raw_scores_with_config(&dataset, config, |p| {
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
