# Rarust — Test Strategy

> 測試策略定義 Rarust 的品質驗證方法，涵蓋單元測試、整合測試、模糊測試、效能基準測試與跨平台驗證。

---

## 1. Test Philosophy

| 原則 | 說明 |
|------|------|
| **正確性優先** | 每個格式解析、壓縮/解壓縮操作必須與 WinRAR 位元組完全一致 |
| **防禦性測試** | 不合法／損壞的輸入不能造成 panic、安全漏洞或無限循環 |
| **可再現性** | 所有測試使用確定性 fixture，CI 每次執行結果一致 |
| **漸進式覆蓋** | MVP 先覆蓋 P0 功能路徑，後續逐步擴充邊界與錯誤案例 |

---

## 2. Test Levels

```
┌──────────────────────────────────────────────┐
│              E2E / CLI Tests                   │
│  (full command invocation, exit codes, stdout) │
├──────────────────────────────────────────────┤
│           Integration Tests                    │
│  (archive open → list → extract → verify)      │
├──────────────────────────────────────────────┤
│            Unit Tests                          │
│  (parser, decoder, encoder, KDF, each module)  │
├──────────────────────────────────────────────┤
│          Fuzz Tests (libfuzzer)               │
│  (random/corrupted input → no crash)           │
├──────────────────────────────────────────────┤
│       Benchmarks (criterion)                  │
│  (throughput, ratio, memory, regression)       │
└──────────────────────────────────────────────┘
```

### 2.1 Unit Tests

| Scope | Location | Framework | Target Coverage |
|-------|----------|-----------|-----------------|
| Header parsing | `rarust-core/src/*.rs` (inline `#[cfg(test)]`) | `#[test]` | 90%+ lines |
| Vint encoding/decoding | `rarust-core/src/util.rs` | `#[test]` | 100% edge cases |
| CRC32/CRC16 verification | `rarust-core/src/*.rs` | `#[test]` | 100% known values |
| Compression method decode | `rarust-core/src/entry.rs` | `#[test]` | All method IDs |
| Password KDF | `rarust-core/src/encryption.rs` | `#[test]` | Known-answer tests |
| AES-256-CBC | `rarust-core/src/encryption.rs` | `#[test]` | NIST test vectors |
| Huffman tree construction | `rarust-core/src/...` | `#[test]` | All table types |
| LZ match decoder | `rarust-core/src/...` | `#[test]` | Edge distances/lengths |

#### Naming Convention

```rust
#[test]
fn test_parse_rar5_main_header_valid() { ... }
#[test]
fn test_parse_rar5_main_header_truncated() { ... }
#[test]
fn test_vint_encode_max_64bit() { ... }
#[test]
fn test_vint_decode_overflow() { ... }
#[test]
fn test_aes256_cbc_nist_vector() { ... }
```

### 2.2 Integration Tests

| Test Suite | File | Scope |
|------------|------|-------|
| Extract tests | `tests/integration/extract_test.rs` | RAR4/RAR5 full, selected, flat, with paths |
| Create tests | `tests/integration/create_test.rs` | All compression levels, solid, store |
| List tests | `tests/integration/list_test.rs` | Table, JSON, tree, name-only, columns |
| Multi-volume tests | `tests/integration/multi_test.rs` | Create + extract 3-volume sets |
| Password tests | `tests/integration/password_test.rs` | Encrypted, header-encrypted, wrong pwd |
| Repair tests | `tests/integration/repair_test.rs` | Recovery record, REV reconstruction |
| Roundtrip tests | `tests/integration/roundtrip_test.rs` | Create → list → extract → byte-compare |

#### Integration Test Pattern

```rust
#[test]
fn test_extract_rar5_normal() -> Result<()> {
    // Arrange
    let fixture = Fixture::open("tests/fixtures/rar5-normal.rar")?;
    let tmp = TempDir::new()?;

    // Act
    let status = rarust_main(&[
        "rarust", "extract",
        fixture.path(),
        "-o", "overwrite",
        "--no-progress",
        tmp.path().to_str().unwrap(),
    ])?;

    // Assert
    assert_eq!(status.code(), Some(0));
    assert!(tmp.path().join("document.pdf").exists());
    assert_eq!(
        Sha256::digest(&fs::read(tmp.path().join("document.pdf"))?),
        fixture.expected_sha256("document.pdf")
    );
    Ok(())
}
```

### 2.3 Fuzz Tests

| Target | Harness | Corpus | Iterations (CI) |
|--------|---------|--------|------------------|
| RAR5 header parser | `cargo fuzz` — `fuzz_rar5_header` | Valid + mutated RAR5 headers | 100k |
| RAR4 header parser | `cargo fuzz` — `fuzz_rar4_header` | Valid + mutated RAR4 headers | 100k |
| Decompression (RAR5) | `cargo fuzz` — `fuzz_decompress_rar5` | Valid RAR5 + bit-flipped | 500k |
| Decompression (RAR4) | `cargo fuzz` — `fuzz_decompress_rar4` | Valid RAR4 + bit-flipped | 500k |
| Multi-volume reader | `cargo fuzz` — `fuzz_multivol` | Truncated/reordered volumes | 50k |
| Encryption KDF | `cargo fuzz` — `fuzz_kdf` | Random salt + password | 100k |

**Fuzz Test Requirements**:

- All fuzz targets must run **24h+** before v1.0.0 release
- CI runs each target for **5 minutes** per push
- Crash findings must produce a minimal reproducer
- Regression tests added for each fixed crash

### 2.4 Benchmarks

| Benchmark | File | Metric | Target |
|-----------|------|--------|--------|
| Extract throughput (non-solid) | `benches/extraction.rs` | MB/s | > 200 MB/s (compressed) |
| Extract throughput (solid) | `benches/extraction.rs` | MB/s | > 100 MB/s |
| Create throughput (m3 normal) | `benches/creation.rs` | MB/s | > 80 MB/s |
| Create throughput (m5 best) | `benches/creation.rs` | MB/s | > 40 MB/s |
| List 10k-entry archive | `benches/listing.rs` | ms | < 200 ms |
| Memory usage (default dict) | `benches/memory.rs` | MB | < 256 MB |
| CRC32 verification | `benches/hashing.rs` | MB/s | > 2 GB/s |
| Multi-volume concatenation | `benches/multivol.rs` | MB/s | > 1 GB/s |

**Benchmark Environment**:

- CPU: x86_64 with AVX2 (minimum); Apple M-series for aarch64
- Memory: 16 GB+ RAM
- OS: Ubuntu 24.04 (primary), Windows 11, macOS 15
- Rust: `cargo bench` with `criterion` statistical harness
- Baselines: stored in `benches/baselines/` for regression detection

---

## 3. Test Fixtures

### 3.1 Fixture Corpus

| Fixture | Type | Size | Purpose |
|---------|------|:----:|---------|
| `rar5-empty.rar` | RAR5 | 28 B | Empty archive (only MAIN + ENDARC) |
| `rar5-single-file.rar` | RAR5 | 1 KB | One text file, store method |
| `rar5-compressed.rar` | RAR5 | 10 KB | Multiple files, m3 normal |
| `rar5-best.rar` | RAR5 | 10 KB | m5 best compression |
| `rar5-solid.rar` | RAR5 | 10 KB | Solid archive, 5 files |
| `rar5-encrypted.rar` | RAR5 | 5 KB | AES-256, `password123` |
| `rar5-header-encrypted.rar` | RAR5 | 5 KB | Header encryption |
| `rar5-multivol.part1.rar` | RAR5 | 5 KB | 3-volume set, part 1 |
| `rar5-multivol.part2.rar` | RAR5 | 5 KB | 3-volume set, part 2 |
| `rar5-multivol.part3.rar` | RAR5 | 5 KB | 3-volume set, part 3 |
| `rar5-unicode.rar` | RAR5 | 2 KB | CJK + emoji filenames |
| `rar5-large-dict.rar` | RAR5 | 50 KB | 64MB dictionary |
| `rar5-recovery.rar` | RAR5 | 10 KB | With recovery record (5%) |
| `rar5-corrupted.rar` | RAR5 | 5 KB | Byte-flipped data block |
| `rar4-empty.rar` | RAR4 | 25 B | Empty archive |
| `rar4-compressed.rar` | RAR4 | 10 KB | Multiple files, m3 normal |
| `rar4-encrypted.rar` | RAR4 | 5 KB | AES-128, `password123` |
| `rar4-solid.rar` | RAR4 | 10 KB | Solid archive |
| `rar4-multivol.r00` | RAR4 | 5 KB | Legacy volume naming |
| `rar4-multivol.r01` | RAR4 | 5 KB | Legacy volume naming |
| `rar4-multivol.rar` | RAR4 | 5 KB | Legacy volume naming |
| `rar4-unicode.rar` | RAR4 | 2 KB | Unicode filenames |

### 3.2 Fixture Generation

Fixtures are generated by a **reproducible script** (`tests/fixtures/generate.ps1` / `tests/fixtures/generate.sh`):

```bash
#!/usr/bin/env bash
# Usage: ./generate.sh [--force]
# Requires: WinRAR CLI (trial), 7z, python3

# Generate RAR5 compressed archive
rar a -m3 -ep1 "$FIXTURES/rar5-compressed.rar" "$SRC/"*

# Generate RAR5 encrypted archive
rar a -m3 -hp"$PASSWORD" "$FIXTURES/rar5-encrypted.rar" "$SRC/"*

# Generate manifest with SHA256 expected values
python3 ../tools/generate_manifest.py "$FIXTURES"
```

**Fixture Integrity**: Each fixture includes a `.manifest.json` with:

```json
{
  "rar5-compressed.rar": {
    "sha256": "a1b2c3d4e5...",
    "format": "RAR5",
    "entries": [
      {"path": "file1.txt", "size": 1024, "sha256": "f6e5d4c3b2..."},
      {"path": "subdir/file2.txt", "size": 2048, "sha256": "a9b8c7d6e5..."}
    ]
  }
}
```

---

## 4. CI Test Pipeline

```yaml
# .github/workflows/test.yml (conceptual)
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        features: ["default", "parallel", "tui", "full-hash"]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --features ${{ matrix.features }}
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo fmt --check

  fuzz:
    runs-on: ubuntu-latest
    steps:
      - run: cargo fuzz run fuzz_rar5_header -- -max_total_time=300
      - run: cargo fuzz run fuzz_decompress_rar5 -- -max_total_time=300

  benchmark:
    runs-on: ubuntu-latest
    steps:
      - run: cargo bench -- --threshold 5  # fail if >5% regression
```

### 4.1 CI Gates

| Gate | Command | Must Pass For |
|------|---------|---------------|
| Build | `cargo build --all-targets` | Every PR |
| Lint | `cargo clippy -- -D warnings` | Every PR |
| Format | `cargo fmt --check` | Every PR |
| Unit tests | `cargo test --lib` | Every PR |
| Integration | `cargo test --test integration/*` | Every PR |
| Fuzz (short) | `cargo fuzz run ... -max_total_time=300` | Nightly / Release |
| Benchmark | `cargo bench -- --threshold 5` | Release branch |
| Security audit | `cargo audit` | Every PR |
| License check | `cargo deny check licenses` | Every PR |
| `unsafe` check | `grep -r 'unsafe' rarust-core/src/` | Every PR |

---

## 5. Test Coverage Strategy

### 5.1 Coverage Targets

| Module | Line Coverage | Branch Coverage | Notes |
|--------|:------------:|:--------------:|-------|
| `rarust-core/src/archive.rs` | 90% | 85% | Integration tests cover most paths |
| `rarust-core/src/entry.rs` | 95% | 90% | All compression methods tested |
| `rarust-core/src/error.rs` | 100% | — | Simple enum |
| `rarust-core/src/encryption.rs` | 95% | 90% | KATs cover all branches |
| `rarust-core/src/recovery.rs` | 80% | 75% | RS complex; deferred |
| `rarust-core/src/multi.rs` | 90% | 85% | |
| `rarust/src/commands/*.rs` | 85% | 80% | CLI integration tests cover |
| `rarust/src/output/*.rs` | 80% | 75% | Formatting code |

### 5.2 Coverage Measurement

```bash
# Install
cargo install cargo-tarpaulin

# Run
cargo tarpaulin --out html --output-dir coverage/
```

Coverage reports generated on **nightly CI** and published to GitHub Pages.

---

## 6. Regression Test Strategy

### 6.1 Regression Triggers

- Any bug fix must include a regression test that reproduces the bug before the fix
- Any crash found by fuzzing must include the crash input as a regression fixture
- Any user-reported issue on a supported format must include a minimized reproducer

### 6.2 Regression Test Pattern

```rust
#[test]
fn test_regression_issue_42_truncated_header() {
    // Minimal reproducer: 5-byte truncated RAR5 MAIN header
    let data = b"\x52\x61\x72\x21\x1a\x07\x01\x00\x02\x00\x00";
    let result = Header::parse_v5(&data[..]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RarustError::Format(_)));
}
```

---

## 7. Manual Test Plan (Pre-Release)

Before v1.0.0-rc.1, perform manual testing on:

| Scenario | Windows | macOS | Linux |
|----------|:-------:|:-----:|:-----:|
| Extract 100+ file archive | ✅ | ✅ | ✅ |
| Create + extract roundtrip (1GB) | ✅ | ✅ | ✅ |
| Encrypted archive with `--password-stdin` | ✅ | ✅ | ✅ |
| Multi-volume (10 parts) | ✅ | ✅ | ✅ |
| Unicode filenames (CJK, emoji) | ✅ | ✅ | ✅ |
| Long paths (>260 chars) on Windows | ✅ | N/A | N/A |
| macOS NFD decomposed Unicode | N/A | ✅ | N/A |
| Permission preservation | ⚠️ Partial | ✅ | ✅ |
| Symlink handling | N/A | ✅ | ✅ |

---

## 8. Test Infrastructure

| Tool | Purpose | Required |
|------|---------|:--------:|
| `cargo test` (built-in) | Standard unit + integration | ✅ |
| `cargo nextest` | Parallel test runner (faster CI) | Recommended |
| `cargo fuzz` / `libfuzzer` | Fuzz testing | ✅ |
| `cargo criterion` | Statistical benchmarks | Recommended |
| `cargo tarpaulin` | Code coverage | CI nightly |
| `cargo audit` | Security vulnerability scan | CI |
| `cargo deny` | License + advisory check | CI |
| `cargo flamegraph` | Performance profiling | Dev |
| `Valgrind` (Linux) | Memory error detection | Pre-release |
| `rr` (Linux) | Reversible debugging | Dev |

---

## 9. Test Documentation

Each test file must include a header comment:

```rust
//! Integration tests for RAR5 encrypted archive extraction.
//!
//! Fixtures used:
//! - `rar5-encrypted.rar`: AES-256, password "password123"
//! - `rar5-header-encrypted.rar`: Header encryption, password "test"
//!
//! Expected behavior:
//! - Correct password → successful extraction
//! - Wrong password → Error::WrongPassword
//! - No password provided → Error::WrongPassword
```

---

## 10. Acceptance Criteria

| Criterion | Verification |
|-----------|-------------|
| All unit tests pass | `cargo test --lib` exit 0 |
| All integration tests pass | `cargo test --test integration/*` exit 0 |
| Fuzz targets run 5 min without crash | `cargo fuzz run ... -max_total_time=300` exit 0 |
| No regression in benchmarks | `cargo bench -- --threshold 5` exit 0 |
| No `unsafe` in `rarust-core` | `grep -r unsafe rarust-core/src/ | wc -l` = 0 |
| All target platforms build | CI matrix green |
| Code coverage ≥ 80% (core lib) | `cargo tarpaulin` report |

---

## 11. Current Verification (2026-07-07)

This section records the **actual** verification state as of the latest
working session. It supplements the planned strategy above.

### 11.1 Commands Under Test

| Command | Backend | Status |
|---------|---------|--------|
| `rarust list` | `rars` | ✅ Works (table / name-only / tree / JSON) |
| `rarust extract` | `rars` | ✅ Works (include/exclude filter, path-safe extraction) |
| `rarust test` | `rars` | ✅ Works (streams all entries to sink, counts OK/failed) |
| `rarust create` | `rars` (write API) | ✅ Works (store / compressed / multi-volume) |
| `rarust create --password` | — | ⏸ Rejected: rars 0.4.1 does not support encrypted writing |
| `rarust repair` | — | ⏸ Stub: returns `Unsupported` |
| `rarust benchmark` | — | ⏸ Stub: returns `Unsupported` |

### 11.2 Test Suites Executed

| Suite | Location | Result |
|-------|----------|--------|
| Unit tests (`rarust-core`) | `src/**` inline `#[cfg(test)]` | ✅ 11 passed |
| Core integration (`read_extract`) | `rarust-core/tests/read_extract.rs` | ✅ 3 passed |
| Core integration (`create_test`) | `rarust-core/tests/create_test.rs` | ✅ 4 passed |
| CLI smoke (`cli_smoke`) | `rarust/tests/cli_smoke.rs` | ✅ 4 passed |

**Totals:** 22 integration + 3 doctest = **25 tests, 0 failures.**

### 11.3 Commands / Gates

| Gate | Command | Result |
|------|---------|--------|
| Build | `cargo build` | ✅ |
| Test | `cargo test` | ✅ 25 passed (22 integration + 3 doctest) |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | ✅ clean |
| `unsafe` check | `grep -r unsafe rarust-core/src/` | ✅ 0 matches |

### 11.4 Create Testing Notes

- **Store method**: Roundtrip verified (create → list → extract → byte-compare).
- **Compressed method**: Roundtrip verified with repetitive data (Best level).
- **Multi-volume**: Creation verified (`.part1.rar` + `.part2.rar` written correctly).
  Extraction of multi-volume is a separate read-path task (requires rars multivolume reader).
- **Encrypted creation**: Rejected at `build()` with clear `Unsupported` error
  because rars 0.4.1's writer rejects encrypted output at `finish()`.
- **CLI smoke**: Roundtrip test via `rarust create + rarust extract` passes.

