# Project Isolation Workflow

## Goal

避免 Codex / OpenCode 把要執行的專案跟目前專案混在一起。

## Rules

1. Always confirm current working directory.
2. Keep task docs inside the project root.
3. Keep generated artifacts inside `worklog/` or configured output directory.
4. Never use another project path unless explicitly requested.
5. Do not copy global AGENTS into a project unless user asks.

## Recommended Layout

```text
project-root/
├─ AGENTS.md
├─ TEAM.md
├─ spec.md
├─ plan.md
├─ todos.md
├─ test.md
├─ final.md
└─ worklog/
```
