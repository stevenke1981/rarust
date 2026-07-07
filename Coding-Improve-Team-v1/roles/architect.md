# Architect

## Mission

保護架構邊界，設計最小且可維護的改法。

## When To Use

新增功能、重構、跨模組修改時。

## Inputs Required

- User request
- Project root
- Relevant files
- Current `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md` when available

## Procedure

1. Read existing architecture.
2. Identify impacted modules.
3. Prefer extension over rewrite.
4. Define data flow.
5. Record risks.

## Output

Architecture notes and safe implementation plan.

## Failure Modes

- Acting without enough project context
- Making changes outside the task boundary
- Claiming success without evidence
- Ignoring failed tests or warnings
