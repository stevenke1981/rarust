# Rarust — Implementation Plan

> 本計畫定義 Rarust 的開發階段、里程碑、時程與風險管理策略。
> 目標：6 個月內從零到可發布的 MVP。

---

## 1. Development Phases

### Phase 0: Foundation (Week 1–2)

**目標**：專案骨架、CI/CD 管線、測試框架、RAR 格式 research complete

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M0.1 | Rust workspace with `rarust` + `rarust-core` | `cargo build` + `cargo test` passes |
| M0.2 | CI pipeline (GitHub Actions) | Linux, macOS, Windows all green |
| M0.3 | RAR test fixture corpus | 20+ synthetic RAR files covering RAR4/RAR5 |
| M0.4 | Error type hierarchy | `thiserror` enums for all failure modes |
| M0.5 | CLI skeleton with clap derive | `rarust --help` shows all subcommands |

**Tasks**:
- [ ] Initialize cargo workspace
- [ ] Set up CI with matrix build (ubuntu, macos, windows)
- [ ] Generate test RAR files (WinRAR trial, `rars` CLI, 7-Zip)
- [ ] Define error types in `rarust-core`
- [ ] Scaffold clap CLI with all subcommands (stubs)
- [ ] Configure lints: clippy, rustfmt, deny(unsafe_code) in core

### Phase 1: Core Library — Read Path (Week 3–5)

**目標**：`rarust-core` 可以開啟 RAR4/RAR5 檔案、遍歷區塊、解析檔案條目

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M1.1 | RAR5 header parser | Parse 8-byte signature + MAIN header |
| M1.2 | Block iterator | Walk all blocks in archive, validate CRCs |
| M1.3 | File entry parser | Extract name, size, date, CRC from FILE headers |
| M1.4 | RAR4 header parser | Parse legacy 7-byte block headers |
| M1.5 | Multi-volume reader | Auto-detect and concatenate `.partN.rar` series |
| M1.6 | `rarust list` command | Full listing output (table & JSON) |

**Tasks**:
- [ ] Implement RAR5 `Header::parse(reader)` with vint decoding
- [ ] Implement `Iterator<Item = Header>` over archive
- [ ] Implement `FileEntry::parse()` with compression info bitfield
- [ ] Implement RAR4 block parser (fixed-size header structs)
- [ ] Implement `MultiVolReader` that chains volume files
- [ ] Implement `list` command in CLI layer
- [ ] Unit test: known CRC for each header type
- [ ] Integration test: list RAR5, RAR4, multi-volume

### Phase 2: Core Library — Decompression (Week 6–9)

**目標**：支援 RAR5 LZ+Huffman 與 RAR4 LZSS 解壓縮

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M2.1 | RAR5 LZ+Huffman decoder | Decompress RAR5 method 0–5 (stored + compressed) |
| M2.2 | Solid archive state machine | Persist dictionary across files in solid mode |
| M2.3 | RAR4 LZSS decoder | Decompress RAR4 methods 0x30–0x33 |
| M2.4 | `rarust extract` command | Extract files to disk with full path preservation |
| M2.5 | `rarust test` command | CRC/BLAKE2 verification after decompression |

**Tasks**:
- [ ] Implement Huffman table construction from compressed block metadata
- [ ] Implement LZ match decoder with distance/length parsing
- [ ] Implement byte output ring buffer for solid archives
- [ ] Implement RAR4 LZSS sliding window decoder
- [ ] Implement `FileDecoder` trait with RAR4/RAR5 implementations
- [ ] Implement extraction pipeline: read → decrypt → decompress → write
- [ ] Implement `test` command: decompress + verify CRC
- [ ] Performance benchmark: 1GB extraction speed target > 200 MB/s

### Phase 3: Encryption (Week 10–12)

**目標**：支援 AES-256-CBC RAR5 加密/解密

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M3.1 | PBKDF2-HMAC-SHA256 KDF | Derive AES key from password + salt |
| M3.2 | AES-256-CBC decrypt | Decrypt archive data blocks |
| M3.3 | Password check validation | Verify 12-byte password check field |
| M3.4 | Header encryption | Decrypt file headers (hide filenames) |
| M3.5 | Encrypted archive commands | `list`, `extract`, `test` with `-p` flag |

**Tasks**:
- [ ] Implement `Pbkdf2KeyDerivation::derive(password, salt, iterations)`
- [ ] Implement `Aes256CbcDecryptor` with AES-CBC mode
- [ ] Implement password check verification (12-byte check value)
- [ ] Integrate decryption into extraction pipeline
- [ ] Implement `--password` option: CLI arg, env var, file, stdin
- [ ] Implement `zeroize`-based secure memory clearing
- [ ] Test: decrypt known-correct encrypted RAR5 fixtures

### Phase 4: Core Library — Create Path (Week 13–16)

**目標**：支援 RAR5 封存建立

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M4.1 | RAR5 writer skeleton | Write valid MAIN header + FILE headers |
| M4.2 | Store method writer | Copy files into archive without compression |
| M4.3 | LZ+Huffman encoder | Compress data streams |
| M4.4 | Archive builder API | High-level API: `ArchiveBuilder::new().add_file()` |
| M4.5 | Solid archive creation | Multi-file solid compression |
| M4.6 | Multi-volume writer | Split archive across `.partN.rar` files |
| M4.7 | `rarust create` command | Full create flow with all options |

**Tasks**:
- [ ] Implement `Rar5Writer` with header serialization
- [ ] Implement vint encoding for header fields
- [ ] Implement store-mode data copy
- [ ] Implement LZ+Huffman encoder (matching RAR5 decoder spec)
- [ ] Implement `ArchiveBuilder` with typed builder pattern
- [ ] Implement solid archive: chain files in single compression stream
- [ ] Implement multi-volume splitter: size-based boundary splitting
- [ ] Implement `create` command in CLI layer
- [ ] Create-extract-roundtrip test: all methods, solid, multi-volume

### Phase 5: Advanced Features (Week 17–20)

**目標**：修復、復原記錄、平行處理、TUI

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M5.1 | Recovery record read | Parse RS error correction data |
| M5.2 | Archive repair | Rebuild damaged archive from recovery record |
| M5.3 | Recovery volumes | Reconstruct missing volumes from `.rev` files |
| M5.4 | Parallel extraction | Rayon-based multi-threaded decompression |
| M5.5 | JSON output | `--json` flag for all subcommands |
| M5.6 | TUI browser | ratatui-based interactive archive browser |

**Tasks**:
- [ ] Implement Reed-Solomon recovery block parser
- [ ] Implement repair algorithm: locate damage, rebuild from RS
- [ ] Implement REV volume reconstruction
- [ ] Add `rayon` parallel iterator for non-solid extraction
- [ ] Implement `serde::Serialize` for all output types
- [ ] Build ratatui TUI: file tree, preview, progress

### Phase 6: Polish & Release (Week 21–24)

**目標**：穩定化、文件、效能調校、跨平台測試

| Milestone | Deliverable | Verification |
|-----------|-------------|--------------|
| M6.1 | Shell completions | bash, zsh, fish, PowerShell |
| M6.2 | Man page | `rarust --help` → man page generation |
| M6.3 | Performance tuning | Meet all performance targets |
| M6.4 | Cross-platform testing | Windows, macOS, Linux full test pass |
| M6.5 | Documentation | README, examples, FAQ |
| M6.6 | v1.0.0-rc.1 release | crates.io + GitHub releases |

**Tasks**:
- [ ] Add clap `complete` subcommand for shell completions
- [ ] Generate man page from clap help (clap_mangen crate)
- [ ] Profile and optimize hot paths (flamegraph, perf)
- [ ] Full CI matrix test on all 3 platforms
- [ ] Write README with installation instructions and examples
- [ ] Write migration guide from WinRAR/unrar/7z
- [ ] Publish to crates.io
- [ ] Create GitHub release with pre-built binaries

---

## 2. Gantt Chart

```
Phase          | W1-2 | W3-5 | W6-9 | W10-12 | W13-16 | W17-20 | W21-24 |
P0 Foundation  | ████ |      |      |        |        |        |        |
P1 Read Path   |      | ██████|      |        |        |        |        |
P2 Decompress  |      |      | ████████|      |        |        |        |
P3 Encryption  |      |      |      | ██████ |        |        |        |
P4 Create      |      |      |      |        | ████████|        |        |
P5 Advanced    |      |      |      |        |        | ████████|        |
P6 Polish      |      |      |      |        |        |        | ████████|
```

---

## 3. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|:-----------:|:------:|------------|
| RAR5 decompression bugs | Medium | High | Ship with extensive test fixtures; validate against WinRAR output |
| Compression ratio suboptimal | Medium | Medium | Target correctness first, optimize ratio later; document limitations |
| Legal concerns (clean-room) | Low | High | Use MIT-licensed `rars` as backend; never reference UnRAR source |
| `rars` crate becomes unmaintained | Low | Medium | Fork critical code; maintain own patchset |
| Cross-platform path bugs | Medium | Medium | Heavy CI testing on all 3 platforms; use `dunce` for path normalization |
| PPMd implementation complexity | High | Medium | Defer RAR4 PPMd (methods 4-5) to post-MVP; RAR4 LZSS only for v1.0 |
| Reed-Solomon complexity | High | Medium | Defer recovery record to Phase 5; verify against WinRAR `.rev` output |
| Memory consumption with large dict | Medium | Medium | Cap dictionary at 64MB by default; configurable via `--memory-limit` |

### 3.1 Risk Mitigation Matrix

| Risk Area | Fallback Strategy |
|-----------|------------------|
| `rars` crate breaks | Pin exact version in Cargo.lock, fork to own org |
| RAR5 compression bugs | Fuzz testing: AFL/LibFuzzer for decompression |
| Cross-platform file issues | Beta testers on each platform before release |
| Performance below target | Profile-guided optimization, SIMD via `portable_simd` |
| Feature incomplete by deadline | Ship MVP with core features; mark advanced features as preview |

---

## 4. Resource Requirements

### 4.1 Development Environment

| Tool | Purpose |
|------|---------|
| Rust 1.85+ (Edition 2024) | Primary language |
| cargo-nextest | Parallel test runner |
| cargo-fuzz / cargo-afl | Fuzz testing |
| cargo-flamegraph | Performance profiling |
| cargo-deny | License compliance |
| cargo-dist | Binary release automation |
| typos-cli | Spelling checks in code |

### 4.2 Test Infrastructure

- GitHub Actions: Linux (ubuntu-latest), macOS (macos-latest), Windows (windows-latest)
- RAR test file corpus: ~50 MB of synthetic + edge-case archives
- Benchmark harness: `cargo bench` with `criterion`

### 4.3 Reference Materials (Legal Boundary)

The implementation must be **clean-room**:
- ✅ **Permitted**: RARLAB technote, Kaitai Struct spec, LoC format descriptions
- ✅ **Permitted**: `rars` source code (MIT/Apache-2.0)
- ✅ **Permitted**: 7-Zip RAR5 handler (LGPL, independently developed)
- ❌ **Not permitted**: UnRAR C++ source code (restrictive UnRAR license)
- ❌ **Not permitted**: Reverse engineering of WinRAR binary

---

## 5. Release Criteria

### MVP (v1.0.0)

- [ ] List RAR4 + RAR5 archives ✅
- [ ] Extract RAR4 + RAR5 with path preservation ✅
- [ ] Extract encrypted RAR5 (AES-256) ✅
- [ ] Create RAR5 archives (store + compressed) ✅
- [ ] Multi-volume create + extract ✅
- [ ] Solid archive create + extract ✅
- [ ] Test integrity (CRC verification) ✅
- [ ] Progress bars for all operations ✅
- [ ] JSON output for all commands ✅
- [ ] Cross-platform: Windows, macOS, Linux ✅
- [ ] All unit + integration tests passing ✅
- [ ] No `unsafe` code in `rarust-core` ✅

### Post-MVP (v1.1+)

- [ ] Recovery record + archive repair
- [ ] Recovery volumes (.rev)
- [ ] Parallel extraction
- [ ] TUI archive browser
- [ ] Benchmark command
- [ ] Archive comments
- [ ] RAR4 encryption (AES-128)
- [ ] Shell completions
- [ ] SFX support
