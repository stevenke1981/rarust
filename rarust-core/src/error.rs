//! Error types for rarust-core.
//!
//! Defines `RarustError` — a thiserror enum covering all failure modes
//! that the library and CLI can encounter.

use thiserror::Error;

/// Unified error type for rarust operations.
#[derive(Error, Debug)]
pub enum RarustError {
    /// I/O error from the filesystem layer.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The file is not a valid RAR archive or uses an unsupported format.
    #[error("Format error: {0}")]
    Format(String),

    /// CRC32 / BLAKE2 checksum mismatch after decompression.
    #[error("CRC mismatch for `{path}`: expected {expected:#010X}, actual {actual:#010X}")]
    CrcMismatch {
        /// Archive member path that failed verification.
        path: String,
        /// Expected CRC32 checksum stored in the archive.
        expected: u32,
        /// Actual CRC32 checksum computed during extraction.
        actual: u32,
    },

    /// Wrong password or corrupted encryption header.
    #[error("Wrong password or corrupted encryption header")]
    WrongPassword,

    /// A requested feature is not yet supported.
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Archive data is corrupt or inconsistent.
    #[error("Corrupt archive: {0}")]
    Corrupt(String),

    /// A required multi-volume part is missing.
    #[error("Volume missing: {path} not found (part {number})")]
    VolumeMissing {
        /// Expected filesystem path of the missing volume.
        path: String,
        /// Volume part number that could not be located.
        number: u32,
    },

    /// Decompression dictionary exceeds the configured memory limit.
    #[error("Memory limit exceeded: dictionary too large ({size} bytes)")]
    MemoryLimit {
        /// Required dictionary or buffered decode size in bytes.
        size: u64,
    },

    /// No entries matched the given filter pattern.
    #[error("No files found matching the given pattern")]
    NoFilesFound,

    /// Wrapper for underlying `rars` crate errors.
    #[error("RAR backend error: {0}")]
    Rars(#[from] rars::error::Error),

    /// Anyhow-compatible catch-all for dynamic errors.
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Convenience alias for `Result<T, RarustError>`.
pub type Result<T> = std::result::Result<T, RarustError>;

/// Convert an `anyhow::Error` into a `RarustError`.
impl From<String> for RarustError {
    fn from(msg: String) -> Self {
        RarustError::Format(msg)
    }
}

impl From<&str> for RarustError {
    fn from(msg: &str) -> Self {
        RarustError::Format(msg.to_string())
    }
}
