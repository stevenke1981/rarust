# Rarust — Feature Enhancement Design

> Rarust 功能強化設計文件，涵蓋 GUI 強化、多格式擴充、系統整合三大階段。

---

## Overview

### Vision

將 Rarust 從純 CLI RAR 工具擴展為完整的跨平台封存管理方案：
- **GUI**：現代化桌面封存瀏覽器，媲美商業工具的操作體驗
- **多格式**：支援主流封存格式的建立與解壓
- **系統整合**：無縫融入桌面環境

### Phased Approach

| Phase | Focus | Priority |
|-------|-------|----------|
| **1** | GUI 強化 — 拖放、主題、預覽、批量操作、進度條 | 最高 |
| **2** | 多格式擴充 + 封存轉換 | 中 |
| **3** | 系統整合 + SFX 自解壓縮 | 低 |

Each phase is independently shippable.

---

## Phase 1: GUI Enhancement

### Architecture

```
rarust/src/gui/
├── mod.rs              # Public exports
├── app.rs              # Main app state & UI composition
├── fonts.rs            # CJK font loading (unchanged)
├── i18n.rs             # Translations (extended)
├── theme.rs            # ✨ NEW — Theme system (Light/Dark)
├── widgets/
│   ├── mod.rs
│   ├── file_table.rs   # ✨ Sortable file list
│   ├── preview.rs      # ✨ In-app file preview
│   ├── progress.rs     # ✨ Progress dialog
│   └── password.rs     # ✨ Password prompt dialog
└── actions/
    ├── mod.rs
    ├── extract.rs      # ✨ Batch extraction logic
    └── create.rs       # ✨ GUI archive creation
```

### 1.1 Theme System (`theme.rs`)

**Design:**

```rust
pub struct Theme {
    pub name: &'static str,
    pub window: Color32,
    pub chrome: Color32,
    pub chrome_dark: Color32,
    pub list_bg: Color32,
    pub header_bg: Color32,
    pub row_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub selected: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}
```

**Dark Theme Palette:**

| Token | Light (existing) | Dark |
|-------|-----------------|------|
| `window` | `#F2F4F7` | `#1E1E2E` |
| `chrome` | `#E8ECF1` | `#2B2B3D` |
| `list_bg` | `#FFFFFF` | `#252540` |
| `header_bg` | `#E2E7EE` | `#363650` |
| `row_alt` | `#F8FAFC` | `#2E2E48` |
| `border` | `#B1BBC7` | `#444466` |
| `text` | `#1A1E24` | `#E0E0F0` |
| `muted` | `#535D6B` | `#9090B0` |
| `accent` | `#0067C0` | `#70A0FF` |
| `selected` | `#D6E9FF` | `#3A3A6A` |

**User Interaction:**
- Menu: `View → Theme → Light / Dark`
- Shortcut: `Ctrl+L` / `Ctrl+D`
- Persisted to `dirs::config_dir()/rarust/gui-theme` as plain text
- `apply(ctx)` rewrites egui `Style` and `Visuals`

### 1.2 Drag & Drop

**Drop from OS file manager:**
- egui `raw_input` event → detect file paths dragged into window
- Supported extensions: `.rar` `.zip` `.tar` `.tar.gz` `.tgz` `.gz`
- Drops on empty area or welcome screen → open archive
- Drops on archive path bar → open archive

**Drag from entry list:**
- egui drag-and-drop source on selected entries
- External drop target (file explorer) → extract to drop location
- Requires platform-native DnD integration via `eframe`

### 1.3 Sortable Column Headers

**Implementation:**
- Click column header → sort by that field
- Click same header again → reverse sort order
- Triangle indicator (`▲` / `▼`) on active sort column
- Sort state: `(field: SortField, reverse: bool)`

**Fields:** Name, Size, Ratio, Modified, CRC32, Method

**State machine:**
```
None → Name(asc) → Name(desc) → Size(asc) → Size(desc) → ...
```
Or direct click on any field.

### 1.4 Password Management

**Flow:**
1. User opens encrypted archive → check if password known
2. If no password → show `PasswordDialog`:
   - Password field (masked, with toggle visibility)
   - `Remember for this session` checkbox
   - `Save permanently` checkbox (future: keyring integration)
3. On successful decrypt → store in `password_cache: HashMap<String, String>`
4. On failed decrypt → show error, allow retry

**Edge Cases:**
- Wrong password during extraction → prompt again (max 3 attempts)
- Header-encrypted archives → prompt before listing
- Cancel → graceful fallback (show error state)

### 1.5 File Preview

**Trigger:**
- Double-click file entry
- Right-click → Preview
- Select + `Space` key

**Preview Types:**

| File Type | Preview Method | Max Size |
|-----------|---------------|----------|
| `.txt`, `.md`, `.rs`, `.py`, etc. | Plain text (UTF-8/UTF-16 auto-detect) | 1 MB |
| Binary (other) | Hex dump (first 512 bytes) | 512 B |
| Directory | N/A | — |
| Archived | N/A | — |

**UI:**
- Modal window or side panel (configurable)
- Monospace font
- Line numbers
- Scrollable
- `Close` button + `Esc` to dismiss

### 1.6 Batch Operations

**Multi-selection model:**

| Action | Behavior |
|--------|----------|
| `Click` | Select single, deselect others |
| `Ctrl+Click` | Toggle selection of one item |
| `Shift+Click` | Range select |
| `Ctrl+A` | Select all |
| `Ctrl+Shift+A` | Deselect all |
| `Space` | Toggle selection (keyboard) |

**State:** `selected_indices: HashSet<usize>`

**Batch Extract:**
- Toolbar: `Extract Selected` / `Extract All` (dropdown or split button)
- Selected count in status bar: `Selected: 5/128 files (12.3 MB)`
- Extract dialog: choose destination, overwrite mode, flat paths

### 1.7 Progress Dialog

**Design:**
```
┌──────────────────────────────────────┐
│  Extracting: data/database.sql       │
│  ┌──────────────────────────────────┐│
│  │ ████████████████░░░░░░░░░░░░░░  ││  62%
│  └──────────────────────────────────┘│
│  File 45/100 • 123.4 MB / 274.2 MB  │
│  Speed: 98.2 MB/s  •  ETA: 00:01:10│
│                                      │
│  [Cancel]                            │
└──────────────────────────────────────┘
```

**Implementation:**
- Separate `egui::Window` with `modal` flag
- Runs on a background thread, communicates via `Arc<Atomic>` progress values
- Cancel sets a flag → current file finishes → operation stops
- On completion: replace with summary view

**Progress Data:**
```rust
pub struct ProgressState {
    pub total_items: u64,
    pub completed_items: u64,
    pub current_item: String,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub elapsed_secs: f64,
    pub cancelled: AtomicBool,
}
```

### 1.8 GUI Archive Creation

**Dialog Flow:**
1. `File → New Archive` or toolbar button
2. Dialog with sections:
   - **Output:** path picker + filename
   - **Format:** RAR5 / RAR4 / ZIP / TAR / TAR.GZ / GZIP
   - **Compression:** Store / Fastest / Fast / Normal / Good / Best
   - **Options:** Password, Solid, Volume size, Header encrypt
   - **Files:** Add files / Add folder (walkdir)
3. File list with remove button
4. `Create` → ProgressDialog → completion notification

### 1.9 Multi-tab Support

**Tab bar:**
- New archive opens in new tab (up to 8)
- Tab shows archive filename (truncated if long)
- `×` button on each tab to close
- Keyboard: `Ctrl+Tab` / `Ctrl+Shift+Tab` to cycle
- State per tab: `(archive_path, LoadedArchive, selected, search, password)`

**Data model:**
```rust
struct Tab {
    id: u64,
    title: String,
    archive: Option<LoadedArchive>,
    archive_path: Option<String>,
    search: String,
    selected: Option<usize>,
    password: Option<String>,
    load_error: Option<String>,
}
```

### 1.10 Context Menu

Right-click on entry row:

```
Extract              → extract_selected()
Extract To...        → pick folder, then extract
─────────────────
Test                 → test_archive()
Preview              → show_preview() (text files only)
─────────────────
Copy Name            → clipboard entry.name
Copy Full Path       → clipboard "archive_path/entry.name"
─────────────────
Properties           → show_properties() dialog
```

---

## Phase 2: Format Expansion & Conversion

### 2.1 New Formats

Add BZIP2, XZ, and 7z support to `rarust-core`:

| Format | Create | Extract | Crate |
|--------|--------|---------|-------|
| BZIP2  | ✅     | ✅      | `bzip2` |
| XZ     | ✅     | ✅      | `xz2` / `liblzma` |
| 7z     | ⚠️ (basic) | ✅ | `sevenz-rust` or custom |

**Dependencies (Cargo.toml):**
```toml
bzip2 = { version = "0.5", optional = true }
xz2 = { version = "0.1", optional = true }
sevenz-rust = { version = "0.7", optional = true }

[features]
full-formats = ["bzip2", "xz2", "sevenz-rust"]
```

### 2.2 `rarust convert` Command

```
rarust convert [OPTIONS] <SOURCE> <TARGET>

Arguments:
  <SOURCE>          Source archive path
  <TARGET>          Target archive path (extension determines format)

Options:
  -p, --password <PASSWORD>   Source decryption password
  -P, --target-password       Target encryption password
  -m, --method <LEVEL>        Compression level for target
  -s, --solid                 Solid compression (RAR5/7z only)
  -v, --volume <SIZE>         Target volume size
  -f, --force                 Overwrite target without prompt
  -d, --dry-run               Show what would be done
```

**Conversion matrix:**

| From ↓ \ To → | RAR5 | RAR4 | ZIP | TAR | TAR.GZ | GZIP | BZIP2 | XZ | 7z |
|--------------|------|------|-----|-----|--------|------|-------|----|----|
| RAR5 | — | copy | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| RAR4 | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ZIP | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TAR | ✅ | ✅ | ✅ | — | ✅ | — | ✅ | ✅ | ✅ |
| TAR.GZ | ✅ | ✅ | ✅ | ✅ | — | — | ✅ | ✅ | ✅ |
| GZIP | ⚠️ | ⚠️ | ⚠️ | ✅ | ✅ | — | — | — | ⚠️ |

Legend: ✅ = supported, ⚠️ = single-file only, — = N/A

### 2.3 GUI: Convert Dialog

- `Tools → Convert Archive...`
- Source: pick archive
- Target: pick path + format (dropdown)
- Options: password, compression level
- Progress dialog during conversion

---

## Phase 3: System Integration & SFX

### 3.1 Windows Shell Integration

**Context Menu (right-click .rar/.zip/.tar/.7z):**
- "Extract with Rarust..." → `rarust extract "%1" "%1_extracted"`
- "Extract Here" → `rarust extract "%1" "."`
- "Test with Rarust" → `rarust test "%1"`

**Registration:**
- Windows Registry: `HKEY_CLASSES_ROOT\*\shell\RarustExtract`
- Icon overlay for supported archive types
- Implemented via `rarust shell-register` / `rarust shell-unregister` CLI commands

**File Association:**
- Installer option: "Associate .rar files with Rarust"
- Registers `rarust-gui "%1"` as default handler

### 3.2 SFX (Self-Extracting Archives)

**Module:** `rarust-core/src/sfx.rs`

**Approach:**
- Generate a standalone executable that contains:
  1. A minimal Rust decompressor (stripped, ~200KB)
  2. The compressed archive payload appended to the binary
- Only for RAR5 (other formats deferred)

**CLI:**
```
rarust create archive.exe input.zip --sfx
rarust create archive.exe input/ --sfx
```

**SFX Module Components:**
```rust
pub struct SfxConfig {
    pub output_path: PathBuf,
    pub extract_to_temp: bool,     // default: extract to CWD
    pub silent: bool,              // no console window
    pub overwrite_mode: OverwriteMode,
    pub extract_path: Option<String>,  // fixed extract path
    pub icon_path: Option<PathBuf>,    // custom .exe icon
}

pub fn build_sfx(archive_data: &[u8], config: &SfxConfig) -> Result<()>;
```

### 3.3 Archive Comment Editing

**CLI:**
```
rarust comment <archive> [comment-file]
rarust comment --clear <archive>
rarust comment --show <archive>
```

**GUI:**
- Properties dialog → Comment tab
- Editable text area
- Max 256 KB (RAR5 limit)

### 3.4 GUI: Properties Dialog

Right-click → Properties or `Ctrl+I`:

```
┌──────────────────────────────────────┐
│  Properties — report.pdf             │
│──────────────────────────────────────│
│  Name:       report.pdf              │
│  Path:       docs/report.pdf         │
│  Size:       1.2 MB (1,228,800 B)    │
│  Packed:     712 KB (729,088 B)      │
│  Ratio:      58%                     │
│  Method:     m3 normal               │
│  Modified:   2026-06-15 14:30:00     │
│  CRC32:      A1B2C3D4               │
│  Encrypted:  No                      │
│  Archived:   backup.rar              │
└──────────────────────────────────────┘
```

---

## GUI Component Dependency Map

```
app.rs
├── theme.rs          (independent)
├── fonts.rs          (independent)
├── i18n.rs           (used by all)
├── widgets/
│   ├── file_table.rs → app.rs
│   ├── preview.rs    → app.rs
│   ├── progress.rs   → actions/extract.rs, actions/create.rs, app.rs
│   └── password.rs   → app.rs
└── actions/
    ├── extract.rs    → widgets/progress.rs, rarust_core
    └── create.rs     → widgets/progress.rs, rarust_core
```

---

## Testing Strategy

| Feature | Test Type | Method |
|---------|-----------|--------|
| Theme switching | Unit | Verify color values after apply |
| Sortable columns | Unit | Verify sort order for each field |
| Password dialog | Unit | Verify prompt flow, cache behavior |
| Progress dialog | Integration | Mock Archive, verify progress updates |
| Batch selection | Unit | Verify set operations |
| File preview | Integration | Preview .txt, .rs, binary; verify content |
| Drag & drop | Manual | Test on Windows/macOS/Linux |
| Multi-tab | Integration | Open 3 archives, verify tab state |
| Format conversion | Integration | Convert RAR→ZIP, verify output |
| BZIP2/XZ/7z | Integration | Create + extract roundtrip |
| SFX | Integration | Build SFX, extract, verify contents |
| Shell integration | Manual | Register, test context menu |
| Comment editing | Integration | Write, read, clear comments |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|:----------:|:------:|------------|
| egui DnD API gaps | Medium | Medium | Fall back to file picker if drag not supported |
| 7z format complexity | High | High | Deploy with LGPL notice; start with read-only |
| SFX binary size | Medium | Medium | Use LTO + strip; target < 500KB |
| Shell reg. permissions | Low | Low | Document admin requirement |
| Theme persistence race | Low | Low | Atomic write to config file |
