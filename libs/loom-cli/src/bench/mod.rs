use std::path::PathBuf;

use clap::Subcommand;

mod cov;
mod run;
mod score;
mod train;
mod validate;

#[derive(Subcommand)]
pub enum BenchAction {
    /// Run benchmark against a dataset
    Run {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Show detailed per-category and per-label results
        #[arg(short, long)]
        verbose: bool,
    },
    /// Validate a benchmark dataset
    Validate {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Show label coverage for a dataset
    Coverage {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
    },
    /// Extract raw scores for Platt calibration training
    Score {
        /// Path to the benchmark dataset JSON file
        path: PathBuf,
        /// Output path for raw scores JSON
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Train Platt calibration parameters from raw scores
    Train {
        /// Path to raw scores JSON (from extract-scores)
        path: PathBuf,
        /// Output path for trained parameters JSON
        #[arg(short, long)]
        output: PathBuf,
        /// Also output Rust code for label.rs
        #[arg(long)]
        code: bool,
    },
}

pub fn run(action: BenchAction) {
    match action {
        BenchAction::Run { path, verbose } => run::exec(&path, verbose),
        BenchAction::Validate { path } => validate::exec(&path),
        BenchAction::Coverage { path } => cov::exec(&path),
        BenchAction::Score { path, output } => score::exec(&path, &output),
        BenchAction::Train { path, output, code } => train::exec(&path, &output, code),
    }
}
