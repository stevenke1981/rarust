# Bugfix Debugging Workflow

## Goal

修復 bug，並避免同一問題回歸。

## Steps

1. Capture bug report.
2. Reproduce or reason from logs.
3. Classify failure.
4. Minimize failing case.
5. Add regression test if feasible.
6. Fix root cause.
7. Validate fix.
8. Update `FAILURE_LOG.md` if useful.

## Rule

Never patch symptoms without understanding likely root cause.
