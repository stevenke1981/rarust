//! `rarust benchmark` — Run compression benchmark.
//!
//! NOTE: Benchmark is not yet implemented.
//! This is a stub that returns an informative error.

use crate::cli::BenchmarkArgs;
use rarust_core::error::{RarustError, Result};

/// Execute the `benchmark` command.
pub fn execute(_args: &BenchmarkArgs) -> Result<()> {
    Err(RarustError::Unsupported(
        "Benchmark is not yet implemented. Planned for post-MVP.".to_string()
    ))
}
