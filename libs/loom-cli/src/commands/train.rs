use std::io::stdout;
use std::path::PathBuf;

use crossterm::ExecutableCommand;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use loom::core::Format;
use loom::io::path::{FilePath, Path};
use loom::runtime::eval;

use super::build_runtime;
use crate::widgets::{self, Widget};

pub async fn exec(path: &PathBuf, output: &PathBuf, generate_rust: bool) {
    widgets::Spinner::new()
        .message(format!("Loading raw scores from {:?}...", path))
        .render()
        .write();

    let runtime = build_runtime();
    let file_path = Path::File(FilePath::from(path.clone()));

    let export: eval::RawScoreExport = match runtime.load("file_system", &file_path).await {
        Ok(e) => e,
        Err(e) => {
            widgets::Spinner::clear();
            eprintln!("Error loading file: {}", e);
            std::process::exit(1);
        }
    };

    widgets::Spinner::clear();
    println!("Loaded {} samples", export.samples.len());

    widgets::Spinner::new()
        .message("Training Platt parameters...")
        .render()
        .write();

    let result = eval::train_platt_params(&export);

    widgets::Spinner::clear();

    // Display results
    println!("=== Training Results ===\n");

    let mut stdout = stdout();
    let mut sorted_labels: Vec<_> = result.params.iter().collect();
    sorted_labels.sort_by_key(|(k, _)| k.as_str());

    for (label, params) in &sorted_labels {
        let stats = result.metadata.samples_per_label.get(*label);
        let (status, color) = if let Some(s) = stats {
            if s.skipped {
                (
                    format!("SKIPPED (pos={}, neg={})", s.positive, s.negative),
                    Color::Yellow,
                )
            } else {
                (
                    format!("pos={}, neg={}", s.positive, s.negative),
                    Color::Green,
                )
            }
        } else {
            ("".to_string(), Color::White)
        };

        print!("{:20} a={:7.4}, b={:7.4}  [", label, params.a, params.b);
        let _ = stdout.execute(SetForegroundColor(color));

        print!("{}", status);
        let _ = stdout.execute(ResetColor);

        println!("]");
    }

    // Write parameters to output file using runtime
    let output_path = Path::File(FilePath::from(output.clone()));
    if let Err(e) = runtime
        .save("file_system", &output_path, &result, Format::Json)
        .await
    {
        eprintln!("\nError writing output file: {}", e);
        std::process::exit(1);
    }

    let _ = stdout.execute(SetForegroundColor(Color::Green));
    print!("✓ ");

    let _ = stdout.execute(ResetColor);
    println!("Parameters written to {:?}", output);

    if generate_rust {
        let rust_code = eval::generate_rust_code(&result);
        println!("\n=== Rust Code ===\n");
        println!("{}", rust_code);
    }
}
