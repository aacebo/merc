mod fetch;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DatasetsArgs {
    #[command(subcommand)]
    command: DatasetsCommands,
}

#[derive(Subcommand)]
enum DatasetsCommands {
    /// Fetch datasets from HuggingFace
    Fetch,
}

pub async fn run(args: DatasetsArgs) -> Result<()> {
    match args.command {
        DatasetsCommands::Fetch => fetch::run().await,
    }
}
