use clap::{Parser, Subcommand};

mod bench;

#[derive(Parser)]
#[command(name = "merc")]
#[command(about = "Merc scoring engine CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Benchmark operations
    Bench {
        #[command(subcommand)]
        action: bench::BenchAction,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bench { action } => bench::run(action),
    }
}
