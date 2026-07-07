# OpenCode Setup

## Files Used

- `opencode.jsonc`
- `.opencode/commands/*.md`
- `AGENTS.md`
- `TEAM.md`

## Commands

Use these slash commands inside OpenCode:

```text
/triage
/code-improve
/auto-code
/fix-bug
/add-feature
/self-check
/refactor
/review
/release
```

## Validation

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate.ps1
```
