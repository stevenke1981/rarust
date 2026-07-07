//! # rarust — A modern, pure Rust RAR command-line tool.
//!
//! Rarust is a cross-platform CLI tool for creating, extracting, listing,
//! and testing RAR archives (RAR4/RAR5 support) with modern output features
//! including progress bars, JSON output, and colored terminal display.

use std::process::ExitCode;

use clap::Parser;

mod cli;
mod commands;

use cli::{Cli, Commands};
use rarust_core::error::Result;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(2)
        }
    }
}

/// Run the appropriate command based on CLI arguments.
fn run(cli: Cli) -> Result<()> {
    let json = cli.json;
    let no_progress = cli.no_progress;
    let quiet = cli.quiet;

    match &cli.command {
        Commands::List(args) => commands::list::execute(args, json),
        Commands::Extract(args) => commands::extract::execute(args, json, no_progress),
        Commands::Test(args) => commands::test::execute(args, json, quiet),
        Commands::Create(args) => commands::create::execute(args),
        Commands::Repair(args) => commands::repair::execute(args),
        Commands::Benchmark(args) => commands::benchmark::execute(args),
        #[cfg(feature = "tui")]
        Commands::Tui(_args) => Err(rarust_core::error::RarustError::Unsupported(
            "TUI mode requires the `tui` feature".to_string(),
        )),
    }
}
