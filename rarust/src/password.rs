//! Shared password resolution for CLI commands.

use std::path::Path;

use rarust_core::encryption::{Password, resolve_password};
use rarust_core::error::Result;

/// Resolve password from CLI flags + env + optional file/stdin.
pub fn resolve_cli_password(
    cli_arg: Option<String>,
    password_file: Option<&str>,
    from_stdin: bool,
) -> Result<Option<Password>> {
    let file = password_file.map(Path::new);
    resolve_password(cli_arg, file, from_stdin)
}
