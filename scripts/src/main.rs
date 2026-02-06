mod commands;
mod datasets;
mod sample;
mod widgets;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scripts", about = "Loom utility scripts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dataset management commands
    Datasets(commands::datasets::DatasetsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Datasets(args) => commands::datasets::run(args).await,
    }
}
