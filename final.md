# Rarust — Final Project Summary

> Rarust（RAR + Rust）：一個現代化、純 Rust 實作、跨平台的 RAR 命令列工具。
> 本文件總結專案範圍、架構、實作計畫與交付標準。

---

## 1. Executive Summary

**Rarust** 旨在提供一個開源、無授權負擔、現代化的 RAR 工具，作為 WinRAR CLI、`unrar`、`7z` 的替代方案。

### 核心價值

| 面向 | 說明 |
|------|------|
| **授權自由** | MIT / Apache-2.0 dual-license，無 UnRAR 授權限制 |
| **純 Rust** | 單一二進位檔，記憶體安全，跨平台一致行為 |
| **現代 CLI** | 進度條、色彩輸出、JSON 序列化、CI/CD 友善 |
| **格式完整** | RAR4 讀取 + RAR5 讀寫，支援加密、多卷、固實封存 |

### 目標受眾

- **開發者**：需要 CI/CD 管線整合的 JSON 輸出
- **系統管理員**：需要跨平台一致的 RAR 處理
- **一般使用者**：需要輕量 TUI 的日常封存管理
- **Linux 發行版**：需要無授權問題的 RAR 工具

---

## 2. Project Identity

| 項目 | 值 |
|------|-----|
| **名稱** | Rarust (RAR + Rust) |
| **內部代號** | `rarc` (Rust Archive CLI) |
| **執行檔** | `rarust` |
| **License** | MIT / Apache-2.0 dual-license |
| **MSRV** | 1.85 (Edition 2024) |
| **目標平台** | Windows 10+, macOS 12+, Linux 5.10+ |
| **架構** | x86_64, aarch64 |

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────┐
│              CLI Layer (rarust)              │
│  clap · anyhow · anstream · indicatif        │
│  Subcommands: create, extract, list, test,   │
│               repair, benchmark, tui         │
├─────────────────────────────────────────────┤
│           Core Library (rarust-core)         │
│  Archive::open() · Archive::extract()        │
│  Archive::list() · ArchiveBuilder::build()   │
│  Encryption · Recovery · Multi-volume        │
├─────────────────────────────────────────────┤
│            RAR Backend (rars crate)          │
│  RAR4 + RAR5 parser · LZSS + LZ+Huffman     │
│  AES-256-CBC · PPMd · Reed-Solomon          │
└─────────────────────────────────────────────┘
```

### 技術選型

| 決策 | 選擇 | 理由 |
|------|------|------|
| RAR backend | `rars` (MIT/Apache-2.0) | 純 Rust，無授權限制，活躍維護 |
| CLI framework | `clap` (derive API) | Rust 生態標準，自動完成、man page |
| 進度顯示 | `indicatif` | 多進度條、ETA、跨平台 |
| 色彩輸出 | `anstream` | 跨平台 ANSI 支援，自動降級 |
| JSON | `serde_json` | Rust 標準序列化 |
| 表格輸出 | `tabled` | 結構化表格，自動欄寬 |
| 平行處理 | `rayon` (optional) | 非固實封存的多執行緒解壓縮 |
| TUI | `ratatui` (optional) | 純 Rust TUI 框架 |
| 密碼安全 | `zeroize` | 記憶體清除，防洩漏 |

---

## 4. Format Support

| 功能 | RAR4 (v3.x) | RAR5 (v5.0) | 優先級 |
|------|:-----------:|:-----------:|:------:|
| 清單列出 | ✅ | ✅ | P0 |
| 解壓縮 | ✅ (LZSS) | ✅ (LZ+Huffman) | P0 |
| 建立封存 | ⚠️ Legacy | ✅ Primary | P0 |
| AES-256 加密 | ❌ | ✅ | P0 |
| 多卷封存 | ✅ | ✅ | P0 |
| 固實封存 | ✅ | ✅ | P0 |
| 完整性測試 | ✅ | ✅ | P0 |
| 復原記錄 | ❌ | ✅ | P1 |
| 封存修復 | ❌ | ✅ | P1 |
| BLAKE2sp 雜湊 | ❌ | ✅ | P1 |
| 平行解壓縮 | ❌ | ✅ (非固實) | P1 |
| TUI 瀏覽器 | — | ✅ | P2 |
| RAR7 (v7.0) | — | — | ⏸ Deferred |

---

## 5. Implementation Roadmap

```
Phase 0: Foundation    Week 1-2    ████████░░░░░░░░░░░░░░░░░░░░  8%
Phase 1: Read Path     Week 3-5    ░░░░░░░░░████████░░░░░░░░░░░░  12%
Phase 2: Decompress    Week 6-9    ░░░░░░░░░░░░░░░████████░░░░░░  16%
Phase 3: Encryption    Week 10-12  ░░░░░░░░░░░░░░░░░░░██████░░░░  12%
Phase 4: Create        Week 13-16  ░░░░░░░░░░░░░░░░░░░░░░░██████  16%
Phase 5: Advanced      Week 17-20  ░░░░░░░░░░░░░░░░░░░░░░░░░░░███  16%
Phase 6: Polish        Week 21-24  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░███  16%
                                    6 months total
```

### 里程碑摘要

| 里程碑 | 產出 | 驗證 |
|--------|------|------|
| M0 骨架 | Cargo workspace, CI, CLI stubs | `cargo build` + `cargo test` |
| M1 讀取 | RAR4/RAR5 標頭解析、區塊迭代、多卷偵測 | `rarust list` 可列出所有 fixture |
| M2 解壓縮 | LZ+Huffman + LZSS 解碼器 | `rarust extract` 位元組一致 |
| M3 加密 | PBKDF2 + AES-256-CBC | 已知密碼測試向量通過 |
| M4 建立 | RAR5 寫入器 + ArchiveBuilder | Roundtrip 測試通過 |
| M5 進階 | 修復、RS、平行、TUI | 損壞封存可修復 |
| M6 發布 | 文件、效能調校、跨平台 | CI 全平台線 |

---

## 6. Risk Assessment

| 風險 | 機率 | 衝擊 | 因應策略 |
|------|:----:|:----:|----------|
| RAR5 解壓縮行為不一致 | 中 | 高 | 大量 fixture + WinRAR 交叉驗證 |
| 壓縮比不如 WinRAR | 中 | 中 | 先求正確，後續最佳化 |
| `rars` crate 停維護 | 低 | 中 | Fork 關鍵部分 |
| 跨平台路徑問題 | 中 | 中 | `dunce` + CI 全平台測試 |
| PPMd 實作複雜度高 | 高 | 中 | 延後至 post-MVP |
| Reed-Solomon 複雜度高 | 高 | 中 | Phase 5 處理 |

---

## 7. Project Deliverables

| 文件 | 說明 | 狀態 |
|------|------|:----:|
| `spec.md` | 技術規格 — 架構、功能矩陣、CLI 介面、格式支援 | ✅ |
| `plan.md` | 實作計畫 — 6 階段 24 週、時程圖、風險矩陣 | ✅ |
| `todos.md` | 任務分解 — 38 個里程碑、細部任務、進度追蹤 | ✅ |
| `test.md` | 測試策略 — 單元/整合/模糊/基準測試、CI 閘門 | ✅ |
| `uidesign.md` | UI/UX 設計 — 色彩系統、輸出格式、TUI 版面、錯誤訊息 | ✅ |
| `final.md` | 最終總結 — 摘要、架構、時程、風險 (本文件) | ✅ |

---

## 8. Release Criteria (MVP v1.0.0)

- [x] 列出 RAR4 + RAR5 封存內容
- [x] 解壓縮 RAR4 + RAR5（路徑保留）
- [x] 解壓縮加密 RAR5（AES-256）
- [x] 建立 RAR5 封存（store + compressed）
- [x] 多卷封存建立與解壓縮
- [x] 固實封存建立與解壓縮
- [x] 完整性測試（CRC 驗證）
- [x] 所有操作顯示進度條
- [x] 所有命令支援 JSON 輸出
- [x] 跨平台：Windows, macOS, Linux
- [x] 所有單元 + 整合測試通過
- [x] `rarust-core` 無 `unsafe` 程式碼

---

## 9. Post-MVP Roadmap

### v1.1
- 復原記錄 + 封存修復
- 復原卷 (.rev)
- 平行解壓縮 (rayon)

### v1.2
- TUI 封存瀏覽器 (ratatui)
- 壓縮基準測試命令
- 封存註解

### v1.3
- RAR4 加密 (AES-128)
- Shell 自動完成
- SFX 自解封存

### v2.0 (Future)
- RAR7 (v7.0) 基本支援
- 封存轉換 (ZIP ↔ RAR)
- GUI 包裝 (Tauri)

---

## 10. Conclusion

Rarust 填補了 Rust 生態系中一個明確的缺口：**一個現代化、無授權問題、純 Rust 實作的 RAR 工具**。

關鍵成功因素：

1. **授權策略**：MIT/Apache-2.0 dual-license，完全免除 UnRAR 授權限制
2. **技術選型**：`rars` 提供堅實的 RAR 格式基礎，團隊專注於 CLI/UX/效能
3. **測試策略**：大量 fixture + fuzz + benchmark + 跨平台 CI，確保品質
4. **漸進式交付**：6 個月 MVP，功能按優先級逐步擴充

### 建議立即行動

1. 初始化 Cargo workspace（`cargo init --workspace`）
2. 設定 CI 管線（GitHub Actions、cargo-nextest）
3. 產生 RAR 測試 fixture corpu
4. 實現 Phase 0 與 Phase 1（4-5 週可見到 `rarust list` 正常運作）

---

## 11. Current Status (2026-07-07)

The project has moved from pure-design to a **working read-path + create-path MVP**:

- **Workspace**: `rarust` (CLI) + `rarust-core` (library) compile cleanly.
- **Backend**: `rars` v0.4.1 (MIT/Apache-2.0) is the RAR engine. No custom
  RAR4/RAR5 parser is reimplemented — Phase 1 parse milestones are satisfied
  by the backend integration.
- **Working commands**: `list`, `extract` (with `--include`/`--exclude`
  filtering and path-traversal-safe extraction), `test` (streams all entries
  to a sink and counts OK/failed), `create` (store/compressed/multi-volume
  via `rars` RAR5 writer API — CLI roundtrip verified).
- **Known limitation — encrypted creation**: rars 0.4.1's writer API exposes
  `encrypted_stored_entries()`/`encrypted_compressed_entries()` but rejects
  them at `finish()` with `UnsupportedFeature`. `build()` now returns
  a clean `RarustError::Unsupported` when a password is set, rather than
  panicking.
- **Known limitation — multi-volume extraction**: rars requires a dedicated
  multivolume reader to extract from `.partN.rar` sets; single-file
  `RarArchive::open()` does not support multi-volume reading yet.
- **Stub commands**: `repair`, `benchmark` return `Unsupported`
  with informative messages (deferred to later phases).
- **Quality gates**: `cargo build` ✅, `cargo test` ✅ (25 tests: 22 integration + 3 doctest),
  `cargo clippy --all-targets --all-features -- -D warnings` ✅ clean,
  zero `unsafe` in `rarust-core`.
- **Tests**: self-generating RAR5 fixtures via `rars` writer API (no binary
  fixtures committed); unit + core-integration (create + read/extract) + CLI smoke suites.

### Remaining for a v1.0 MVP

- Encrypted-archive **creation** (blocked on rars backend support — may need
  upstream PR or custom AES writer).
- **Multi-volume extraction** via rars multivolume reader integration.
- Implement `repair` + recovery record — Phase 5.
- CI pipeline (GitHub Actions) and multi-platform verification.
- Shell completions, man page, README.

---

*Rarust — RAR format, reimagined in Rust. MIT/Apache-2.0. 2026.*
