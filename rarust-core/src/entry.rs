//! Archive entry types for rarust-core.
//!
//! `Entry` is the high-level metadata type exposed to CLI consumers.
//! It wraps `rars::ArchiveMemberMeta` to provide a stable, serializable view
//! with additional computed fields.

use serde::Serialize;
use std::path::Path;

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

    /// Return the local filesystem-safe path components for extraction.
    ///
    /// Rejects paths that would escape the destination directory (directory
    /// traversal prevention).
    pub fn safe_extract_path(&self, dest: &Path) -> Option<std::path::PathBuf> {
        let name = self.name.replace('\\', "/");
        // Reject absolute paths or paths with ".." components
        if name.starts_with('/') || name.contains("..") {
            return None;
        }
        let target = dest.join(&name);
        // Ensure the target is within the destination (canonicalization)
        // For now, a basic containment check; production should use
        // canonicalize + starts_with on the dest.
        if target.starts_with(dest) {
            Some(target)
        } else {
            None
        }
    }
}
