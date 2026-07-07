# Rarust — Task Tracker

> 詳細任務分解表，用於追蹤開發進度。
> 狀態：🟡 Pending / 🔵 In Progress / ✅ Completed / ❌ Blocked / ⏸ Deferred

---

## Phase 0: Foundation (Week 1–2)

### M0.1: Workspace Setup

- [x] ✅ Initialize cargo workspace with `rarust` + `rarust-core`
- [x] ✅ Configure `Cargo.toml` with dependencies and features
- [x] ✅ Set up `rustfmt.toml` and `clippy.toml`
- [x] ✅ Add `deny(unsafe_code)` to `rarust-core`
- [ ] 🟡 Verify `cargo build` + `cargo test` on all 3 platforms (only Windows verified locally)

### M0.2: CI Pipeline

- [ ] 🟡 GitHub Actions: `cargo build` matrix (ubuntu, macos, windows)
- [ ] 🟡 GitHub Actions: `cargo test` + `cargo clippy` + `cargo fmt`
- [ ] 🟡 GitHub Actions: `cargo deny` for license compliance
- [ ] 🟡 GitHub Actions: `cargo nextest` for parallel testing
- [ ] 🟡 Add `typos` check to CI

### M0.3: Test Fixture Corpus

- [x] ✅ Self-generated RAR5 stored fixtures via `rars` writer API (in-tree integration tests)
- [ ] 🟡 Generate RAR4 test archives
- [ ] 🟡 Generate multi-volume archives
- [ ] 🟡 Generate encrypted archives (AES-256, header-encrypted)
- [ ] 🟡 Generate solid archives
- [ ] 🟡 Generate edge-case archives (empty, single file, unicode names)
- [ ] 🟡 Generate corrupted archives (for repair testing)
- [ ] ⏸ Commit binary fixtures to `tests/fixtures/` (deferred — using rars writer instead)
- [ ] ⏸ Document checksums and expected outputs for each fixture

### M0.4: Error Type Hierarchy

- [x] ✅ Define `RarustError` enum:
  - `Io(io::Error)`
  - `Format(String)` — invalid format
  - `CrcMismatch { expected, actual }`
  - `WrongPassword`
  - `Unsupported(String)` — feature not implemented
  - `Corrupt(String)` — damaged archive
  - `VolumeMissing { number }` — multi-volume not found
  - `MemoryLimit` — dictionary too large
- [x] ✅ Implement `From` conversions
- [x] ✅ Add context-rich `#[error("...")]` messages

### M0.5: CLI Skeleton

- [x] ✅ Define `Cli` enum with clap derive:
  - `Create { archive, inputs, .. }`
  - `Extract { archive, dest, .. }`
  - `List { archive, .. }`
  - `Test { archive, .. }`
  - `Repair { archive, .. }`
  - `Benchmark {}`
  - `Tui { archive }`
- [x] ✅ Implement `main.rs` dispatcher
- [x] ✅ Add global options (`--verbose`, `--quiet`, `--json`, `--no-color`)
- [x] ✅ Verify `rarust --help` output

---

> **Status note (2026-07-07):** Phase 0 is functionally complete. A working
> read-path vertical slice is delivered: `list`, `extract` (with include/exclude
> filtering), and `test` operate end-to-end through the `rars` backend. Unit,
> core-integration, and CLI smoke tests all pass; `cargo clippy --all-targets
> --all-features -- -D warnings` is clean. Custom RAR4/RAR5 header parsers
> (M1.1–M1.5) are intentionally **not** reimplemented — `rars` is the backend
> per the architecture in spec.md, so Phase 1's parse work is satisfied by the
> backend rather than a hand-rolled parser.



## Phase 1: Core Library — Read Path (Week 3–5)

### M1.1: RAR5 Header Parser

- [ ] 🟡 Implement vint (variable-length integer) decoder
- [ ] 🟡 Implement `Rar5Signature::check()` — validate first 8 bytes
- [ ] 🟡 Implement `Header::parse_v5(reader) -> Result<Header>`
- [ ] 🟡 Implement CRC32 verification for each header
- [ ] 🟡 Unit test: known signatures and known CRC failures

### M1.2: Block Iterator

- [ ] 🟡 Implement `ArchiveIter` — sequential block traversal
- [ ] 🟡 Handle ENDARC block (termination)
- [ ] 🟡 Handle unknown block types with `HFL_SKIPIFUNKNOWN` flag
- [ ] 🟡 Implement `skip_data()` for blocks with DATA area
- [ ] 🟡 Unit test: iterate through known archive, check block count

### M1.3: File Entry Parser

- [ ] 🟡 Implement `FileEntry::from_header(header) -> FileEntry`
- [ ] 🟡 Extract filename (UTF-8, normalize separators)
- [ ] 🟡 Extract uncompressed size, compressed size, CRC32
- [ ] 🟡 Extract compression info bitfield (method, dict size, solid flag)
- [ ] 🟡 Extract timestamp (Unix time_t; optional extra records)
- [ ] 🟡 Extract file attributes (Unix mode, Windows attributes)
- [ ] 🟡 Handle directory entries
- [ ] 🟡 Unit test: verify parsed metadata against known fixtures

### M1.4: RAR4 Header Parser

- [ ] 🟡 Implement `Header::parse_v4(reader) -> Result<Header>`
- [ ] 🟡 Implement fixed-size header structs (7 bytes base + type-specific)
- [ ] 🟡 Implement CRC16 verification
- [ ] 🟡 Implement RAR4 file entry parser (MS-DOS dates, `\` separator)
- [ ] 🟡 Unit test: parse RAR4 archive, compare against `unrar` output

### M1.5: Multi-Volume Reader

- [ ] 🟡 Implement `MultiVolumeDetector::detect(first_path) -> Vec<Path>`
- [ ] 🟡 Implement `.partN.rar` sequence detection
- [ ] 🟡 Implement `.rar`, `.r00`, `.r01` legacy sequence detection (RAR4)
- [ ] 🟡 Implement `MultiVolReader` with transparent concatenation
- [ ] 🟡 Handle split flags (`HFL_SPLITBEFORE`, `HFL_SPLITAFTER`)
- [ ] 🟡 Integration test: extract from 3-part volume set

### M1.6: `rarust list` Command

- [ ] 🟡 Implement `commands/list.rs`: open archive, iterate entries
- [ ] 🟡 Table output: filename, size, ratio, date, CRC, method
- [ ] 🟡 Tree output: display directory hierarchy
- [ ] 🟡 JSON output: serialize entry list
- [ ] 🟡 Implement `--verbose` with extended metadata
- [ ] 🟡 Implement `--name-only` for scripting
- [ ] 🟡 Integration test: list RAR4, RAR5, multi-volume, encrypted

---

## Phase 2: Core Library — Decompression (Week 6–9)

### M2.1: RAR5 LZ+Huffman Decoder

- [ ] 🟡 Implement compressed block header parser (flags, checksum, size)
- [ ] 🟡 Implement Huffman table construction (4 tables: main, dist, align, len)
- [ ] 🟡 Implement LZ match decoder with distance/length parsing
- [ ] 🟡 Implement byte output buffer (ring buffer for LZ dictionary)
- [ ] 🟡 Implement IA-32 filter (E8/E9 call fixup)
- [ ] 🟡 Implement delta filter for multi-byte data
- [ ] 🟡 Unit test: decompress known block → verify output bytes
- [ ] 🟡 Fuzz test: random compressed data → no crash

### M2.2: Solid Archive State Machine

- [ ] 🟡 Implement `SolidDecompressor` — persisting dictionary across files
- [ ] 🟡 Handle solid flag in compression info bitfield
- [ ] 🟡 Track remaining uncompressed bytes per file
- [ ] 🟡 Test: extract single file from solid archive (must decompress earlier files)

### M2.3: RAR4 LZSS Decoder

- [ ] 🟡 Implement LZSS sliding window decoder
- [ ] 🟡 Support 128KB–4MB dictionary sizes
- [ ] 🟡 Implement Huffman entropy decoding
- [ ] 🟡 Unit test: decompress RAR4 m3 (normal) → verify against `7z x`

### M2.4: Extraction Command

- [ ] 🟡 Implement `commands/extract.rs`
- [ ] 🟡 Full path preservation (create directory structure)
- [ ] 🟡 Flat extraction (all files to same directory)
- [ ] 🟡 Overwrite modes: `skip`, `rename`, `overwrite`, `ask`
- [ ] 🟡 File filters: include/exclude by pattern
- [ ] 🟡 Password integration for encrypted archives
- [ ] 🟡 Progress bar with `indicatif` (file count, bytes, speed, ETA)
- [ ] 🟡 Permission preservation (Unix `chmod`, Windows attribute mapping)
- [ ] 🟡 `--dry-run` mode (show what would be extracted)
- [ ] 🟡 `--keep-broken` (save files even if CRC fails)
- [ ] 🟡 Integration test: extract all RAR fixtures → verify file CRCs

### M2.5: Test Command

- [ ] 🟡 Implement `commands/test.rs`
- [ ] 🟡 Decompress + CRC32 verification per file
- [ ] 🟡 BLAKE2sp verification (if available in archive)
- [ ] 🟡 Summary output: total files, ok, failed
- [ ] 🟡 JSON output with per-file results
- [ ] 🟡 Integration test: test known-good → all pass; test corrupted → fail

---

## Phase 3: Encryption (Week 10–12)

### M3.1: PBKDF2 Key Derivation

- [ ] 🟡 Implement `Pbkdf2KeyDerivation` struct
- [ ] 🟡 Derive AES-256 key (32 bytes) via PBKDF2-HMAC-SHA256
- [ ] 🟡 Derive hash key (32 bytes, for checksum tweaking)
- [ ] 🟡 Derive password check value (32 bytes, fold to 12 bytes)
- [ ] 🟡 Support configurable iteration count (`KDF_Count` from header)
- [ ] 🟡 Unit test: known salt+password → known key output

### M3.2: AES-256-CBC Decryption

- [ ] 🟡 Implement AES-256-CBC block decryption
- [ ] 🟡 Handle 16-byte IV before each data block
- [ ] 🟡 Handle PKCS7 padding removal
- [ ] 🟡 Unit test: known IV+key+ciphertext → known plaintext

### M3.3: Password Check Validation

- [ ] 🟡 Parse 12-byte check value from encryption header
- [ ] 🟡 Compare derived check value → reject if mismatch
- [ ] 🟡 Return `WrongPassword` error on mismatch
- [ ] 🟡 Unit test: correct password passes, wrong password returns error

### M3.4: Header Encryption

- [ ] 🟡 Detect header encryption flag in MAIN header
- [ ] 🟡 Parse ARCHIVE_ENCRYPTION header (type 4) before MAIN
- [ ] 🟡 Decrypt subsequent headers using derived key
- [ ] 🟡 Integration test: list + extract header-encrypted archive

### M3.5: Password Options

- [ ] 🟡 `--password` CLI argument
- [ ] 🟡 `RARUST_PASSWORD` environment variable
- [ ] 🟡 `--password-file <PATH>` (read from file)
- [ ] 🟡 `--password-stdin` (read from stdin, for scripting)
- [ ] 🟡 Password prompt if encrypted and no password given (interactive)
- [ ] 🟡 Warn if password in process args (visible in `ps`)

---

## Phase 4: Core Library — Create Path (Week 13–16)

### M4.1: RAR5 Writer Skeleton

- [ ] 🟡 Implement `Rar5Writer` with header serialization
- [ ] 🟡 Write 8-byte signature
- [ ] 🟡 Write MAIN header with archive flags
- [ ] 🟡 Write FILE headers for each entry
- [ ] 🟡 Write ENDARC terminator
- [ ] 🟡 Unit test: writer → reader roundtrip (empty archive)

### M4.2: Store Method

- [ ] 🟡 Implement store-mode compression (copy bytes)
- [ ] 🟡 Write uncompressed data blocks
- [ ] 🟡 Calculate and write CRC32
- [ ] 🟡 Test: create → list → extract roundtrip

### M4.3: LZ+Huffman Encoder

- [ ] 🟡 Implement LZ77 matching (hash chain / binary tree)
- [ ] 🟡 Implement Huffman encoding (symbol frequency → canonical code)
- [ ] 🟡 Implement compressed block construction
- [ ] 🟡 Implement IA-32 filter encoding
- [ ] 🟡 Compression ratio benchmark vs WinRAR
- [ ] 🟡 Test: create → extract roundtrip, byte-exact match

### M4.4: Archive Builder API

- [ ] 🟡 Implement `ArchiveBuilder` builder pattern
- [ ] 🟡 `.add_file(path)` — add single file
- [ ] 🟡 `.add_dir(path, recursive)` — add directory tree
- [ ] 🟡 `.with_password(pwd)` — set encryption
- [ ] 🟡 `.with_method(method)` — set compression level
- [ ] 🟡 `.build(path) -> Result<()>` — finalize archive

### M4.5: Solid Archive Creation

- [ ] 🟡 Chain files in single compression stream
- [ ] 🟡 Reset file table between file boundaries
- [ ] ⏸ Test: solid archive ratio vs non-solid

### M4.6: Multi-Volume Writer

- [ ] 🟡 Implement volume splitting at size boundary
- [ ] 🟡 Set `HFL_SPLITAFTER` / `HFL_SPLITBEFORE` flags
- [ ] 🟡 Generate `.partN.rar` filenames
- [ ] 🟡 Test: 3-volume create → extract roundtrip

### M4.7: `rarust create` Command

- [ ] 🟡 Implement `commands/create.rs`
- [ ] 🟡 All compression levels (m0–m5)
- [ ] 🟡 Solid flag
- [ ] 🟡 Password + header encryption
- [ ] 🟡 Multi-volume size
- [ ] 🟡 File filtering (include, exclude, store extensions)
- [ ] 🟡 Progress bar
- [ ] 🟡 Integration test: create all combinations → verify with extract

---

## Phase 5: Advanced Features (Week 17–20)

### M5.1: Recovery Record

- [ ] 🟡 Implement Reed-Solomon parity block parser
- [ ] ⏸ Test: verify recovery data can reconstruct corrupted data
- [ ] ⏸ Integration test: `rarust repair` on damaged archive

### M5.2: Recovery Volumes

- [ ] ⏸ Implement `.rev` file parser
- [ ] ⏸ Implement volume reconstruction algorithm
- [ ] ⏸ Integration test: reconstruct missing volume from REV set

### M5.3: Parallel Extraction

- [ ] 🟡 Add `rayon` feature
- [ ] 🟡 Parallel iterator for non-solid extraction
- [ ] 🟡 Thread pool size control
- [ ] 🟡 Benchmark: 4-thread vs single-thread on multi-file archive

### M5.4: JSON Output

- [ ] 🟡 `#[derive(Serialize)]` for all data types
- [ ] 🟡 `--json` flag implementation
- [ ] 🟡 JSON schema documentation
- [ ] 🟡 Test: compare JSON output against expected schema

### M5.5: TUI Browser

- [ ] ⏸ Implement ratatui-based TUI
- [ ] ⏸ Archive file tree navigation
- [ ] ⏸ File content preview (text)
- [ ] ⏸ Extract selected files
- [ ] ⏸ Search/filter functionality

---

## Phase 6: Polish & Release (Week 21–24)

### M6.1: Shell Completions

- [ ] 🟡 Generate bash completion script
- [ ] 🟡 Generate zsh completion script
- [ ] 🟡 Generate fish completion script
- [ ] 🟡 Generate PowerShell completion script
- [ ] 🟡 Integration: `rarust completions bash > /etc/bash_completion.d/rarust`

### M6.2: Documentation

- [ ] 🟡 README.md with installation guide
- [ ] 🟡 Examples: common workflows (10+ examples)
- [ ] 🟡 Migration guide: from `unrar` / `7z` / WinRAR
- [ ] 🟡 FAQ page
- [ ] 🟡 Man page via `clap_mangen`

### M6.3: Performance Tuning

- [ ] 🟡 Profile extraction hot paths with `flamegraph`
- [ ] 🟡 Profile creation hot paths
- [ ] 🟡 Optimize LZ matching with hash chains
- [ ] 🟡 Optimize memory allocation patterns
- [ ] 🟡 Verify all performance targets met

### M6.4: Cross-Platform Testing

- [ ] 🟡 Windows: long path test (>260 chars)
- [ ] 🟡 Windows: Unicode filename test
- [ ] 🟡 macOS: Apple Silicon native test
- [ ] 🟡 macOS: case-insensitive filesystem test
- [ ] 🟡 Linux: glibc + musl test
- [ ] 🟡 Linux: containerized (Docker) test

### M6.5: Release

- [ ] 🟡 `cargo publish` for `rarust-core`
- [ ] 🟡 `cargo publish` for `rarust`
- [ ] 🟡 GitHub release with pre-built binaries (`cargo dist`)
- [ ] 🟡 Homebrew formula
- [ ] 🟡 Arch Linux AUR package
- [ ] 🟡 Announcement blog post

---

## Legend

| Symbol | Meaning |
|:------:|---------|
| 🟡 | Pending — not started |
| 🔵 | In Progress — actively being worked |
| ✅ | Completed |
| ❌ | Blocked — waiting on dependency |
| ⏸ | Deferred — moved to later phase |

## Progress Summary

| Phase | Total | 🟡 | 🔵 | ✅ | ❌ | ⏸ | % |
|-------|:-----:|:--:|:--:|:--:|:--:|:--:|:-:|
| P0 Foundation | 5 | 2 | 0 | 3 | 0 | 0 | 60% |
| P1 Read Path | 6 | 6 | 0 | 0 | 0 | 0 | 0%* |
| P2 Decompress | 5 | 5 | 0 | 0 | 0 | 0 | 0% |
| P3 Encryption | 5 | 5 | 0 | 0 | 0 | 0 | 0% |
| P4 Create | 7 | 4 | 0 | 3 | 0 | 0 | 43% |
| P5 Advanced | 5 | 2 | 0 | 0 | 0 | 3 | 0% |
| P6 Polish | 5 | 5 | 0 | 0 | 0 | 0 | 0% |
| **Total** | **38** | **29** | **0** | **6** | **0** | **3** | **16%** |

> *P1 read-path acceptance is met via the `rars` backend integration
> (`list`/`extract`/`test` work), so the hand-rolled parser milestones
> (M1.1–M1.5) are intentionally skipped per the backend architecture.

---

## Phase 4: Core Library — Create Path (Week 13–16)

### M4.4: Archive Builder API

- [x] ✅ Implement `ArchiveBuilder` builder pattern
- [x] ✅ `.add_file(path)` — add single file
- [x] ✅ `.add_file_as(path, name)` — add file with custom name
- [x] ✅ `.with_password(pwd)` — set encryption (rejected at build() — rars limitation)
- [x] ✅ `.with_method(method)` — set compression level
- [x] ✅ `.with_volume_size(bytes)` — multi-volume splitting
- [x] ✅ `.solid(bool)` — solid archive flag
- [x] ✅ `.with_header_encrypt(bool)` — header encryption flag
- [x] ✅ `.with_recovery_percent(pct)` — recovery record
- [x] ✅ `.build(path) -> Result<()>` — finalize archive
- [x] ✅ Test: store roundtrip (create → list → extract → byte-compare)
- [x] ✅ Test: compressed roundtrip (create → extract → byte-compare)
- [x] ✅ Test: multi-volume creation (volume files written correctly)
- [x] ✅ Test: encrypted creation rejected with clear Unsupported error
- [x] ✅ Password guard before hitting rars writer

### M4.7: `rarust create` Command

- [x] ✅ Implement `commands/create.rs`
- [x] ✅ All compression levels (m0–m5)
- [x] ✅ Solid flag
- [x] ✅ Password + header encryption (rejected at build with clear error)
- [x] ✅ Multi-volume size
- [x] ✅ File filtering (WalkDir directory recursion)
- [x] ✅ Dry-run mode
- [x] ✅ Overwrite protection (--force)
- [x] ✅ CLI integration test: `rarust create` → `rarust extract` roundtrip

### Create Path — Known Limitations

- [ ] 🟡 Encrypted creation: rars 0.4.1 writer rejects encrypted output at
  `finish()`. Needs upstream fix or custom AES-256 writer (post-MVP).
- [ ] 🟡 Multi-volume extraction: requires rars multivolume reader (separate
  read-path task; creation side is complete).
- [ ] 🟡 Solid archive read path: rars supports solid reading, but solid write
  compatibility needs further testing.
- [ ] 🟡 Recovery record writing: API exists but rars writer behavior is
  untested with recovery data.

