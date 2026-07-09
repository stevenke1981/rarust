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

    /// Return a non-secret owned `String` copy (prefer `as_bytes` when possible).
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password([REDACTED])")
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

/// Read a password from the first line of stdin.
pub fn password_from_stdin() -> std::io::Result<String> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

/// Resolve a password from CLI sources in priority order:
/// 1. explicit CLI argument (caller should warn about process visibility)
/// 2. `RARUST_PASSWORD` environment variable
/// 3. first line of `password_file`
/// 4. first line of stdin when `from_stdin` is true
///
/// Returns `None` when no source provided a non-empty password.
pub fn resolve_password(
    cli_arg: Option<String>,
    password_file: Option<&std::path::Path>,
    from_stdin: bool,
) -> crate::error::Result<Option<Password>> {
    if let Some(p) = cli_arg.filter(|s| !s.is_empty()) {
        warn_cli_password();
        return Ok(Some(Password::from_string(p)));
    }
    if let Some(p) = password_from_env() {
        return Ok(Some(Password::from_string(p)));
    }
    if let Some(path) = password_file {
        let p = password_from_file(path).map_err(crate::error::RarustError::Io)?;
        if !p.is_empty() {
            return Ok(Some(Password::from_string(p)));
        }
    }
    if from_stdin {
        let p = password_from_stdin().map_err(crate::error::RarustError::Io)?;
        if !p.is_empty() {
            return Ok(Some(Password::from_string(p)));
        }
    }
    Ok(None)
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
