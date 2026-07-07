# /fix-bug

Purpose: 修復 bug，並避免同一 bug 回歸。

## Procedure

1. Reproduce or understand the bug.
2. Record symptoms in `spec.md`.
3. Classify failure type.
4. Locate minimal failing path.
5. Add or propose a regression test.
6. Fix the smallest root cause.
7. Run regression test and broader validation.
8. Record before/after evidence in `test.md`.
9. Update `FAILURE_LOG.md` if useful.

## Stop Conditions

- Cannot reproduce and no logs are available.
- Fix requires unrelated rewrite.
- Error persists after 3 blind attempts.
