//! `rarust extract` — Extract files from a RAR archive.
//!
//! Supports full path preservation, flat extraction, file filtering,
//! overwrite modes, and progress reporting.

use std::path::Path;

use crate::cli::ExtractArgs;
use indicatif::{ProgressBar, ProgressStyle};
use rarust_core::archive::{ArchiveProgress, OpenOptions, PortableArchive};
use rarust_core::error::Result;

/// Execute the `extract` command.
pub fn execute(args: &ExtractArgs, json: bool, no_progress: bool) -> Result<()> {
    let dest = args
        .dest
        .as_deref()
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
        Some(progress_bar(matched as u64, total_bytes))
    };

    let summary = archive.extract_with_filter_controlled(
        dest,
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

fn progress_bar(total_entries: u64, total_bytes: u64) -> ProgressBar {
    let pb = if total_bytes > 0 {
        ProgressBar::new(total_bytes)
    } else {
        ProgressBar::new(total_entries)
    };
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("#>-");
    pb.set_style(style);
    pb.set_message("Extracting");
    pb
}

fn update_progress_bar(pb: &ProgressBar, progress: &ArchiveProgress) {
    if progress.total_bytes > 0 {
        pb.set_length(progress.total_bytes);
        pb.set_position(progress.processed_bytes.min(progress.total_bytes));
    } else if progress.total_entries > 0 {
        pb.set_length(progress.total_entries);
        pb.set_position(progress.completed_entries.min(progress.total_entries));
    }
    if !progress.current_entry.is_empty() {
        pb.set_message(progress.current_entry.clone());
    }
}

/// Preview what would be extracted without writing.
fn dry_run(archive_path: &str, dest: &Path, args: &ExtractArgs) -> Result<()> {
    let options = OpenOptions {
        password: args.password.clone(),
        ..OpenOptions::default()
    };

    let archive = PortableArchive::open_with_options(archive_path, &options)?;
    let entries = archive.list()?;

    println!(
        "[Dry Run] Would extract {} files to {}",
        entries.len(),
        dest.display()
    );
    for entry in entries.iter().take(20) {
        println!(
            "[Dry Run]   {} → {}/{}",
            entry.name,
            dest.display(),
            entry.name
        );
    }
    if entries.len() > 20 {
        println!("[Dry Run]   ... and {} more files", entries.len() - 20);
    }
    println!(
        "[Dry Run] Total: {} files • {}",
        entries.len(),
        rarust_core::util::format_size(entries.iter().map(|e| e.size).sum::<u64>())
    );
    Ok(())
}
