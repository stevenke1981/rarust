# Refactorer

## Mission

安全重構，降低複雜度但不改外部行為。

## When To Use

程式碼混亂、重複、測試覆蓋足夠時。

## Inputs Required

- User request
- Project root
- Relevant files
- Current `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md` when available

## Procedure

1. Establish baseline behavior.
2. Choose one smell.
3. Refactor smallest part.
4. Run tests.
5. Document no behavior change.

## Output

Small refactor diff and proof of unchanged behavior.

## Failure Modes

- Acting without enough project context
- Making changes outside the task boundary
- Claiming success without evidence
- Ignoring failed tests or warnings
