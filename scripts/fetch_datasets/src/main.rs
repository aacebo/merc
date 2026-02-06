mod datasets;
mod sample;
mod widgets;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;

use datasets::build_client;
use sample::Dataset;
use widgets::{ProgressBar, Widget};

const DATASETS: [(&str, &str); 3] = [
    ("DailyDialog", "daily_dialog.samples.json"),
    ("Multi-Session Chat", "multi_session_chat.samples.json"),
    ("MSC-Self-Instruct", "msc_self_instruct.samples.json"),
];

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn save_dataset(dataset: &Dataset, path: &Path) -> Result<u64> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(dataset)?;
    fs::write(path, &json)?;

    Ok(json.len() as u64)
}

fn print_status(icon: char, color: Color, message: &str) {
    let mut stdout = std::io::stdout();
    let _ = stdout.execute(SetForegroundColor(color));
    print!("{} ", icon);
    let _ = stdout.execute(ResetColor);
    println!("{}", message);
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\nFetching datasets...\n");

    // Get output directory (relative to project root)
    let output_dir = Path::new("datasets");

    // Build HTTP client
    let client = build_client().context("Failed to build HTTP client")?;

    let mut results: Vec<(String, usize, u64, bool)> = Vec::new();

    for (idx, (name, filename)) in DATASETS.iter().enumerate() {
        // Show overall progress
        ProgressBar::new()
            .current(idx)
            .total(DATASETS.len())
            .message(format!("Downloading {}...", name))
            .render()
            .write();

        println!();
        println!("=== {} ===", name);

        let output_path = output_dir.join(filename);

        let result = match *name {
            "DailyDialog" => datasets::daily_dialog::download(&client).await,
            "Multi-Session Chat" => datasets::multi_session_chat::download(&client).await,
            "MSC-Self-Instruct" => datasets::msc_self_instruct::download(&client).await,
            _ => unreachable!(),
        };

        match result {
            Ok(dataset) => {
                let sample_count = dataset.samples.len();

                if sample_count > 0 {
                    match save_dataset(&dataset, &output_path) {
                        Ok(file_size) => {
                            print_status(
                                '✓',
                                Color::Green,
                                &format!(
                                    "Saved {} samples to {} ({})",
                                    sample_count,
                                    filename,
                                    format_bytes(file_size)
                                ),
                            );
                            results.push((name.to_string(), sample_count, file_size, true));
                        }
                        Err(e) => {
                            print_status(
                                '✗',
                                Color::Red,
                                &format!("Failed to save {}: {}", filename, e),
                            );
                            results.push((name.to_string(), sample_count, 0, false));
                        }
                    }
                } else {
                    print_status('✗', Color::Yellow, &format!("No samples found for {}", name));
                    results.push((name.to_string(), 0, 0, false));
                }
            }
            Err(e) => {
                print_status('✗', Color::Red, &format!("Failed to download {}: {}", name, e));
                results.push((name.to_string(), 0, 0, false));
            }
        }

        println!();
    }

    // Clear progress bar
    ProgressBar::clear();

    // Print summary
    println!("=== Summary ===\n");

    let mut total_samples = 0usize;
    let mut total_size = 0u64;
    let mut success_count = 0usize;

    for (name, samples, size, success) in &results {
        if *success {
            print_status(
                '✓',
                Color::Green,
                &format!("{:<20} {:>10} samples  {:>10}", name, samples, format_bytes(*size)),
            );
            total_samples += samples;
            total_size += size;
            success_count += 1;
        } else {
            print_status('✗', Color::Red, &format!("{:<20} failed", name));
        }
    }

    println!();
    println!(
        "Total: {} datasets, {} samples, {}",
        success_count,
        total_samples,
        format_bytes(total_size)
    );
    println!();

    if success_count < DATASETS.len() {
        std::process::exit(1);
    }

    Ok(())
}
