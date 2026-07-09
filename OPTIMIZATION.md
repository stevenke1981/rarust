# Rarust — Optimization & Improvement Plan

> Generated from codebase review (2026-07-09).  
> Status: 🟡 Pending / 🔵 In Progress / ✅ Done  
> Priority: **P0** (security/correctness) → **P1** (UX/ops) → **P2** (perf/polish)

---

## Executive snapshot

| Area | Current state | Main gap |
|------|---------------|----------|
| Core read path | list / extract / test via `rars` | Overwrite, flat extract incomplete |
| Core write path | RAR5/RAR4 create, encrypt, multi-vol | Full-file RAM load; limited streaming |
| Security | Basic path checks; Password type exists | Traversal incomplete; password not zeroized in archive; CLI password resolution unused |
| CLI | clap subcommands wired | Progress bar unused; error exit codes weak |
| CI / quality | Local tests pass | No GitHub Actions, no rustfmt/clippy configs in CI |
| GUI | egui shell present | Secondary to CLI hardening |

**Verification baseline (before this plan):**  
`cargo test --workspace` — all tests green.

---

## P0 — Security & correctness (implement first)

### O1. Harden extract path traversal ✅ (target this pass)

**Problem:** `Entry::safe_extract_path` only rejects `..` substrings and absolute `/` paths.  
Windows absolute paths (`C:\...`), UNC (`\\server\share`), mixed separators, and component-level `..` can still be unsafe. Comment admits production should canonicalize.

**Plan:**
- Normalize separators, reject empty segments, `.`, `..`, drive prefixes, UNC roots.
- Join only relative components under `dest`.
- Prefer `dunce::simplified` / component walk; optional post-create containment check.
- Unit tests for `../escape`, `C:\windows`, `//evil`, nested `a/../../b`.

### O2. Use zeroizing password storage end-to-end ✅ (target this pass)

**Problem:** `RarArchive` stores `password: Option<Vec<u8>>` without zeroize.  
`encryption::Password` + `password_from_env` / `password_from_file` / `warn_cli_password` exist but are **not wired** into CLI open/create/extract/list/test.

**Plan:**
- Store `Option<Password>` (or zeroizing buffer) inside `RarArchive` / builder path.
- Add `resolve_password(cli, password_file, password_stdin)` helper in CLI layer.
- Priority: `--password` (warn) → `RARUST_PASSWORD` → `--password-file` → stdin when flagged.
- Wire list/extract/test/create/repair.

### O3. Extract overwrite modes + flat mode ✅ (target this pass)

**Problem:** CLI defines `OverwriteMode` and `--flat` but extract ignores them.  
Always `File::create` (silent overwrite). Dry-run ignores include/exclude.

**Plan:**
- Pass extract options into core (`flat`, overwrite policy).
- `Skip` existing files; `Overwrite` create; `Rename` unique suffix; `Ask` → treat as skip in non-TTY / CI with warning (or require `-y`).
- Flat: write basename only under dest (still reject `..` basenames).
- Dry-run respects include/exclude filters.
- Return non-zero / error when `summary.errors > 0`.

### O4. Extract error accounting ✅ (target this pass)

**Problem:** `create_dir_all` / `File::create` failures often swallowed with `let _ =` or counted while still returning `Ok`. CLI prints success with only a WARN.

**Plan:**
- Propagate or count failures accurately; CLI exit code 2 when errors > 0.
- Map `WrongPasswordOrCorruptData` already done — extend for common I/O messages.

---

## P1 — Reliability, DX, ops

### O5. Multi-volume detector: compile regex once ✅ (target this pass)

**Problem:** `detect_part_n_rar` builds `Regex` on every call.

**Plan:** `std::sync::OnceLock<Regex>` static pattern.

### O6. GitHub Actions CI ✅ (target this pass)

**Problem:** todos list CI as pending; no `.github/workflows`.

**Plan:**
- Matrix: ubuntu / macos / windows
- Jobs: `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`
- Optional: nextest later

### O7. Progress bars for extract/create

**Problem:** `indicatif` is a dependency; extract comments “pending”.

**Plan:** MultiProgress for multi-file extract; byte progress when sizes known.

### O8. Wire `--keep-broken` or document unsupported

**Problem:** Flag stored in `OpenOptions` but never passed to backend (rars has no keep-broken API).

**Plan:** Either implement best-effort (write partial + log) or fail with clear “unsupported until backend support”.

### O9. Tree listing quality ✅ (this pass)

**Problem:** `print_tree` is a flat print with unused prefix placeholder.

**Plan:** Build hierarchy from path segments; box-drawing prefixes.

### O10. Split `archive.rs` (~700 lines)

**Problem:** Open/extract + full ArchiveBuilder write path in one module.

**Plan:** `builder.rs` (create path) + keep `archive.rs` for open/list/extract/test.

---

## P2 — Performance & product polish

### O11. Streaming create (avoid full RAM load)

**Problem:** `read_builder_entries` reads every input file fully into `Vec<u8>` before writing.

**Plan:** Where `rars` allows, stream store-mode from `Read`; for compressed path document memory requirement; chunk large files if API supports.

### O12. Parallel extract (non-solid)

**Problem:** `rayon` feature present but unused (`use rayon as _`).

**Plan:** Only after solid detection is reliable; parallel per-member extract for non-solid RAR5.

### O13. Repair / Benchmark stubs

**Problem:** Commands return Unsupported stubs.

**Plan:** Repair when recovery records exist; benchmark with synthetic buffer + timing table/JSON.

### O14. Metadata completeness

**Problem:** `crc32: None` always; method guessed as `"m3 normal"`.

**Plan:** Surface CRC/method from rars meta when available; otherwise document limitation.

### O15. rustfmt.toml / clippy.toml in repo root

**Problem:** todos claim done; files may be missing from workspace root.

**Plan:** Add minimal configs matching project standards.

### O16. GUI / TUI deferred polish

- TUI feature stub message misleading when feature off vs on.
- GUI i18n / large app.rs — separate track after CLI P0/P1.

---

## Implementation order for this session

| # | ID | Action | Verify |
|---|-----|--------|--------|
| 1 | O1 | Path-safe extract helpers + tests | unit tests |
| 2 | O2 | Password resolve + zeroize in open path | unit + smoke |
| 3 | O3–O4 | Extract options + exit codes | integration |
| 4 | O5 | OnceLock regex | unit |
| 5 | O6 | CI workflow | yaml present; local clippy/test |
| 6 | O15 | rustfmt/clippy configs | present |

Later sessions: O8, O10–O14.

---

## Risk notes

- Changing extract defaults from “always overwrite” to “ask/skip” may break scripts → default `Overwrite` when non-interactive, keep `Ask` only when TTY; or keep CLI default as documented (`ask`) but implement non-TTY fallback to `skip` with message.
- Zeroizing password requires careful lifetime with `ArchiveReadOptions` (borrows password bytes).
- CI on macOS/Windows may need feature flags if GUI fails headless — use `--no-default-features` + `parallel` for pure CLI CI, or `default-features` with GUI only on Windows display.

**CI recommendation:**  
`cargo test -p rarust-core` and `cargo test -p rarust --features parallel --no-default-features` for headless; optional GUI job separate.

---

## Done criteria for this pass

- [x] `OPTIMIZATION.md` written (this file)
- [x] O1–O5 implemented with tests
- [x] O6 CI workflow added
- [x] O15 configs added
- [x] `cargo test --workspace --no-default-features --features parallel` green
- [x] `cargo clippy -p rarust-core -p rarust --no-default-features --features parallel --all-targets -- -D warnings` clean

### Implementation log (2026-07-09)

| ID | Change |
|----|--------|
| O1 | `entry::safe_join_under` — reject `..`, drive letters, absolute, reserved device names; flat mode |
| O2 | `OpenOptions.password: Option<Password>`; CLI `resolve_password` + `--password-file` / `--password-stdin` |
| O3 | `ExtractOptions` + `OverwritePolicy` (Overwrite/Skip/Rename); dry-run respects filters |
| O4 | extract returns error when `errors > 0`; dir/file create failures counted |
| O5 | `OnceLock` for `.partN.rar` regex |
| O6 | `.github/workflows/ci.yml` matrix + clippy/fmt/test |
| O15 | `rustfmt.toml`, `clippy.toml` |
| O9 | `list --tree` now builds a stable hierarchy and renders branch prefixes |

### Implementation log (2026-07-09, follow-up pass)

| ID | Change |
|----|--------|
| O9 | Replaced path-repeat tree output with deterministic `BTreeMap` hierarchy rendering |
| O9 | Added unit tests for inferred directories and explicit empty directories |
| O9 | Updated CLI tree smoke test to verify branch-rendered output |
