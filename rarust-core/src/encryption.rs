//! Password handling utilities for rarust-core.
//!
//! Provides secure password sourcing and zeroize-on-drop wrappers.

use zeroize::ZeroizeOnDrop;

/// A password container that zeroizes on drop.
#[derive(Clone, ZeroizeOnDrop)]
pub struct Password(Vec<u8>);

impl Password {
    /// Create a new password from bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Password(bytes)
    }

    /// Create a new password from a string.
    pub fn from_string(s: String) -> Self {
        Password(s.into_bytes())
    }

    /// Return the password as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Password source priority:
/// 1. `--password` CLI arg (with warning about process visibility)
/// 2. `RARUST_PASSWORD` env var
/// 3. `--password-file <PATH>` (first line)
/// 4. `--password-stdin`
/// 5. Interactive TTY prompt
#[derive(Debug, Clone)]
pub enum PasswordSource {
    /// Directly from CLI argument.
    CliArg,
    /// From environment variable.
    EnvVar,
    /// From a file path.
    File,
    /// From stdin.
    Stdin,
    /// Interactive prompt.
    Interactive,
}

/// Read a password from the `RARUST_PASSWORD` environment variable.
pub fn password_from_env() -> Option<String> {
    std::env::var("RARUST_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Read a password from the first line of a file.
pub fn password_from_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::BufRead;
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    if let Some(line) = reader.lines().next() {
        line.map(|l| l.trim_end_matches(['\n', '\r']).to_string())
    } else {
        Ok(String::new())
    }
}

/// Warn about password in process arguments (visible via `ps` on Unix).
pub fn warn_cli_password() {
    #[cfg(unix)]
    eprintln!(
        "[WARN] --password in process arguments may be visible to other users (ps -ef).\n\
         [WARN] Use RARUST_PASSWORD environment variable or --password-stdin for security."
    );
    #[cfg(windows)]
    eprintln!(
        "[WARN] --password in process arguments may be visible to other users.\n\
         [WARN] Use RARUST_PASSWORD environment variable or --password-stdin for security."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_zeroize() {
        let pwd = Password::from_string("secret123".to_string());
        assert_eq!(pwd.as_bytes(), b"secret123");
        // On drop, memory should be zeroized (verified by zeroize crate)
    }

    #[test]
    fn test_password_from_env_not_set() {
        // RARUST_PASSWORD should not be set in test environment
        assert!(password_from_env().is_none());
    }
}
