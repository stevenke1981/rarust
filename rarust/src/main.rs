//! `rarust` — command-line interface for RAR archives.

use std::process::ExitCode;

use clap::Parser;

use rarust::cli::{Cli, Commands};
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

fn run(cli: Cli) -> Result<()> {
    let json = cli.json;
    let no_progress = cli.no_progress;
    let quiet = cli.quiet;

    match &cli.command {
        Commands::List(args) => rarust::commands::list::execute(args, json),
        Commands::Extract(args) => rarust::commands::extract::execute(args, json, no_progress),
        Commands::Test(args) => {
            rarust::commands::test::execute(args, json, quiet || args.quiet, no_progress)
        }
        Commands::Create(args) => rarust::commands::create::execute(args),
        Commands::Repair(args) => rarust::commands::repair::execute(args),
        Commands::Benchmark(args) => rarust::commands::benchmark::execute(args),
        #[cfg(feature = "tui")]
        Commands::Tui(_args) => Err(rarust_core::error::RarustError::Unsupported(
            "TUI mode requires the `tui` feature".to_string(),
        )),
    }
}
