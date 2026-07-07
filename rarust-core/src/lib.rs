//! # rarust-core
//!
//! Core library for **Rarust** — a modern, pure Rust RAR command-line tool.
//!
//! This crate provides the high-level API that wraps the `rars` RAR backend
//! with rarust-specific convenience types, error handling, and output formatting.
//!
//! ## Architecture
//!
//! ```text
//! CLI Layer (rarust binary)       ← you are here (library layer)
//! ───────────────────────────────────────────
//! rarust-core (this crate)        ← high-level API wrapping rars
//! ───────────────────────────────────────────
//! rars crate                      ← RAR format parsing & decompression
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rarust_core::archive::RarArchive;
//!
//! let archive = RarArchive::open("backup.rar").unwrap();
//! let entries = archive.list().unwrap();
//! for entry in &entries {
//!     println!("{} ({} bytes)", entry.name, entry.size);
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

// Dependencies intended for future use — silence unused_crate_dependencies lint.
#[cfg(feature = "parallel")]
use rayon as _;
use dunce as _;
use serde_json as _;
use walkdir as _;

pub mod archive;
pub mod encryption;
pub mod entry;
pub mod error;
pub mod multi;
pub mod util;

// Re-export key types from rars for CLI convenience.
pub use rars::ArchiveFamily;
