//! `rarust extract` — Extract files from a RAR archive.
//!
//! Supports full path preservation, flat extraction, file filtering,
//! overwrite modes, and progress reporting.

use std::path::Path;

use crate::cli::{ExtractArgs, OverwriteMode};
use crate::commands::progress::{progress_bar, update_progress_bar};
use crate::password::resolve_cli_password;
use rarust_core::archive::{ExtractOptions, OpenOptions, OverwritePolicy, PortableArchive};
use rarust_core::error::{RarustError, Result};

/// Execute the `extract` command.
pub fn execute(args: &ExtractArgs, json: bool, no_progress: bool) -> Result<()> {
    let dest = args
        .dest
        .as_deref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    let password = resolve_cli_password(
        args.password.clone(),
        args.password_file.as_deref(),
        args.password_stdin,
    )?;

    let options = OpenOptions {
        password,
        keep_broken: args.keep_broken,
        ..OpenOptions::default()
    };

    if args.dry_run {
        return dry_run(&args.archive, dest, args, &options);
    }

    let archive = PortableArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;

    // Apply include/exclude filter to detect empty result early.
    let matched = entries.iter().filter(|e| entry_matches(e, args)).count();

    if matched == 0 {
        if json {
            println!(
                "{{\"archive\": {}, \"extracted\": 0, \"skipped\": 0}}",
                serde_json::to_string(&args.archive).unwrap()
            );
        } else {
            println!("No files matched for extraction.");
        }
        return Ok(());
    }

    let total_bytes = entries
        .iter()
        .filter(|entry| entry_matches(entry, args))
        .map(|entry| entry.size)
        .sum();
    let progress_bar = if json || no_progress {
        None
    } else {
        Some(progress_bar("Extracting", matched as u64, total_bytes))
    };

    let extract_opts = ExtractOptions {
        flat: args.flat,
        overwrite: map_overwrite(&args.overwrite),
    };

    let summary = archive.extract_with_options_controlled(
        dest,
        &extract_opts,
        |entry| entry_matches(entry, args),
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
        let output = serde_json::json!({
            "archive": args.archive,
            "destination": dest.to_string_lossy(),
            "extracted": summary.extracted,
            "skipped": summary.skipped,
            "errors": summary.errors,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!(
            "Extracted {} files to {}",
            summary.extracted,
            dest.display()
        );
        if summary.skipped > 0 {
            println!("Skipped {} entries", summary.skipped);
        }
        if summary.errors > 0 {
            eprintln!("[WARN] {} errors encountered", summary.errors);
        }
    }

    if summary.errors > 0 {
        return Err(RarustError::Corrupt(format!(
            "{} extraction error(s)",
            summary.errors
        )));
    }

    Ok(())
}

fn map_overwrite(mode: &OverwriteMode) -> OverwritePolicy {
    match mode {
        OverwriteMode::Overwrite => OverwritePolicy::Overwrite,
        OverwriteMode::Skip => OverwritePolicy::Skip,
        OverwriteMode::Rename => OverwritePolicy::Rename,
        // Non-interactive default: skip rather than block on prompts.
        OverwriteMode::Ask => OverwritePolicy::Skip,
    }
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
fn dry_run(
    archive_path: &str,
    dest: &Path,
    args: &ExtractArgs,
    options: &OpenOptions,
) -> Result<()> {
    let archive = PortableArchive::open_with_options(archive_path, options)?;
    let entries = archive.list()?;
    let matched: Vec<_> = entries.iter().filter(|e| entry_matches(e, args)).collect();

    println!(
        "[Dry Run] Would extract {} files to {}",
        matched.len(),
        dest.display()
    );
    for entry in matched.iter().take(20) {
        let rel = if args.flat {
            entry
                .name
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(entry.name.as_str())
        } else {
            entry.name.as_str()
        };
        println!("[Dry Run]   {} → {}/{}", entry.name, dest.display(), rel);
    }
    if matched.len() > 20 {
        println!("[Dry Run]   ... and {} more files", matched.len() - 20);
    }
    println!(
        "[Dry Run] Total: {} files • {}",
        matched.len(),
        rarust_core::util::format_size(matched.iter().map(|e| e.size).sum::<u64>())
    );
    Ok(())
}
