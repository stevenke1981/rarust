# Rarust — UI/UX Design

> Rarust 的 CLI/TUI 介面設計原則、色彩系統、輸出格式與互動模式。

---

## 1. Design Principles

| 原則 | 說明 |
|------|------|
| **遵循慣例** | CLI 子命令設計與 `git`、`cargo`、`7z` 一致，降低學習曲線 |
| **CI/CD 優先** | 所有命令支援 `--json` 與 `--no-progress`，可完美整合腳本管線 |
| **漸進式揭露** | 簡單操作只需 2-3 個參數，進階功能透過選項逐步揭露 |
| **可預測輸出** | stdout 只有使用者要求的輸出；stderr 只有進度與錯誤；`--quiet` 完全靜默 |
| **錯誤訊息可操作** | 錯誤訊息包含「發生什麼 + 為什麼 + 怎麼修復」三要素 |

---

## 2. Terminal Color System

### 2.1 Color Palette

```
Level      | Color    | ANSI Code   | Usage
-----------+----------+-------------+---------------------------
Error      | Bold Red   | \x1b[1;31m | Fatal errors, CRC mismatch
Warning    | Yellow     | \x1b[33m   | Non-fatal issues
Success    | Green      | \x1b[32m   | Completed operations
Info       | Cyan       | \x1b[36m   | Progress info, ETA
Highlight  | Bold White | \x1b[1;37m | File names, paths
Dim        | Bright Black| \x1b[90m  | Secondary info, timestamps
```

### 2.2 Color Implementation

使用 `anstream` crate 實現跨平台色彩：

```rust
use anstream::println;
use anstyle::*;

println!("{}{}{}",
    AnsiColor::Red.on_bold().render(),
    "Error: ",
    AnsiColor::Cyan.render(),
    "File not found",
);
```

- Windows: 自動檢測 `ENABLE_VIRTUAL_TERMINAL_PROCESSING`
- macOS/Linux: ANSI escape codes 原生支援
- `--no-color`: 完全停用所有色彩輸出
- `NO_COLOR` 環境變數支援
- 非互動式終端（piped）：自動停用顏色（可透過 `--color=always` 強制開啟）

### 2.3 Emoji / Glyph Policy

不使用 Emoji（跨平台字型不一致）。使用純 ASCII 或 Unicode 符號：

| 用途 | 符號 | Unicode |
|------|------|---------|
| Directory | `[DIR]` | — |
| File | `    ` | — |
| Symlink | `[SYM]` | — |
| Success | `[OK]` | — |
| Fail | `[FAIL]` | — |
| Warning | `[WARN]` | — |
| Progress fill | `█` | U+2588 |
| Progress empty | `░` | U+2591 |
| Separator | `─` | U+2500 |
| Bullet | `•` | U+2022 |
| Tree connector | `├──` / `└──` | Box drawing |

---

## 3. CLI Interaction Design

### 3.1 Output Modes

每個命令支援三種輸出模式：

| Mode | Flag | stdout | stderr | Use Case |
|------|------|--------|--------|----------|
| **Human** | (default) | Formatted table / status | Progress bars | Terminal use |
| **JSON** | `--json` | JSON document | Errors only | CI/CD, scripting |
| **Silent** | `--quiet` | Nothing | Errors only | Production scripts |

### 3.2 Progress Display

使用 `indicatif` 實作多層進度顯示：

**Single operation** (extract single file):
```
[00:03:05] [████████████████░░░░░░░░░░]  45%  123.4 MB/s  ETA 00:02:30
  Extracting: documents/report.pdf
  File 45/100 • 123.4 MB / 274.2 MB
```

**Multi-operation** (extract with sub-tasks):
```
[00:05:12] [████████████████████░░░░░░]  82%  98.2 MB/s  ETA 00:01:10
  ├─ Extracting: documents/report.pdf     [OK]    1.2 MB
  ├─ Extracting: images/photo.jpg         [OK]    4.5 MB
  └─ Extracting: data/database.sql       [OK]    42.1 MB

[00:05:12] [████████████████████░░░░░░]  82%  98.2 MB/s  ETA 00:01:10
  Verifying CRC32: ████████████████████░░░  85%
```

**Archive creation**:
```
[00:08:30] [████████████████░░░░░░░░░░]  42%  85.3 MB/s  ETA 00:11:40
  Compressing: src/main.rs         1.2 KB →  482 B  (60% ratio)
  Compressing: src/lib.rs          4.5 KB →  1.8 KB (60% ratio)
  Overall ratio: 58%
```

### 3.3 Status Bar Design (Stderr)

固定格式，始終寫入 stderr，不影響 stdout 的管線輸出：

```
[{elapsed}] [{bar}] {percent:>3}%  {speed:>8}  ETA {eta}
  {message}
```

### 3.4 Spinner for Indeterminate Operations

```rust
let spinner = indicatif::ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::with_template("{spinner:.cyan} {msg}"));
spinner.set_message("Scanning archive...");
// ... work ...
spinner.finish_with_message("Scan complete: 1,234 entries");
```

---

## 4. Output Format Specifications

### 4.1 Human Table (`list` command)

```
 Archive: project-backup.rar
 Format: RAR5 (v5.0) | Solid: Yes | Volumes: 1 | Encrypted: No

 Name                         Size      Ratio   Date                 CRC32     Method
 ─────────────────────────────────────────────────────────────────────────────────────
 src/
 ├── main.rs                  1.2 KB    58%     2026-06-15 14:30    A1B2C3D4  m3 normal
 ├── lib.rs                   4.5 KB    40%     2026-06-15 14:30    E5F6A7B8  m3 normal
 └── config.rs                802 B     62%     2026-06-14 10:15    9C0D1E2F  m3 normal
 docs/
 ├── guide.pdf                2.4 MB    92%     2026-06-10 09:00    3A4B5C6D  m0 store
 └── readme.txt               156 B     35%     2026-06-10 09:00    7E8F9A0B  m5 best
 ─────────────────────────────────────────────────────────────────────────────────────
 5 files                    2.4 MB     89%                       3 errors: 0
```

### 4.2 JSON Output (`list --json`)

```json
{
  "archive": "project-backup.rar",
  "format": "RAR5",
  "version": "5.0",
  "solid": true,
  "volumes": 1,
  "encrypted": false,
  "entries": [
    {
      "name": "src/main.rs",
      "size": 1228,
      "compressed_size": 712,
      "ratio": 0.58,
      "method": "normal",
      "compression_level": 3,
      "modified": "2026-06-15T14:30:00Z",
      "crc32": "A1B2C3D4",
      "blake2sp": null,
      "encrypted": false,
      "directory": false,
      "symlink": null
    }
  ],
  "summary": {
    "total_files": 5,
    "total_size": 2512346,
    "compressed_size": 2235988,
    "errors": 0
  }
}
```

### 4.3 Extraction Output

**Human mode** (`extract`):
```
[00:03:05] [████████████████░░░░░░░░░░]  45%  123.4 MB/s  ETA 00:02:30
  Extracted: documents/report.pdf  (1.2 MB, 58% ratio)
  Extracted: images/photo.jpg      (4.5 MB, 92% ratio) [stored]
  Extracted: data/database.sql     (42.1 MB, 40% ratio)
  ──────────────────────────────────────────────────────────
  3 files extracted • 47.8 MB • Speed: 98.2 MB/s • Time: 00:05:12
```

**JSON mode** (`extract --json`):
```json
{
  "command": "extract",
  "archive": "project-backup.rar",
  "destination": "/output/dir",
  "results": [
    {"file": "documents/report.pdf", "size": 1228800, "status": "ok"},
    {"file": "images/photo.jpg", "size": 4718592, "status": "ok"},
    {"file": "data/database.sql", "size": 44122000, "status": "ok"}
  ],
  "summary": {
    "extracted": 3,
    "skipped": 0,
    "errors": 0,
    "total_bytes": 50069392,
    "elapsed_secs": 312.5,
    "average_speed_mbs": 98.2
  }
}
```

### 4.4 Test Output

**Human mode**:
```
Testing archive: project-backup.rar
[00:02:10] [████████████████████████░░]  95%  210.5 MB/s  ETA 00:00:07

 Results:
 ────────────────────────────────────────────────────────
 src/main.rs                  [OK]   CRC32: A1B2C3D4 ✓
 src/lib.rs                   [OK]   CRC32: E5F6A7B8 ✓
 docs/guide.pdf               [OK]   CRC32: 3A4B5C6D ✓
 ────────────────────────────────────────────────────────
 3 files tested • 3 OK • 0 failed • 0 warnings
```

**JSON mode**:
```json
{
  "command": "test",
  "archive": "project-backup.rar",
  "results": [
    {"file": "src/main.rs", "status": "ok", "crc32": "A1B2C3D4", "blake2sp": null},
    {"file": "src/lib.rs", "status": "ok", "crc32": "E5F6A7B8", "blake2sp": null}
  ],
  "summary": {"total": 3, "ok": 3, "failed": 0, "warnings": 0, "elapsed_secs": 130}
}
```

### 4.5 Create Output

**Human mode**:
```
Creating archive: project-backup.rar
[00:08:30] [████████████████░░░░░░░░░░]  42%  85.3 MB/s  ETA 00:11:40
 Files added: 5/12  •  Compressed: 2.1 MB / 4.7 MB  •  Ratio: 44%

 ────────────────────────────────────────────────────────
 12 files compressed • 4.7 MB → 2.1 MB (44% ratio)
 Method: m3 normal • Solid: Yes • Dictionary: 32 MB
 Time: 00:20:10 • Speed: 85.3 MB/s
```

### 4.6 Repair Output

```
Repairing archive: damaged-backup.rar
 Stage 1: Scanning for damaged blocks... [OK] 3 damaged blocks found
 Stage 2: Rebuilding from recovery record... [OK] 2 blocks recovered
 Stage 1 blocks: 3/3 recovered
 Unrecoverable: 0 blocks

 Output written to: damaged-backup.repaired.rar
```

---

## 5. TUI Mode Design (`rarust tui`)

### 5.1 TUI Layout (ratatui)

```
┌────────────────────────────────────────────────────┐
│  rarust — Archive Browser        project-backup.rar │  ← Title bar
├──────────────────────────┬─────────────────────────┤
│  📁 src/                  │  Name:    main.rs       │  ← Split pane
│    ├── main.rs            │  Size:    1.2 KB        │
│    ├── lib.rs             │  Ratio:   58%           │
│    └── config.rs          │  Method:  m3 normal     │
│  📁 docs/                 │  Date:    2026-06-15    │
│    ├── guide.pdf          │  CRC32:   A1B2C3D4     │
│    └── readme.txt         │  Encrypted: No          │
│                           │                         │
│                           │  [Extract] [View] [Info]│  ← Actions
├──────────────────────────┴─────────────────────────┤
│  [Search: ______________]    5 files | 2.4 MB       │  ← Status bar
└────────────────────────────────────────────────────┘
```

### 5.2 TUI Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate file list |
| `→` / `←` | Expand/collapse directory |
| `Enter` | Open file / execute action |
| `/` | Search/filter files |
| `e` | Extract selected file(s) |
| `x` | Extract all |
| `t` | Test integrity |
| `i` | Show file info panel |
| `v` | View file content preview (text) |
| `Space` | Select/deselect file |
| `Tab` | Focus switch (tree → info → search) |
| `q` / `Esc` | Quit TUI |
| `?` | Show help overlay |

### 5.3 TUI Color Scheme

```
Element            | Color         | Style
-------------------+---------------+-------------------
Title bar          | White on Blue | Bold
File tree          | Cyan          | Normal
Directory          | Yellow        | Bold
Selected file      | White on Blue | Reversed
Highlight match    | Black on Yellow| —
Info panel title   | Cyan          | Bold
Info panel value   | White         | Normal
Status bar         | White on Black| —
Error message      | Red           | Bold
Help overlay       | White on Dim  | Dim background
```

---

## 6. Error Message Design

### 6.1 Error Message Structure

```
Error: <type>: <what happened>
Help: <why it happened>
Help: <how to fix or workaround>
```

### 6.2 Error Examples

```
Error: Format error: Invalid RAR5 signature (expected 52 61 72 21 1A 07 01 00, got 52 61 72 21 1A 07 00 00)
Help: This file appears to be a RAR4 archive. Use `rarust list` without format detection.
Help: RAR4 archives use 7-byte signatures (missing the 8th version byte).

Error: CRC mismatch: src/main.rs (expected A1B2C3D4, actual E5F6A7B8)
Help: The file was modified or the archive is corrupted.
Help: Try `rarust test` with --keep-broken to extract damaged files.

Error: Wrong password: Password check value mismatch
Help: The archive is encrypted and the provided password is incorrect.
Help: Use -p/--password with the correct password, or set RARUST_PASSWORD.

Error: Volume missing: project-backup.part3.rar not found
Help: This is a multi-volume archive and part 3 is required.
Help: Ensure all .partN.rar files are in the same directory.

Warning: --password in process arguments may be visible to other users (ps -ef)
Help: Use RARUST_PASSWORD environment variable or --password-stdin for security.
```

### 6.3 Warning Design

Warnings are yellow, non-fatal, and do not change exit code (unless `--warnings-as-errors`):

```
[WARN] Unsupported recovery record version: 2 (expected 0). Skipping repair.
[WARN] 2 files skipped due to exclude pattern "*.tmp"
[WARN] Archive comment exceeds 256KB, truncating to 256KB
```

---

## 7. CLI Interaction Rules

### 7.1 Prompt Behavior

- **Destructive operations** (overwrite, delete): prompt for confirmation by default
- `--force` / `-f`: skip all prompts (assume yes)
- `--assume-yes` / `-y`: answer yes to all prompts (for scripts)
- Non-interactive terminal: automatically behave as `--assume-yes`

### 7.2 Dry Run

All destructive commands support `--dry-run` / `-d`:

```
$ rarust extract backup.rar --dry-run
[Dry Run] Would extract 12 files to /current/directory
[Dry Run]   src/main.rs → /current/directory/src/main.rs (1.2 KB)
[Dry Run]   src/lib.rs  → /current/directory/src/lib.rs (4.5 KB)
...
[Dry Run] Total: 12 files • 47.8 MB • Disk space needed: 47.8 MB
```

### 7.3 Password Input Priority

1. `--password <PASSWORD>` CLI argument (emits warning about process visibility)
2. `RARUST_PASSWORD` environment variable (recommended for scripts)
3. `--password-file <PATH>` (read first line as password)
4. `--password-stdin` (read from stdin, for piping)
5. Interactive TTY prompt: `Enter password:` (hidden input)

### 7.4 Signal Handling

| Signal | Behavior |
|--------|----------|
| `SIGINT` (Ctrl+C) | Graceful shutdown: complete current file, save partial output, print summary |
| `SIGTERM` | Same as SIGINT |
| `SIGPIPE` | Exit cleanly (broken pipe for `| head`) |

On interrupt:
```
^C
Interrupted by user
 Partial results:
   Extracted: 3/12 files complete
   Output: /output/dir/ (may contain partial files)
   To resume: rarust extract backup.rar -o skip
```

---

## 8. Exit Code Design

See [spec.md §3.4](spec.md) for the complete exit code table.

Key design: exit codes match WinRAR/UnRAR conventions where possible for drop-in replacement:

| Code | Rarust | WinRAR | unrar |
|:----:|--------|:------:|:-----:|
| 0 | Success | ✅ | ✅ |
| 1 | Warning (non-fatal) | ✅ | ✅ |
| 2 | Fatal error | ✅ | ✅ |
| 3 | CRC error | ✅ | ✅ |
| 11 | Wrong password | — | ✅ |

---

## 9. Unicode and Locale Handling

### 9.1 Output Encoding

- **stdout**: Always UTF-8 (Windows: `chcp 65001` detection; fallback to replacement characters)
- **JSON output**: Always UTF-8 with `\uXXXX` escaping for non-BMP characters
- **File names on disk**: Platform-native encoding (Windows UTF-16LE, macOS NFD decomposition, Linux raw UTF-8)

### 9.2 macOS NFD Normalization

On macOS, HFS+/APFS decomposes Unicode (NFD). Rarust provides:

- **Default**: Preserve NFD (matches Finder behavior)
- `--output-encoding=nfc`: Convert to NFC (matches Linux/Windows behavior)
- `--output-encoding=auto`: Detect from locale (recommended)

---

## 10. Accessibility

- `--no-color`: All information conveyed without color
- `--no-progress`: Progress information in text form only
- Screen reader friendly: JSON output for assistive technology integration
- High-contrast mode: Automatically detected via terminal theme (future)
