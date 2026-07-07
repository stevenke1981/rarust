# Rarust — Technical Specification

> **Rarust**（RAR + Rust）是一個現代化、純 Rust 實作的 RAR 命令列工具。
> 支援 RAR4/RAR5 的建立、解壓縮、測試、修復，具備 TUI 進度介面與 JSON 輸出。

---

## 1. Project Overview

### 1.1 Vision

提供一個**開源、跨平台、可腳本化**的 RAR 工具，克服現有工具（WinRAR CLI、`unrar`、`7z`）的痛點：
- 無授權焦慮（MIT / Apache-2.0）
- 現代 CLI/TUI 介面（進度條、顏色、即時 ETA）
- JSON 輸出，支援 CI/CD 管線整合
- 純 Rust 實作，單一二進位檔，跨平台一致行為

### 1.2 Project Name

| 項目 | 值 |
|------|-----|
| **名稱** | Rarust (RAR + Rust) |
| **內部代號** | `rarc` (Rust Archive CLI) |
| **執行檔名** | `rarust` |
| **License** | MIT / Apache-2.0 dual-license |
| **最低 Rust 版本** | MSRV 1.85 (Edition 2024) |

### 1.3 Target Audience

- **開發者**：需要可腳本化的 CI/CD 工具，JSON 輸出
- **系統管理員**：需要跨平台一致的 RAR 處理
- **一般使用者**：需要 TUI 介面的輕量工具
- **Linux 發行版維護者**：需要無授權問題的 RAR 工具

---

## 2. Technical Architecture

### 2.1 Crate Dependency Tree

```
rarust (binary crate)
├── rarust-core (library crate, pure Rust)
│   ├── rars (RAR format: read/write/test/repair — MIT/Apache-2.0)
│   ├── aes-gcm (AES-256-CBC for RAR5 encryption)
│   ├── sha2 + hmac + pbkdf2 (key derivation)
│   └── blake2s_simd (BLAKE2sp hash verification)
│
├── clap (derive API, argument parsing)
├── anyhow + thiserror (error handling)
├── indicatif (progress bars, multi-spinner)
├── serde + serde_json (JSON output)
├── tabled (table formatted output)
├── anstream (cross-platform colored output)
├── walkdir (directory traversal)
├── dunce (cross-platform path normalization)
├── rayon (optional parallel operations)
└── ratatui (optional TUI archive browser, feature-gated)
```

### 2.2 Architecture Layers

```
┌──────────────────────────────────────────────────┐
│              CLI Layer (rarust)                   │
│  clap derive · anyhow · anstream · indicatif      │
│  Subcommands: create, extract, list, test,        │
│               repair, benchmark, tui              │
├──────────────────────────────────────────────────┤
│            Core Library (rarust-core)             │
│  rarust-core API:                                 │
│    - Archive::open(path) → Archive                │
│    - Archive::extract(entries, dest) → Result     │
│    - Archive::list() → Vec<Entry>                 │
│    - ArchiveBuilder::new() → ArchiveBuilder       │
│    - ArchiveBuilder::add_file(path) → Self        │
│    - ArchiveBuilder::build(dest) → Result         │
│  Stream adapters: CompressionMethod, Encryption,  │
│                   SolidStream, MultiVolumeReader   │
├──────────────────────────────────────────────────┤
│              RAR Backend (rars)                   │
│  - RAR4 & RAR5 parser                             │
│  - LZSS + Huffman decoder                         │
│  - AES-256-CBC decrypt                           │
│  - PPMd decoder (RAR4)                            │
│  - Reed-Solomon recovery (RAR5)                   │
│  - Multi-volume concatenation                     │
│  - Solid archive state machine                    │
├──────────────────────────────────────────────────┤
│         System Layer (std::fs, std::io)           │
│  - Platform-native path handling                  │
│  - Permission preservation                        │
│  - Memory-mapped I/O for large files              │
└──────────────────────────────────────────────────┘
```

### 2.3 Module Structure

```
rarust/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point, CLI dispatch
│   ├── cli.rs                   # clap derive definitions
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── create.rs            # `rarust create` implementation
│   │   ├── extract.rs           # `rarust extract` implementation
│   │   ├── list.rs              # `rarust list` implementation
│   │   ├── test.rs              # `rarust test` implementation
│   │   ├── repair.rs            # `rarust repair` implementation
│   │   ├── benchmark.rs         # `rarust benchmark` implementation
│   │   └── tui.rs               # `rarust tui` (ratatui browser, feature-gated)
│   ├── output/
│   │   ├── mod.rs
│   │   ├── human.rs             # Human-readable table output
│   │   ├── json.rs              # JSON serialization
│   │   └── progress.rs          # indicatif progress bar integration
│   └── utils/
│       ├── mod.rs
│       ├── path.rs              # Cross-platform path handling
│       ├── password.rs          # Password input (stdin, env, file, keychain)
│       └── filter.rs            # File inclusion/exclusion patterns
│
├── rarust-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Public API
│       ├── archive.rs           # Archive open/create abstraction
│       ├── entry.rs             # Archive entry metadata
│       ├── error.rs             # Error types
│       ├── encryption.rs        # Password derivation, AES-256
│       ├── recovery.rs          # Recovery record reading
│       ├── multi.rs             # Multi-volume handling
│       └── util.rs              # Shared utilities
│
├── tests/
│   ├── integration/
│   │   ├── extract_test.rs      # Extraction test suite
│   │   ├── create_test.rs       # Creation test suite
│   │   ├── list_test.rs         # Listing test suite
│   │   ├── multi_test.rs        # Multi-volume test suite
│   │   └── password_test.rs     # Encryption test suite
│   └── fixtures/                # Test RAR files (small, synthetic)
│       ├── rar5-test.rar
│       ├── rar4-test.rar
│       ├── encrypted.rar
│       ├── multi-part.part1.rar
│       └── ...
│
└── benches/
    ├── extraction.rs            # Extraction benchmarks
    └── creation.rs              # Creation benchmarks
```

---

## 3. Feature Specification

### 3.1 Core Features (MVP)

| Feature | Priority | RAR4 | RAR5 | Description |
|---------|----------|:----:|:----:|-------------|
| List contents | P0 | ✅ | ✅ | Display archive file listing with metadata |
| Extract (full) | P0 | ✅ | ✅ | Extract all files with path preservation |
| Extract (selected) | P0 | ✅ | ✅ | Extract specific files by pattern |
| Test integrity | P0 | ✅ | ✅ | Verify archive CRC/BLAKE2 + decompress test |
| Create archive | P0 | ✅ | ✅ | Create new RAR5 archive from files |
| Compression levels | P0 | — | ✅ | m0 (store) through m5 (best) |
| Solid archives | P0 | — | ✅ | `-s` flag for solid compression |
| Multi-volume | P0 | ✅ | ✅ | Create/extract `.partN.rar` archives |
| Password protection | P0 | — | ✅ | AES-256 with PBKDF2 key derivation |
| Header encryption | P0 | — | ✅ | `-hp` style filename encryption |
| Progress display | P0 | — | — | indicatif-based progress bars |
| JSON output | P0 | — | — | `--json` flag for all commands |
| Cross-platform | P0 | — | — | Windows, macOS, Linux |

### 3.2 Advanced Features (Post-MVP)

| Feature | Priority | Description |
|---------|----------|-------------|
| Recovery record | P1 | Add/read `-rr[N]` Reed-Solomon protection |
| Recovery volumes | P1 | Create `.rev` files for volume reconstruction |
| Archive repair | P1 | Rebuild damaged archives using recovery data |
| BLAKE2 hashing | P1 | `-htb` flag for BLAKE2sp hash verification |
| Parallel extraction | P1 | Rayon-based parallel decompression (non-solid) |
| File filtering | P1 | Name/size/date/regex-based include/exclude |
| Overwrite modes | P1 | skip/overwrite/rename/ask |
| Archive comments | P2 | Read/write RAR comments |
| Benchmark | P2 | Built-in compression speed benchmark |
| TUI browser | P2 | ratatui-based interactive archive browser |
| Streaming extract | P2 | Download + extract in one pass |
| SFX creation | P3 | Self-extracting archive support |
| Update modes | P3 | Freshen/synchronize archive contents |
| Archive conversion | P3 | Between RAR4/RAR5/ZIP (via external crate) |

### 3.3 CLI Interface Specification

#### Top-Level Usage

```
rarust [OPTIONS] <COMMAND>

Commands:
  create     Create a RAR archive
  extract    Extract files from a RAR archive
  list       List archive contents
  test       Test archive integrity
  repair     Repair a damaged archive
  benchmark  Run compression benchmark
  tui        Interactive archive browser (TUI mode)
  help       Print help information

Global Options:
  -v, --verbose...           Increase verbosity
  -q, --quiet                Suppress non-error output
      --json                 Output in JSON format
      --no-color             Disable colored output
      --no-progress          Hide progress bars (for CI scripts)
  -h, --help                 Print help
  -V, --version              Print version
```

#### `rarust create` Subcommand

```
rarust create [OPTIONS] <ARCHIVE> <INPUT>...

Arguments:
  <ARCHIVE>              Output archive path (e.g., output.rar)
  <INPUT>...             Input files/directories to add

Options:
  -m, --method <LEVEL>       Compression level: store|fastest|fast|normal|good|best [default: normal]
  -s, --solid                Create solid archive
  -v, --volume <SIZE>        Volume size (e.g., 100m, 1g)
  -p, --password <PASSWORD>  Set password for encryption
  -h, --header-encrypt       Encrypt file headers (requires --password)
  -r, --recovery <N>         Add recovery record (N = percent of archive size)
  -e, --encrypt-filenames    Alias for --header-encrypt
  -n, --name-encoding <ENC>  Filename encoding (auto|utf-8) [default: auto]
      --filter <PATTERN>     Include files matching pattern
      --exclude <PATTERN>    Exclude files matching pattern
      --store <EXT>          Store without compression for extensions
      --threads <N>          Number of compression threads [default: auto]
      --no-dir-records       Don't store directory entries
      --comment-file <FILE>  Read archive comment from file
  -o, --overwrite            Overwrite output if exists [default: prompt]
  -d, --dry-run              Show what would be done without executing
  -f, --force                Force overwrite without prompt
  -y, --assume-yes           Assume yes for all prompts
```

#### `rarust extract` Subcommand

```
rarust extract [OPTIONS] <ARCHIVE> [DEST]

Arguments:
  <ARCHIVE>              Source archive path
  [DEST]                 Output directory [default: current directory]

Options:
  -p, --password <PASSWORD>    Decryption password
  -e, --extract-paths          Extract with full paths [default]
  -f, --flat                   Extract without paths (all to same dir)
  -o, --overwrite <MODE>       Overwrite mode: skip|rename|overwrite|ask [default: ask]
  -i, --include <PATTERN>      Extract only files matching pattern
  -x, --exclude <PATTERN>      Skip files matching pattern
      --filter-size <RANGE>    File size filter (e.g., 1k-10m)
      --filter-date <RANGE>    Date filter (e.g., after:2024-01-01)
      --no-directories         Skip directory entries
      --keep-broken            Save partially extracted files on error
      --preserve-perms         Preserve file permissions [default: true]
      --preserve-ownership     Preserve file ownership (requires root)
      --threads <N>            Number of extraction threads [default: auto]
      --output-encoding <ENC>  Output filename encoding
  -d, --dry-run                Show what would be extracted without writing
  -l, --list-only              Alias for `rarust list`
```

#### `rarust list` Subcommand

```
rarust list [OPTIONS] <ARCHIVE>

Arguments:
  <ARCHIVE>              Archive path

Options:
  -p, --password <PASSWORD>   Password for encrypted archives
  -v, --verbose               Show detailed metadata (CRC, compression, method, etc.)
  -t, --tree                  Display as directory tree
      --no-header             Omit column headers (for scripting)
      --columns <COLS>        Custom columns: name,size,ratio,date,crc,method,type
  -n, --name-only             Print filenames only (one per line)
      --sort <FIELD>          Sort by: name|size|date|ratio|crc [default: name]
      --reverse               Reverse sort order
```

#### `rarust test` Subcommand

```
rarust test [OPTIONS] <ARCHIVE>

Arguments:
  <ARCHIVE>              Archive path

Options:
  -p, --password <PASSWORD>   Password for encrypted archives
  -v, --verbose               Show detailed test results per-file
  -q, --quiet                 Only show summary (total/ok/fail)
      --skip-hash             Skip BLAKE2/CRC hash verification
      --progress              Show progress bar [default: true]
```

#### `rarust repair` Subcommand

```
rarust repair [OPTIONS] <ARCHIVE>

Arguments:
  <ARCHIVE>              Archive path

Options:
  -p, --password <PASSWORD>   Password for encrypted archives
  -o, --output <PATH>         Output repaired archive path [default: ARCHIVE.repaired.rar]
  -r, --recovery-volumes      Also attempt recovery from .rev files
      --force-stage2          Attempt stage 2 repair even without recovery record
  -v, --verbose               Show detailed repair log
```

#### `rarust benchmark` Subcommand

```
rarust benchmark [OPTIONS]

Options:
  -m, --method <LEVEL>         Compression level to test [default: best]
  -d, --dictionary <SIZE>      Dictionary size (e.g., 32m, 64m, 128m) [default: 32m]
  -t, --threads <N>            Number of threads [default: auto]
      --data-size <SIZE>       Test data size (e.g., 100m, 1g) [default: 100m]
      --format <FORMAT>        Output format: table|json|markdown [default: table]
```

### 3.4 Exit Codes

| Code | Meaning |
|:----:|---------|
| 0 | Success |
| 1 | Warning (CRC error in one file, others OK) |
| 2 | Fatal error |
| 3 | CRC error (all files, or specified file failed) |
| 4 | Locked archive |
| 5 | Write error (disk full, permission) |
| 6 | Open error (file not found, corrupt header) |
| 7 | User error (bad arguments) |
| 8 | Memory error |
| 9 | Feature not implemented yet |
| 10 | No files found matching pattern |
| 11 | Wrong password |

---

## 4. RAR Format Support Matrix

| Feature | RAR4 (v3.x) | RAR5 (v5.0) | RAR7 (v7.0) | Notes |
|---------|:-----------:|:-----------:|:-----------:|-------|
| **Signature** | `52 61 72 21 1A 07 00` | `52 61 72 21 1A 07 01 00` | TBD | |
| **Extraction** | ✅ v3+ | ✅ | ⚠️ Partial (v1 algo) | RAR7 is very new (2024) |
| **Creation** | ⚠️ Legacy | ✅ Primary target | ❌ | RAR5 is default modern format |
| **Solid archives** | ✅ | ✅ | ✅ | |
| **Multi-volume** | ✅ `.partN.rar` / `.r00` | ✅ `.partN.rar` | ✅ | Only `.partN.rar` for RAR5+ |
| **Password (AES-128)** | ✅ | ❌ | ❌ | RAR4 legacy, deprecated |
| **Password (AES-256)** | ❌ | ✅ | ✅ | Modern standard |
| **Header encryption** | ⚠️ Partial (flag 0x0080) | ✅ | ✅ | |
| **Recovery record** | ❌ | ✅ (v0 RS) | ✅ | RAR4 uses different format |
| **Recovery volumes** | ❌ | ✅ | ✅ | RAR4 max 255, RAR5 max 65535 |
| **BLAKE2sp hash** | ❌ | ✅ | ✅ | |
| **Quick-open record** | ❌ | ✅ | ✅ | |
| **64GB+ dictionary** | ❌ | ❌ (max 1GB) | ✅ | RAR7 theoretical 64GB |
| **Comment** | ✅ (UTF-16, 64KB) | ✅ (UTF-8, 256KB) | ✅ | |
| **Symlinks** | ❌ | ✅ (extra record) | ✅ | |
| **AV1 decode** | ❌ | N/A | N/A | Media engine, not RAR |
| **PPMd** | ✅ | ❌ (removed) | ❌ | RAR5 uses new LZ+Huffman |
| **Text/Multimedia filters** | ✅ | ❌ (removed) | ❌ | |

---

## 5. Cross-Platform Requirements

| Platform | Minimum Version | Architecture | Notes |
|----------|----------------|:------------:|-------|
| Windows | 10 21H2+ | x86_64, aarch64 | Long path support, Win32 console |
| macOS | 12 Monterey+ | x86_64, aarch64 | Apple Silicon native |
| Linux | Kernel 5.10+ | x86_64, aarch64 | glibc 2.31+, musl target for Alpine |

### 5.1 Path Handling

- **Windows**: Use verbatim `\\?\` paths for >260 char paths via `dunce` crate
- **macOS**: Normalize HFS+/APFS decomposed Unicode (NFD → NFC option)
- **Linux**: Raw UTF-8 byte sequences (no locale conversion needed)
- All platforms: Forward slash (`/`) as RAR internal separator → platform-native on extraction

### 5.2 Performance Targets

| Operation | Target (M4 Mac / i7-13700 / AMD Ryzen 9) |
|-----------|------------------------------------------|
| **List** 10k-entry archive | < 200ms |
| **Extract** 1GB (stored, non-solid) | > 800 MB/s |
| **Extract** 1GB (compressed, non-solid) | > 200 MB/s |
| **Extract** 1GB (solid) | > 100 MB/s |
| **Create** 1GB (m3 normal, non-solid) | > 80 MB/s |
| **Test** 1GB archive | > 300 MB/s |
| **Memory usage** (default) | < 256 MB |
| **Binary size** (stripped) | < 10 MB |

---

## 6. Security Considerations

- **Password in process args**: Warn user, support `RARUST_PASSWORD` env var, `--password-file`, or `--password-stdin`
- **Sensitive memory**: Use `zeroize` to clear passwords from memory after use
- **Safe path handling**: Prevent directory traversal attacks via crafted archive entries (`../../etc/passwd`)
- **Symlink safety**: Option to skip/limit symlink creation on extraction
- **Resource limits**: Configurable memory limit for decompression, bail on dictionary > configured limit

---

## 7. Dependency Versioning

```toml
[package]
name = "rarust"
version = "0.1.0"
edition = "2024"

[dependencies]
# CLI framework
clap = { version = "4.6", features = ["derive", "env", "unicode"] }

# RAR backend
rars = "0.4"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Output
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tabled = "0.21"
anstream = "0.6"

# Progress
indicatif = "0.18"

# Filesystem
walkdir = "2.5"
dunce = "1.0"

# Security
aes = "0.8"
aes-gcm = "0.10"      # AES-256-CBC via AES + CBC mode
sha2 = "0.10"
hmac = "0.12"
pbkdf2 = "0.12"
zeroize = "1.8"

# Optional
rayon = { version = "1.10", optional = true }
ratatui = { version = "0.30", optional = true, features = ["crossterm"] }
blake2 = { version = "0.10", optional = true }

[features]
default = ["parallel"]
parallel = ["rayon"]
tui = ["ratatui"]
full-hash = ["blake2"]

[profile.release]
lto = true
strip = true
codegen-units = 1
```

---

## 8. Specification References

- RARLAB Technote: https://www.rarlab.com/technote.htm
- Library of Congress RAR4: https://www.loc.gov/preservation/digital/formats/fdd/fdd000458.shtml
- Library of Congress RAR5: https://www.loc.gov/preservation/digital/formats/fdd/fdd000460.shtml
- UnRAR source code: https://github.com/aawc/unrar
- 7-Zip RAR5 handler: https://github.com/ip7z/7zip
- Kaitai Struct RAR spec: https://formats.kaitai.io/rar/
- `rars` crate: https://crates.io/crates/rars
- BLAKE2sp specification: https://www.blake2.net/
- Reed-Solomon error correction: https://en.wikipedia.org/wiki/Reed%E2%80%93Solomon_error_correction
