//! CLI argument definitions using clap derive.

use clap::{Parser, Subcommand, ValueEnum};

/// A modern, pure Rust RAR command-line tool.
#[derive(Parser, Debug)]
#[command(name = "rarust")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Increase verbosity (can be specified multiple times: -v, -vv, -vvv).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output.
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Output in JSON format.
    #[arg(long = "json", global = true)]
    pub json: bool,

    /// Disable colored output.
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    /// Hide progress bars (for CI scripts).
    #[arg(long = "no-progress", global = true)]
    pub no_progress: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a RAR archive.
    Create(CreateArgs),

    /// Extract files from a RAR archive.
    Extract(ExtractArgs),

    /// List archive contents.
    List(ListArgs),

    /// Test archive integrity.
    Test(TestArgs),

    /// Repair a damaged archive.
    Repair(RepairArgs),

    /// Run compression benchmark.
    Benchmark(BenchmarkArgs),

    /// Interactive archive browser (TUI mode).
    #[cfg(feature = "tui")]
    Tui(TuiArgs),
}

/// Create a RAR archive.
#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// Compression level.
    #[arg(short = 'm', long = "method", default_value = "normal")]
    pub method: CompressionLevel,

    /// Create solid archive.
    #[arg(short = 's', long = "solid")]
    pub solid: bool,

    /// Volume size (e.g., 100m, 1g).
    #[arg(long = "volume")]
    pub volume: Option<String>,

    /// Set password for encryption.
    #[arg(short = 'p', long = "password")]
    pub password: Option<String>,

    /// Encrypt file headers / filenames (requires `--password`).
    #[arg(long = "header-encrypt", alias = "encrypt-filenames")]
    pub header_encrypt: bool,

    /// Add recovery record (N = percent of archive size).
    #[arg(short = 'r', long = "recovery")]
    pub recovery: Option<u32>,

    /// Include files matching pattern.
    #[arg(long = "filter")]
    pub filter: Option<String>,

    /// Exclude files matching pattern.
    #[arg(long = "exclude")]
    pub exclude: Option<String>,

    /// Force overwrite output without prompt.
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Show what would be done without executing.
    #[arg(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// Assume yes for all prompts.
    #[arg(short = 'y', long = "assume-yes")]
    pub assume_yes: bool,

    /// Output archive format (`rar5` or `rar4`).
    #[arg(long = "format", default_value = "rar5")]
    pub format: ArchiveFormatArg,

    /// Output archive path.
    pub archive: String,

    /// Input files/directories to add.
    #[arg(required = true, num_args = 1..)]
    pub inputs: Vec<String>,
}

/// Extract files from a RAR archive.
#[derive(Parser, Debug)]
pub struct ExtractArgs {
    /// Source archive path.
    pub archive: String,

    /// Output directory (default: current directory).
    pub dest: Option<String>,

    /// Decryption password.
    #[arg(short = 'p', long = "password")]
    pub password: Option<String>,

    /// Extract without paths (all to same directory).
    #[arg(short = 'f', long = "flat")]
    pub flat: bool,

    /// Overwrite mode.
    #[arg(short = 'o', long = "overwrite", default_value = "ask")]
    pub overwrite: OverwriteMode,

    /// Extract only files matching pattern.
    #[arg(short = 'i', long = "include")]
    pub include: Option<String>,

    /// Skip files matching pattern.
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Option<String>,

    /// Keep broken files on CRC error.
    #[arg(long = "keep-broken")]
    pub keep_broken: bool,

    /// Show what would be extracted without writing.
    #[arg(short = 'd', long = "dry-run")]
    pub dry_run: bool,
}

/// List archive contents.
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Archive path.
    pub archive: String,

    /// Password for encrypted archives.
    #[arg(short = 'p', long = "password")]
    pub password: Option<String>,

    /// Show detailed metadata.
    #[arg(short = 't', long = "tree")]
    pub tree: bool,

    /// Print filenames only (one per line).
    #[arg(short = 'n', long = "name-only")]
    pub name_only: bool,

    /// Sort field.
    #[arg(long = "sort", default_value = "name")]
    pub sort: SortField,

    /// Reverse sort order.
    #[arg(long = "reverse")]
    pub reverse: bool,
}

/// Test archive integrity.
#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Archive path.
    pub archive: String,

    /// Password for encrypted archives.
    #[arg(short = 'p', long = "password")]
    pub password: Option<String>,

    /// Only show summary.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

/// Repair a damaged archive.
#[derive(Parser, Debug)]
pub struct RepairArgs {
    /// Archive path.
    pub archive: String,

    /// Password for encrypted archives.
    #[arg(short = 'p', long = "password")]
    pub password: Option<String>,

    /// Output repaired archive path.
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,
}

/// Run compression benchmark.
#[derive(Parser, Debug)]
pub struct BenchmarkArgs {
    /// Compression level to test.
    #[arg(short = 'm', long = "method", default_value = "best")]
    pub method: CompressionLevel,

    /// Dictionary size (e.g., 32m, 64m).
    #[arg(short = 'd', long = "dictionary", default_value = "32m")]
    pub dictionary: String,

    /// Output format.
    #[arg(long = "format", default_value = "table")]
    pub format: BenchmarkFormat,
}

/// Interactive archive browser (TUI mode).
#[cfg(feature = "tui")]
#[derive(Parser, Debug)]
pub struct TuiArgs {
    /// Archive path to open.
    pub archive: String,
}

// --- Value enums ---

/// Compression level for create and benchmark.
#[derive(Clone, Debug, ValueEnum)]
pub enum CompressionLevel {
    Store,
    Fastest,
    Fast,
    Normal,
    Good,
    Best,
}

/// Overwrite behavior during extraction.
#[derive(Clone, Debug, ValueEnum)]
pub enum OverwriteMode {
    Skip,
    Rename,
    Overwrite,
    Ask,
}

/// Sort field for list command.
#[derive(Clone, Debug, ValueEnum)]
pub enum SortField {
    Name,
    Size,
    Date,
    Ratio,
    Crc,
}

/// Benchmark output format.
#[derive(Clone, Debug, ValueEnum)]
pub enum BenchmarkFormat {
    Table,
    Json,
    Markdown,
}

/// Archive format for the create command.
#[derive(Clone, Debug, ValueEnum)]
pub enum ArchiveFormatArg {
    /// RAR 5.x (default).
    Rar5,
    /// RAR 4.x legacy format.
    Rar4,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn create_args_parse_header_encrypt_after_password_short_flag() {
        let cli = Cli::parse_from([
            "rarust",
            "create",
            "out.rar",
            "in.txt",
            "-p",
            "hpw",
            "--header-encrypt",
            "-f",
        ]);
        let Commands::Create(args) = cli.command else {
            panic!("expected create subcommand");
        };
        assert_eq!(args.password.as_deref(), Some("hpw"));
        assert!(args.header_encrypt);
    }

    #[test]
    fn create_args_parse_encrypt_filenames_alias() {
        let cli = Cli::parse_from([
            "rarust",
            "create",
            "--password",
            "hpw",
            "--encrypt-filenames",
            "out.rar",
            "in.txt",
        ]);
        let Commands::Create(args) = cli.command else {
            panic!("expected create subcommand");
        };
        assert!(args.header_encrypt);
    }
}
