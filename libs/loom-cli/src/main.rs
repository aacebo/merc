use clap::{Parser, Subcommand};

mod commands;
pub mod widgets;

use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "loom")]
#[command(about = "Loom scoring engine CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run evaluation against a dataset
    Run {
        /// Path to the dataset JSON file
        path: PathBuf,
        /// Path to config file (YAML/JSON/TOML)
        #[arg(short, long)]
        config: PathBuf,
        /// Show detailed per-category and per-label results
        #[arg(short, long)]
        verbose: bool,
        /// Number of parallel inference workers (overrides config)
        #[arg(long)]
        concurrency: Option<usize>,
        /// Batch size for ML inference (overrides config)
        #[arg(long)]
        batch_size: Option<usize>,
        /// Fail if samples have categories/labels not in config (overrides config)
        #[arg(long)]
        strict: Option<bool>,
    },
    /// Validate a dataset
    Validate {
        /// Path to the dataset JSON file
        path: PathBuf,
        /// Path to config file (YAML/JSON/TOML) for category/label validation
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Fail if samples have categories/labels not in config (default: report errors)
        #[arg(long)]
        strict: bool,
    },
    /// Extract raw scores for Platt calibration training
    Score {
        /// Path to the dataset JSON file
        path: PathBuf,
        /// Path to config file (YAML/JSON/TOML)
        #[arg(short, long)]
        config: PathBuf,
        /// Output path for results (overrides config)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Number of parallel inference workers (overrides config)
        #[arg(long)]
        concurrency: Option<usize>,
        /// Batch size for ML inference (overrides config)
        #[arg(long)]
        batch_size: Option<usize>,
        /// Fail if samples have categories/labels not in config (overrides config)
        #[arg(long)]
        strict: Option<bool>,
    },
    /// Train Platt calibration parameters from raw scores
    Train {
        /// Path to raw scores JSON (from score command)
        path: PathBuf,
        /// Output path for trained parameters JSON
        #[arg(short, long)]
        output: PathBuf,
        /// Also output Rust code for label.rs
        #[arg(long)]
        code: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            path,
            config,
            verbose,
            concurrency,
            batch_size,
            strict,
        } => commands::run::exec(&path, &config, verbose, concurrency, batch_size, strict).await,
        Commands::Validate {
            path,
            config,
            strict,
        } => commands::validate::exec(&path, config.as_ref(), strict).await,
        Commands::Score {
            path,
            config,
            output,
            concurrency,
            batch_size,
            strict,
        } => {
            commands::score::exec(
                &path,
                &config,
                output.as_ref(),
                concurrency,
                batch_size,
                strict,
            )
            .await
        }
        Commands::Train { path, output, code } => commands::train::exec(&path, &output, code).await,
    }
}
