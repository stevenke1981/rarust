# /refactor

Purpose: 安全重構，不改變外部行為。

## Procedure

1. Identify code smell.
2. Confirm behavior boundaries.
3. Ensure baseline tests exist.
4. Refactor in one small step.
5. Run tests.
6. Compare behavior.
7. Document the refactor reason.

## Forbidden

- No feature changes.
- No dependency upgrade unless required.
- No sweeping formatting.
- No public API change unless approved.
