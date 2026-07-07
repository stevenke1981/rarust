# /add-feature

Purpose: 新增功能，但保持架構邊界與測試完整。

## Procedure

1. Define feature acceptance criteria in `spec.md`.
2. Identify impacted modules.
3. Add tests or smoke test plan.
4. Implement minimal vertical slice.
5. Validate.
6. Expand only after first slice passes.
7. Update docs and final report.

## Feature Gate

- Must not break existing behavior.
- Must not mix unrelated refactor.
- Must include usage notes if user-facing.
