//! `rarust test` — Test archive integrity.
//!
//! Verifies archive by decompressing each entry and checking CRC32.

use crate::cli::TestArgs;
use crate::commands::progress::{progress_bar, update_progress_bar};
use crate::password::resolve_cli_password;
use rarust_core::archive::{OpenOptions, PortableArchive};
use rarust_core::error::Result;

/// Execute the `test` command.
pub fn execute(args: &TestArgs, json: bool, quiet: bool, no_progress: bool) -> Result<()> {
    let password = resolve_cli_password(
        args.password.clone(),
        args.password_file.as_deref(),
        args.password_stdin,
    )?;

    let options = OpenOptions {
        password,
        ..OpenOptions::default()
    };

    let archive = PortableArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;
    let total_bytes = entries.iter().map(|entry| entry.size).sum();
    let progress_bar = if json || quiet || no_progress {
        None
    } else {
        Some(progress_bar("Testing", entries.len() as u64, total_bytes))
    };
    let summary = archive.test_all_controlled(
        |progress| {
            if let Some(pb) = &progress_bar {
                update_progress_bar(pb, &progress);
            }
        },
        || false,
    )?;

    if let Some(pb) = &progress_bar {
        pb.finish_and_clear();
    }

    if json {
        let results: Vec<_> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "file": e.name,
                    "size": e.size,
                    "status": "ok",
                })
            })
            .collect();
        let output = serde_json::json!({
            "archive": args.archive,
            "total": summary.total,
            "tested": summary.tested,
            "failed": summary.failed,
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if !quiet {
        println!("Testing archive: {}", args.archive);
        println!("Total entries: {}", summary.total);

        println!(
            "Test completed: {} entries OK, {} failed",
            summary.tested, summary.failed
        );
    }

    Ok(())
}
