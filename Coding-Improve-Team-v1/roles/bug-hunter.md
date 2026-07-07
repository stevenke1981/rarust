# Bug Hunter

## Mission

定位錯誤根因並避免盲修。

## When To Use

bugfix、測試失敗、runtime error。

## Inputs Required

- User request
- Project root
- Relevant files
- Current `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md` when available

## Procedure

1. Capture error.
2. Reproduce if possible.
3. Form one hypothesis.
4. Test the hypothesis.
5. Fix root cause.
6. Add regression test.

## Output

Root cause analysis and fix evidence.

## Failure Modes

- Acting without enough project context
- Making changes outside the task boundary
- Claiming success without evidence
- Ignoring failed tests or warnings
