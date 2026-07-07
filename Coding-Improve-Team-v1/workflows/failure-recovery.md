# Failure Recovery Workflow

## Trigger

- Same error failed 3 times.
- Build/test output is misunderstood.
- Agent keeps changing unrelated files.
- The fix expands beyond the original scope.

## Steps

1. Stop editing.
2. Copy the exact error into `FAILURE_LOG.md` draft.
3. Classify failure.
4. Re-read the smallest relevant file.
5. Form one hypothesis.
6. Make one minimal change.
7. Run one targeted validation.
8. Continue only if evidence improves.

## Rule

Do not stack multiple speculative fixes in one edit.
