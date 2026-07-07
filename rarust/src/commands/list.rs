//! `rarust list` — List archive contents.
//!
//! Displays archive file listing with metadata in table, tree, JSON,
//! or name-only format.

use crate::cli::{ListArgs, SortField};
use rarust_core::archive::{OpenOptions, RarArchive};
use rarust_core::entry::Entry;
use rarust_core::error::Result;
use rarust_core::util;

/// Execute the `list` command.
pub fn execute(args: &ListArgs, json: bool) -> Result<()> {
    let options = OpenOptions {
        password: args.password.clone(),
        ..OpenOptions::default()
    };

    let archive = RarArchive::open_with_options(&args.archive, &options)?;
    let entries = archive.list()?;

    if entries.is_empty() {
        if json {
            println!("{{\"archive\": {}, \"entries\": [], \"total\": 0}}", 
                serde_json::to_string(&args.archive).unwrap());
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
fn print_table(archive_path: &str, entries: &[Entry], archive: &RarArchive) -> Result<()> {
    println!(" Archive: {}", archive_path);
    println!(" Format: {} | Entries: {}", 
        format_family(archive.family()),
        entries.len()
    );

    println!();
    println!(" {:<60} {:>10} {:>7} {:>19} {:>10} {:>10}", 
        "Name", "Size", "Ratio", "Modified", "CRC32", "Method");
    println!(" {}",
        "─".repeat(120));
    
    for entry in entries {
        let ratio = if entry.size > 0 {
            format!("{:.0}%", entry.ratio * 100.0)
        } else {
            "-".to_string()
        };
        let modified = entry.modified
            .map(util::format_dos_time)
            .unwrap_or_else(|| "-".to_string());
        let crc = entry.crc32
            .map(|c| format!("{:08X}", c))
            .unwrap_or_else(|| "-".to_string());
        let name = if entry.is_directory {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        println!(" {:<60} {:>10} {:>7} {:>19} {:>10} {:>10}",
            if name.len() > 58 { format!("{}...", &name[..58]) } else { name },
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
    println!(" {}",
        "─".repeat(120));
    println!(" {} files                     {:>10}  {:>3}%                        ",
        entries.iter().filter(|e| !e.is_directory).count(),
        util::format_size(total_size),
        overall_ratio,
    );

    Ok(())
}

/// Print archive listing as a tree (directory hierarchy).
fn print_tree(entries: &[Entry]) -> Result<()> {
    let mut last_dir = String::new();
    for entry in entries {
        if entry.is_directory {
            println!(" {}/", entry.name);
            last_dir = entry.name.clone();
        } else {
            #[allow(unused_variables)]
            let _prefix = if entry.name.starts_with(&last_dir) && !last_dir.is_empty() {
                // Placeholder for tree formatting — simplified for now
                ""
            } else {
                ""
            };
            println!(" {}", entry.name);
        }
    }
    Ok(())
}

/// Print archive listing as JSON.
fn print_json(archive_path: &str, entries: &[Entry], archive: &RarArchive) -> Result<()> {
    let output = serde_json::json!({
        "archive": archive_path,
        "format": format_family(archive.family()),
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
        SortField::Size => entries.sort_by(|a, b| a.size.cmp(&b.size)),
        SortField::Date => entries.sort_by(|a, b| a.modified.cmp(&b.modified)),
        SortField::Ratio => entries.sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap_or(std::cmp::Ordering::Equal)),
        SortField::Crc => entries.sort_by(|a, b| a.crc32.cmp(&b.crc32)),
    }
    if reverse {
        entries.reverse();
    }
}

/// Format archive family for display.
fn format_family(family: rarust_core::ArchiveFamily) -> &'static str {
    match family {
        rarust_core::ArchiveFamily::Rar13 => "RAR 1.3/1.4",
        rarust_core::ArchiveFamily::Rar15To40 => "RAR 1.5-4.0",
        rarust_core::ArchiveFamily::Rar50Plus => "RAR 5.0+",
        _ => "Unknown",
    }
}
