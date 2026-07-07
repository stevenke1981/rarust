//! `rarust extract` — Extract files from a RAR archive.
//!
//! Supports full path preservation, flat extraction, file filtering,
//! overwrite modes, and progress reporting.

use std::path::Path;

use crate::cli::ExtractArgs;
use rarust_core::archive::{OpenOptions, RarArchive};
use rarust_core::error::Result;

/// Execute the `extract` command.
pub fn execute(args: &ExtractArgs, json: bool, _no_progress: bool) -> Result<()> {
    let dest = args.dest.as_deref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    let options = OpenOptions {
        password: args.password.clone(),
        keep_broken: args.keep_broken,
        ..OpenOptions::default()
    };

    if args.dry_run {
        return dry_run(&args.archive, dest, args);
    }

    let archive = RarArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;

    // Apply include/exclude filter to detect empty result early.
    let matched = entries.iter().filter(|e| entry_matches(e, args)).count();

    if matched == 0 {
        if json {
            println!("{{\"archive\": {}, \"extracted\": 0, \"skipped\": 0}}",
                serde_json::to_string(&args.archive).unwrap());
        } else {
            println!("No files matched for extraction.");
        }
        return Ok(());
    }

    // No progress display for now; indicatif integration pending
    let summary = archive.extract_with_filter(dest, |entry| entry_matches(entry, args))?;

    if json {
        let output = serde_json::json!({
            "archive": args.archive,
            "destination": dest.to_string_lossy(),
            "extracted": summary.extracted,
            "skipped": summary.skipped,
            "errors": summary.errors,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Extracted {} files to {}", summary.extracted, dest.display());
        if summary.errors > 0 {
            eprintln!("[WARN] {} errors encountered", summary.errors);
        }
    }

    Ok(())
}

/// Return true if an entry passes the include/exclude filters.
fn entry_matches(entry: &rarust_core::entry::Entry, args: &ExtractArgs) -> bool {
    if args
        .include
        .as_ref()
        .is_some_and(|include| !entry.name.contains(include))
    {
        return false;
    }
    if args
        .exclude
        .as_ref()
        .is_some_and(|exclude| entry.name.contains(exclude))
    {
        return false;
    }
    true
}

/// Preview what would be extracted without writing.
fn dry_run(archive_path: &str, dest: &Path, args: &ExtractArgs) -> Result<()> {
    let options = OpenOptions {
        password: args.password.clone(),
        ..OpenOptions::default()
    };

    let archive = RarArchive::open_with_options(archive_path, &options)?;
    let entries = archive.list()?;

    println!("[Dry Run] Would extract {} files to {}", entries.len(), dest.display());
    for entry in entries.iter().take(20) {
        println!("[Dry Run]   {} → {}/{}", entry.name, dest.display(), entry.name);
    }
    if entries.len() > 20 {
        println!("[Dry Run]   ... and {} more files", entries.len() - 20);
    }
    println!("[Dry Run] Total: {} files • {}", 
        entries.len(),
        rarust_core::util::format_size(entries.iter().map(|e| e.size).sum::<u64>())
    );
    Ok(())
}
