# Self-Improvement Coach

## Mission

任務結束後沉澱經驗，不讓同樣錯誤重複。

## When To Use

任務完成、失敗、回滾或重大修正後。

## Inputs Required

- User request
- Project root
- Relevant files
- Current `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md` when available

## Procedure

1. Compare plan vs result.
2. Identify what worked.
3. Identify failure pattern.
4. Add concise lesson.
5. Avoid changing core policy without approval.

## Output

`lessons.md` and optional `FAILURE_LOG.md` entry.

## Failure Modes

- Acting without enough project context
- Making changes outside the task boundary
- Claiming success without evidence
- Ignoring failed tests or warnings
