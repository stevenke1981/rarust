//! Archive entry types for rarust-core.
//!
//! `Entry` is the high-level metadata type exposed to CLI consumers.
//! It wraps `rars::ArchiveMemberMeta` to provide a stable, serializable view
//! with additional computed fields.

use serde::Serialize;
use std::path::{Component, Path, PathBuf};

/// High-level metadata for a single archive member (file or directory).
#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    /// Filename as stored in the archive (forward-slash separated).
    pub name: String,

    /// Raw filename bytes (for non-UTF-8 archives).
    #[serde(skip_serializing)]
    pub name_raw: Vec<u8>,

    /// Uncompressed size in bytes.
    pub size: u64,

    /// Compressed (packed) size in bytes.
    pub compressed_size: u64,

    /// Compression ratio (compressed_size / size), 0.0 if size is 0.
    pub ratio: f64,

    /// Whether this entry is a directory.
    pub is_directory: bool,

    /// Whether the entry payload is encrypted.
    pub is_encrypted: bool,

    /// Whether the entry is stored without compression.
    pub is_stored: bool,

    /// Whether this entry continues from a previous volume.
    pub is_split_before: bool,
    /// Whether this entry continues into the next volume.
    pub is_split_after: bool,

    /// DOS-style file timestamp (seconds since 1980-01-01 00:00:00).
    pub modified: Option<u32>,

    /// CRC32 checksum, if available from the archive.
    pub crc32: Option<u32>,

    /// Compression method string (e.g. "m3 normal", "m0 store", "m5 best").
    pub method: String,
}

impl Entry {
    /// Build an `Entry` from a `rars::ArchiveMemberMeta` (full metadata).
    pub fn from_rars_meta(meta: &rars::ArchiveMemberMeta) -> Self {
        let name = String::from_utf8_lossy(&meta.name).to_string();
        let size = meta.unpacked_size;
        let compressed_size = meta.packed_size;
        let ratio = if size > 0 {
            compressed_size as f64 / size as f64
        } else {
            0.0
        };

        let method = if meta.is_stored {
            "m0 store".to_string()
        } else {
            // rars doesn't expose per-member compression level in meta directly,
            // so we derive a reasonable label from family + stored flag.
            match meta.family {
                rars::ArchiveFamily::Rar50Plus => "m3 normal".to_string(),
                rars::ArchiveFamily::Rar15To40 => "m3 normal".to_string(),
                rars::ArchiveFamily::Rar13 => "legacy".to_string(),
                _ => "unknown".to_string(),
            }
        };

        Entry {
            name,
            name_raw: meta.name.clone(),
            size,
            compressed_size,
            ratio,
            is_directory: meta.is_directory,
            is_encrypted: meta.is_encrypted,
            is_stored: meta.is_stored,
            is_split_before: meta.is_split_before,
            is_split_after: meta.is_split_after,
            modified: meta.file_time,
            crc32: None, // rars doesn't expose CRC in ArchiveMemberMeta
            method,
        }
    }

    /// Build an `Entry` from a `rars::ExtractedEntryMeta` (limited fields available
    /// during streaming extraction).
    pub fn from_extracted_meta(meta: &rars::ExtractedEntryMeta) -> Self {
        let name = String::from_utf8_lossy(meta.name_bytes()).to_string();
        Entry {
            name,
            name_raw: meta.name.clone(),
            size: 0, // not available in ExtractedEntryMeta
            compressed_size: 0,
            ratio: 0.0,
            is_directory: meta.is_directory,
            is_encrypted: false, // not available during extraction
            is_stored: false,
            is_split_before: false,
            is_split_after: false,
            modified: Some(meta.file_time),
            crc32: None,
            method: String::new(),
        }
    }

    /// Return the local filesystem-safe path for extraction under `dest`.
    ///
    /// Rejects absolute paths, drive letters, UNC roots, and `..` components so
    /// archive members cannot escape the destination directory.
    pub fn safe_extract_path(&self, dest: &Path) -> Option<PathBuf> {
        safe_join_under(dest, &self.name, false)
    }

    /// Like [`safe_extract_path`], but only uses the final path segment (flat extract).
    pub fn safe_extract_path_flat(&self, dest: &Path) -> Option<PathBuf> {
        safe_join_under(dest, &self.name, true)
    }
}

/// Join an archive member name under `dest` after sanitizing path components.
///
/// When `flat` is true, only the final non-empty segment is used as the filename.
pub fn safe_join_under(dest: &Path, archive_name: &str, flat: bool) -> Option<PathBuf> {
    let normalized = archive_name.replace('\\', "/");
    if normalized.is_empty() || normalized.contains('\0') {
        return None;
    }

    // Absolute POSIX / UNC-style / rooted paths
    if normalized.starts_with('/') {
        return None;
    }

    // Windows drive-absolute: "C:/...", "c:foo" is also rejected for safety
    let bytes = normalized.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return None;
    }

    let segments: Vec<&str> = normalized
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();

    if segments.is_empty() {
        return None;
    }

    if segments.contains(&"..") {
        return None;
    }

    // Reject Windows reserved device names in any segment (CON, NUL, COM1, …)
    if segments.iter().any(|s| is_windows_reserved_name(s)) {
        return None;
    }

    let chosen: &[&str] = if flat {
        // Use last segment only (basename)
        segments.last().map(std::slice::from_ref).unwrap_or(&[])
    } else {
        &segments
    };

    if chosen.is_empty() {
        return None;
    }

    let mut out = PathBuf::new();
    for seg in chosen {
        // Use Component::Normal only — never Prefix/RootDir/ParentDir
        let p = Path::new(seg);
        let mut normals = p.components().filter_map(|c| match c {
            Component::Normal(os) => Some(os),
            _ => None,
        });
        let first = normals.next()?;
        // Segment must be a single normal component (no internal separators left)
        if normals.next().is_some() {
            return None;
        }
        out.push(first);
    }

    let target = dest.join(&out);

    // Structural containment: every component of `out` is relative, so joining
    // under dest cannot escape via `..`. Extra check for absolute `out` (paranoia).
    if out.is_absolute() {
        return None;
    }

    Some(target)
}

fn is_windows_reserved_name(name: &str) -> bool {
    // Strip trailing dots/spaces which Windows also treats specially
    let base = name.trim_end_matches(['.', ' ']);
    let stem = base.split('.').next().unwrap_or(base);
    let upper = stem.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_rejects_parent_dir() {
        let dest = Path::new("/tmp/out");
        assert!(safe_join_under(dest, "../evil.txt", false).is_none());
        assert!(safe_join_under(dest, "a/../../evil.txt", false).is_none());
        assert!(safe_join_under(dest, "ok/../evil.txt", false).is_none());
    }

    #[test]
    fn safe_join_rejects_absolute_and_drive() {
        let dest = Path::new("/tmp/out");
        assert!(safe_join_under(dest, "/etc/passwd", false).is_none());
        assert!(safe_join_under(dest, "C:/Windows/system32", false).is_none());
        assert!(safe_join_under(dest, "c:\\Windows\\system32", false).is_none());
    }

    #[test]
    fn safe_join_allows_nested_relative() {
        let dest = Path::new("/tmp/out");
        let p = safe_join_under(dest, "nested/world.txt", false).expect("ok");
        assert_eq!(p, dest.join("nested").join("world.txt"));
    }

    #[test]
    fn safe_join_flat_uses_basename() {
        let dest = Path::new("/tmp/out");
        let p = safe_join_under(dest, "nested/world.txt", true).expect("ok");
        assert_eq!(p, dest.join("world.txt"));
    }

    #[test]
    fn safe_join_rejects_reserved_device_names() {
        let dest = Path::new("/tmp/out");
        assert!(safe_join_under(dest, "NUL", false).is_none());
        assert!(safe_join_under(dest, "dir/CON.txt", false).is_none());
    }

    #[test]
    fn safe_join_normalizes_backslashes() {
        let dest = Path::new("/tmp/out");
        let p = safe_join_under(dest, "a\\b\\c.txt", false).expect("ok");
        assert_eq!(p, dest.join("a").join("b").join("c.txt"));
    }
}
