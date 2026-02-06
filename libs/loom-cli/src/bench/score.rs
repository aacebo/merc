use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use loom::core::Format;
use loom::io::path::{FilePath, Path};
use loom::runtime::{bench, score::ScoreConfig};

use super::build_runtime;

pub async fn exec(path: &PathBuf, config_path: &PathBuf, output: &PathBuf, concurrency: usize) {
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
        "\nExtracting raw scores with {} parallel workers...\n",
        concurrency
    );

    let total = dataset.samples.len();
    let scorer = Arc::new(Mutex::new(scorer));
    let config = bench::AsyncRunConfig { concurrency };

    let export = bench::export_async_with_config(&dataset, scorer, config, |p| {
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
    })
    .await;

    // Clear the progress line
    print!("\r\x1B[K");
    println!("Extracted scores for {} samples", total);

    // Write to output file using runtime
    let output_path = Path::File(FilePath::from(output.clone()));
    if let Err(e) = runtime
        .save("file_system", &output_path, &export, Format::Json)
        .await
    {
        eprintln!("Error writing output file: {}", e);
        std::process::exit(1);
    }

    println!("Raw scores written to {:?}", output);
}
