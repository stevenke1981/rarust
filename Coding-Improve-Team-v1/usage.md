# Usage — Coding-Improve-Team

## 1. 專案內使用

最推薦方式：每個專案都放一份。

```powershell
Copy-Item -Recurse Coding-Improve-Team-v1\* D:\my-project\
cd D:\my-project
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate.ps1
```

## 2. OpenCode 使用

將 `.opencode/commands/` 保留在專案根目錄後，可使用：

```text
/code-improve 改善目前專案品質
/auto-code 依照 spec 自動實作
/fix-bug 修 bug 並補 regression test
/add-feature 新增功能
/self-check 自我檢驗目前修改
/refactor 安全重構
/review 審查 diff
/release 檢查 release readiness
```

## 3. Codex Desktop 使用

把整包放在專案根目錄，Codex Desktop 進入該資料夾後會讀取 `AGENTS.md`。

如果你想建立全域 Skill，可複製：

```text
.agents/skills/coding-improve-team/
```

到 Codex 可讀取的 skills 位置。

## 4. 建立新任務工作區

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\new-task.ps1 -Name "hardware-info-page"
```

會建立：

```text
worklog/tasks/hardware-info-page/
├─ spec.md
├─ plan.md
├─ todos.md
├─ test.md
└─ final.md
```

## 5. 推薦 Prompt

```text
請使用 Coding-Improve-Team 流程。
先確認專案根目錄與現有架構。
請建立或更新 spec.md、plan.md、todos.md、test.md、final.md。
一次只做小步修改，並在每一步後驗證。
任務：<寫你的任務>
限制：不要改 unrelated files，不要重寫整個架構。
驗收：<寫 build/test/run 條件>
```

## 6. Windows 建議

如果你使用 PowerShell：

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate.ps1
```

如果專案在 Windows，請務必讓 agent 使用正確工作目錄，例如：

```powershell
cd D:\your-project
```

不要從 `C:\Users\<name>` 直接執行專案任務。
