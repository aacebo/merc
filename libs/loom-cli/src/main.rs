use clap::{Parser, Subcommand};

mod commands;
pub mod widgets;

use commands::{ClassifyCommand, RunCommand, ScoreCommand, TrainCommand, ValidateCommand};

/// Loom scoring engine CLI
///
/// Evaluate, validate, and train ML-based content scoring models.
#[derive(Parser)]
#[command(name = "loom")]
#[command(version, author)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Classify a single text
    Classify(ClassifyCommand),

    /// Run evaluation against a dataset
    Run(RunCommand),

    /// Validate a dataset
    Validate(ValidateCommand),

    /// Extract raw scores for Platt calibration training
    Score(ScoreCommand),

    /// Train Platt calibration parameters from raw scores
    Train(TrainCommand),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Classify(cmd) => cmd.exec(),
        Commands::Run(cmd) => cmd.exec().await,
        Commands::Validate(cmd) => cmd.exec().await,
        Commands::Score(cmd) => cmd.exec().await,
        Commands::Train(cmd) => cmd.exec().await,
    }
}
