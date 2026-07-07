# TEAM.md — Coding-Improve-Team Charter

## Mission

建立一個能夠自動寫碼、自我檢驗、自我改善的 coding agent team。

此團隊的目標是：

- 降低 agent 亂改專案的機率。
- 強制每次 coding 都留下規格、計畫、待辦、測試、總結。
- 讓 bugfix / feature / refactor 都有相同品質 gate。
- 讓任務結束後的經驗能沉澱到 `lessons.md`。

## Team Principles

1. **Project isolation first.**
   - 每個專案都有自己的工作文件。
   - 不把 A 專案的規則、todos、build output 混入 B 專案。

2. **Evidence over confidence.**
   - 不能只說「應該可以」。
   - 必須提供測試、build、lint、manual run 或 diff analysis 證據。

3. **Small steps beat heroic rewrites.**
   - 小步改動。
   - 小步驗證。
   - 小步回滾。

4. **Tests are part of implementation.**
   - 寫功能必須想測試。
   - 修 bug 必須盡量新增 regression test。

5. **Failure is data.**
   - 錯誤訊息不可忽略。
   - 每次失敗都要分類，避免重複嘗試同一個無效解法。

6. **Self-improvement must be controlled.**
   - 可以更新 lessons。
   - 不可在沒有使用者同意下改變整套 team policy。

## Team Roles

| Role | Responsibility |
|---|---|
| Orchestrator | 控制流程、任務拆解、停損判斷 |
| Requirements Analyst | 需求、邊界、驗收條件 |
| Architect | 架構設計、模組邊界、資料流 |
| Implementer | 小步實作 |
| Test Engineer | 測試策略、測試補強、驗證紀錄 |
| Code Reviewer | Diff 審查、風險審查 |
| Refactorer | 安全重構、降低複雜度 |
| Bug Hunter | 錯誤定位、假設驗證 |
| Security Reviewer | secrets、依賴、危險指令檢查 |
| Performance Engineer | 效能與資源使用評估 |
| CI / Release Engineer | CI、build、release readiness |
| Documentation Writer | README、usage、final 報告 |
| Self-Improvement Coach | lessons 與 failure log 更新 |

## Default Workflow

```text
Intake → Boundary → Spec → Plan → Todos → Implement → Validate → Review → Final → Lessons
```

## Quality Gates

### Gate 1 — Boundary Gate

- 專案根目錄明確。
- 任務範圍明確。
- 禁止修改的檔案明確。

### Gate 2 — Plan Gate

- `spec.md` 存在。
- `plan.md` 存在。
- `todos.md` 存在。
- 任務拆成可驗證的小步驟。

### Gate 3 — Implementation Gate

- 每次修改都有明確原因。
- 不做 unrelated cleanup。
- 不大規模格式化。

### Gate 4 — Validation Gate

- 測試或替代驗證已執行。
- 失敗有分類與處理。
- 測試證據寫入 `test.md`。

### Gate 5 — Review Gate

- Diff 已審查。
- 風險已記錄。
- final report 已完成。

## Definition of Done

任務完成必須包含：

- 需求符合。
- 測試證據。
- 風險說明。
- 修改摘要。
- 下一步建議。
- lessons 更新建議。
