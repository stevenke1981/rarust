//! `rarust create` — Create a RAR archive.
//!
//! Builds RAR5 archives (store / compressed, optional AES-256 encryption,
//! solid mode, recovery records, and multi-volume splitting) via
//! `rarust_core::archive::ArchiveBuilder`.

use std::path::Path;

use walkdir::WalkDir;

use crate::cli::CreateArgs;
use crate::password::resolve_cli_password;
use rarust_core::archive::{ArchiveBuilder, ArchiveFormat, CompressionMethod};
use rarust_core::error::{RarustError, Result};

/// Execute the `create` command.
pub fn execute(args: &CreateArgs) -> Result<()> {
    let mut builder = ArchiveBuilder::new()
        .with_format(map_format(&args.format, Path::new(&args.archive))?)
        .with_method(map_method(&args.method))
        .solid(args.solid);

    let password = resolve_cli_password(
        args.password.clone(),
        args.password_file.as_deref(),
        args.password_stdin,
    )?;
    let has_password = password.is_some();
    if let Some(p) = password {
        builder = builder.with_password(p.to_string_lossy());
    }
    if let Some(v) = &args.volume {
        match parse_volume(v) {
            Some(bytes) => builder = builder.with_volume_size(bytes),
            None => {
                return Err(RarustError::Unsupported(format!(
                    "invalid volume size: {v} (expected e.g. 100m, 1g)"
                )));
            }
        }
    }
    if let Some(r) = args.recovery {
        builder = builder.with_recovery_percent(r);
    }
    if args.header_encrypt {
        if !has_password {
            return Err(RarustError::Unsupported(
                "header encryption requires a password (--password, --password-file, --password-stdin, or RARUST_PASSWORD)".to_string(),
            ));
        }
        builder = builder.with_header_encrypt(true);
    }

    // Expand inputs into (source, archive_name) pairs.
    let mut count = 0usize;
    for input in &args.inputs {
        let path = Path::new(input);
        if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let rel = entry.path().strip_prefix(path).unwrap_or(entry.path());
                    let archive_name = rel.to_string_lossy().replace('\\', "/");
                    builder = builder.add_file_as(entry.path(), archive_name);
                    count += 1;
                }
            }
        } else if path.is_file() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| input.clone());
            builder = builder.add_file_as(path, name);
            count += 1;
        } else {
            return Err(RarustError::Unsupported(format!(
                "input not found: {input}"
            )));
        }
    }

    if count == 0 {
        return Err(RarustError::Unsupported(
            "no input files to add (did you specify existing paths?)".to_string(),
        ));
    }

    if args.dry_run {
        let kind = match (has_password, args.header_encrypt) {
            (true, true) => "header-encrypted ",
            (true, false) => "encrypted ",
            _ => "",
        };
        let vol = args
            .volume
            .as_ref()
            .map(|v| format!(" (multi-volume, {v})"))
            .unwrap_or_default();
        println!(
            "[Dry Run] Would create {} with {} {}entries{}",
            args.archive, count, kind, vol
        );
        return Ok(());
    }

    let dest = Path::new(&args.archive);
    if dest.exists() && !args.force {
        return Err(RarustError::Unsupported(format!(
            "{} already exists; use --force to overwrite",
            args.archive
        )));
    }

    builder.build(dest)?;

    println!("Created {}", args.archive);
    Ok(())
}

/// Map the CLI archive format to the core `ArchiveFormat`.
fn map_format(format: &crate::cli::ArchiveFormatArg, archive: &Path) -> Result<ArchiveFormat> {
    match format {
        crate::cli::ArchiveFormatArg::Auto => match ArchiveFormat::from_path(archive) {
            Ok(format) => Ok(format),
            Err(_) if archive.extension().is_none() => Ok(ArchiveFormat::Rar5),
            Err(e) => Err(e),
        },
        crate::cli::ArchiveFormatArg::Rar5 => Ok(ArchiveFormat::Rar5),
        crate::cli::ArchiveFormatArg::Rar4 => Ok(ArchiveFormat::Rar4),
        crate::cli::ArchiveFormatArg::Zip => Ok(ArchiveFormat::Zip),
        crate::cli::ArchiveFormatArg::Tar => Ok(ArchiveFormat::Tar),
        crate::cli::ArchiveFormatArg::TarGz => Ok(ArchiveFormat::TarGz),
        crate::cli::ArchiveFormatArg::Gzip => Ok(ArchiveFormat::Gzip),
    }
}

/// Map the CLI `CompressionLevel` to the core `CompressionMethod`.
fn map_method(level: &crate::cli::CompressionLevel) -> CompressionMethod {
    match level {
        crate::cli::CompressionLevel::Store => CompressionMethod::Store,
        crate::cli::CompressionLevel::Fastest => CompressionMethod::Fastest,
        crate::cli::CompressionLevel::Fast => CompressionMethod::Fast,
        crate::cli::CompressionLevel::Normal => CompressionMethod::Normal,
        crate::cli::CompressionLevel::Good => CompressionMethod::Good,
        crate::cli::CompressionLevel::Best => CompressionMethod::Best,
    }
}

/// Parse a human volume size like `100m`, `1g`, `500k` into bytes.
fn parse_volume(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num, unit) = match s.chars().next_back() {
        Some(c) if c.is_ascii_alphabetic() => (&s[..s.len() - 1], c),
        _ => (s, 'b'),
    };
    let n: u64 = num.trim().parse().ok()?;
    let mult = match unit.to_ascii_lowercase() {
        'k' => 1024,
        'm' => 1024 * 1024,
        'g' => 1024 * 1024 * 1024,
        'b' | ' ' => 1,
        _ => return None,
    };
    Some(n * mult)
}
