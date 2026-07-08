//! `rarust list` — List archive contents.
//!
//! Displays archive file listing with metadata in table, tree, JSON,
//! or name-only format.

use crate::cli::{ListArgs, SortField};
use rarust_core::archive::{OpenOptions, PortableArchive};
use rarust_core::entry::Entry;
use rarust_core::error::Result;
use rarust_core::util;

/// Execute the `list` command.
pub fn execute(args: &ListArgs, json: bool) -> Result<()> {
    let options = OpenOptions {
        password: args.password.clone(),
        ..OpenOptions::default()
    };

    let archive = PortableArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;

    if entries.is_empty() {
        if json {
            println!(
                "{{\"archive\": {}, \"entries\": [], \"total\": 0}}",
                serde_json::to_string(&args.archive).unwrap()
            );
        } else {
            println!("Archive: {}", args.archive);
            println!("(empty archive)");
        }
        return Ok(());
    }

    // Sort entries
    let mut entries = entries;
    sort_entries(&mut entries, &args.sort, args.reverse);

    if json {
        print_json(&args.archive, &entries, &archive)?;
    } else if args.name_only {
        for e in &entries {
            println!("{}", e.name);
        }
    } else if args.tree {
        print_tree(&entries)?;
    } else {
        print_table(&args.archive, &entries, &archive)?;
    }

    Ok(())
}

/// Print archive listing as a formatted table (human-readable).
fn print_table(archive_path: &str, entries: &[Entry], archive: &PortableArchive) -> Result<()> {
    println!(" Archive: {}", archive_path);
    println!(
        " Format: {} | Entries: {}",
        archive.format_name(),
        entries.len()
    );

    println!();
    println!(
        " {:<60} {:>10} {:>7} {:>19} {:>10} {:>10}",
        "Name", "Size", "Ratio", "Modified", "CRC32", "Method"
    );
    println!(" {}", "─".repeat(120));

    for entry in entries {
        let ratio = if entry.size > 0 {
            format!("{:.0}%", entry.ratio * 100.0)
        } else {
            "-".to_string()
        };
        let modified = entry
            .modified
            .map(util::format_dos_time)
            .unwrap_or_else(|| "-".to_string());
        let crc = entry
            .crc32
            .map(|c| format!("{:08X}", c))
            .unwrap_or_else(|| "-".to_string());
        let name = if entry.is_directory {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        println!(
            " {:<60} {:>10} {:>7} {:>19} {:>10} {:>10}",
            if name.len() > 58 {
                format!("{}...", &name[..58])
            } else {
                name
            },
            util::format_size(entry.size),
            ratio,
            modified,
            crc,
            entry.method,
        );
    }

    // Summary line
    let total_size: u64 = entries.iter().map(|e| e.size).sum();
    let total_compressed: u64 = entries.iter().map(|e| e.compressed_size).sum();
    let overall_ratio = if total_size > 0 {
        (total_compressed as f64 / total_size as f64 * 100.0) as u64
    } else {
        0
    };
    println!(" {}", "─".repeat(120));
    println!(
        " {} files                     {:>10}  {:>3}%                        ",
        entries.iter().filter(|e| !e.is_directory).count(),
        util::format_size(total_size),
        overall_ratio,
    );

    Ok(())
}

/// Print archive listing as a tree (directory hierarchy).
fn print_tree(entries: &[Entry]) -> Result<()> {
    let mut printed_dirs = std::collections::BTreeSet::new();

    for entry in entries {
        let normalized = entry.name.replace('\\', "/");
        let parts: Vec<_> = normalized
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .collect();
        if parts.is_empty() {
            continue;
        }

        let dir_count = if entry.is_directory {
            parts.len()
        } else {
            parts.len().saturating_sub(1)
        };

        for depth in 0..dir_count {
            let key = parts[..=depth].join("/");
            if printed_dirs.insert(key) {
                println!("{}{}/", "  ".repeat(depth), parts[depth]);
            }
        }

        if !entry.is_directory {
            let depth = parts.len().saturating_sub(1);
            println!("{}{}", "  ".repeat(depth), parts[depth]);
        }
    }
    Ok(())
}

/// Print archive listing as JSON.
fn print_json(archive_path: &str, entries: &[Entry], archive: &PortableArchive) -> Result<()> {
    let output = serde_json::json!({
        "archive": archive_path,
        "format": archive.format_name(),
        "entries": entries,
        "summary": {
            "total_files": entries.iter().filter(|e| !e.is_directory).count(),
            "total_size": entries.iter().map(|e| e.size).sum::<u64>(),
        }
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}

/// Sort entries by the specified field.
fn sort_entries(entries: &mut [Entry], field: &SortField, reverse: bool) {
    match field {
        SortField::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        SortField::Size => entries.sort_by_key(|entry| entry.size),
        SortField::Date => entries.sort_by_key(|entry| entry.modified),
        SortField::Ratio => entries.sort_by(|a, b| {
            a.ratio
                .partial_cmp(&b.ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortField::Crc => entries.sort_by_key(|entry| entry.crc32),
    }
    if reverse {
        entries.reverse();
    }
}
