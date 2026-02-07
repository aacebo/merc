use std::path::PathBuf;

use clap::Args;
use loom::runtime::{FileSystemSource, JsonCodec, Runtime, TomlCodec, YamlCodec};

use super::load_config;

/// Classify a single text
#[derive(Debug, Args)]
pub struct ClassifyCommand {
    /// Text to classify
    pub text: String,

    /// Path to config file (YAML/JSON/TOML)
    #[arg(short, long)]
    pub config: PathBuf,
}

impl ClassifyCommand {
    pub fn exec(self) {
        let config_path = &self.config;

        let config = match load_config(config_path.to_str().unwrap_or_default()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                std::process::exit(1);
            }
        };

        // Build runtime with config (scorer building uses rust-bert which conflicts with tokio)
        let runtime = Runtime::new()
            .source(FileSystemSource::builder().build())
            .codec(JsonCodec::new())
            .codec(YamlCodec::new())
            .codec(TomlCodec::new())
            .config(config)
            .build();

        // Use runtime.score() which internally uses runtime.eval()
        match runtime.score(&self.text) {
            Ok(result) => {
                println!("Decision: Accept");
                println!("Score: {:.3}", result.score);
                println!("\nDetected labels:");
                for (label, score) in result.raw_scores() {
                    if score > 0.0 {
                        println!("  {}: {:.3}", label, score);
                    }
                }
            }
            Err(e) => {
                // Score below threshold means rejection
                println!("Decision: Reject");
                println!("Reason: {}", e);
            }
        }
    }
}
