//! `rarust test` — Test archive integrity.
//!
//! Verifies archive by decompressing each entry and checking CRC32.

use crate::cli::TestArgs;
use rarust_core::archive::{OpenOptions, PortableArchive};
use rarust_core::error::Result;

/// Execute the `test` command.
pub fn execute(args: &TestArgs, json: bool, quiet: bool) -> Result<()> {
    let options = OpenOptions {
        password: args.password.clone(),
        ..OpenOptions::default()
    };

    let archive = PortableArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;
    let summary = archive.test_all()?;

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
