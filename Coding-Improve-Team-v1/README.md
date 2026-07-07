# Coding-Improve-Team

一套模仿 `Research-Team` 架構的 **Coding Team / Auto Coding / Self Check / Continuous Improve** 文件包。

這個包適合放進每一個程式專案根目錄，讓 Codex Desktop、OpenCode、其他 agent coding 工具在執行任務時遵守同一套流程：

1. 先理解專案，不亂改。
2. 先建立 `spec.md`、`plan.md`、`todos.md`、`test.md`、`final.md`。
3. 再小步修改程式碼。
4. 每一步都要有驗證證據。
5. 測試失敗要分類、回滾、修正。
6. 任務結束後寫入 lessons，讓下次更穩。

> 核心目標：讓 coding agent 不只是「寫程式」，而是能規劃、實作、測試、審查、回滾、自我改善。

## 適用場景

- Rust / TypeScript / Python / C / C++ / Go / JavaScript 專案
- Codex Desktop 專案目錄
- OpenCode 專案目錄
- 需要自動化長任務的 agent coding workflow
- 想避免「寫到一半亂改架構」或「任務和目前專案混在一起」的情境

## 快速開始

將整包內容複製到你的專案根目錄：

```powershell
Copy-Item -Recurse Coding-Improve-Team-v1\* D:\your-project\
cd D:\your-project
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate.ps1
```

如果你只想全域使用，可以把 `AGENTS.md`、`TEAM.md`、`.opencode/commands/`、`.agents/skills/` 複製到你的 Codex/OpenCode 全域設定目錄，但最建議的方式仍然是：

> 每個專案都放一份自己的 coding team 規則，避免任務互相污染。

## 建議任務輸入格式

```text
請使用 Coding-Improve-Team 流程。
任務：新增 Windows app 的硬體資訊頁面。
限制：不要改動 unrelated files。
必須產出：spec.md、plan.md、todos.md、test.md、final.md。
驗收：cargo test 通過，cargo clippy 無重大警告，GUI 可啟動。
```

## 核心流程

```text
Request
  ↓
Project Boundary Check
  ↓
Spec / Plan / Todos
  ↓
Small Implementation Step
  ↓
Build / Test / Lint
  ↓
Self Review
  ↓
Fix or Rollback
  ↓
Final Evidence Report
  ↓
Lessons Update
```

## 主要檔案

| 檔案 | 用途 |
|---|---|
| `AGENTS.md` | Agent 入口規則，最重要 |
| `TEAM.md` | 團隊角色與工作原則 |
| `opencode.jsonc` | OpenCode 指令載入設定 |
| `.opencode/commands/` | 可直接呼叫的 slash commands |
| `.agents/skills/coding-improve-team/SKILL.md` | Codex-style Skill |
| `roles/` | Coding agent 角色定義 |
| `workflows/` | 自動 coding、bugfix、refactor、release 流程 |
| `policies/` | 邊界、安全、測試、回滾政策 |
| `templates/` | `spec.md`、`plan.md`、`todos.md` 等模板 |
| `scripts/validate.ps1` | Windows 驗證入口 |
| `scripts/validate.py` | 跨平台驗證腳本 |

## 最重要的使用原則

1. 不要讓 agent 從使用者家目錄或錯誤資料夾執行專案。
2. 每個專案都要有自己的 `spec.md`、`plan.md`、`todos.md`、`test.md`、`final.md`。
3. 每次修改都要有測試證據。
4. 沒有讀過現有程式碼，不得重寫架構。
5. 不得把 temporary experiment 混進正式程式。
6. 失敗三次要停止盲修，改走 failure recovery。

## 推薦 GitHub Topics

```text
coding-agent
codex
opencode
auto-coding
self-improvement
agent-workflow
tdd
code-review
rust
devex
```
