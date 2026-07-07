# /code-improve

Purpose: 改善目前專案品質，但不得做無邊界的大重寫。

## Procedure

1. Read `AGENTS.md`, `TEAM.md`, README, build files, and existing tests.
2. Identify the project root and language stack.
3. Create or update `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md`.
4. Run baseline validation if possible.
5. Identify the top 3 improvement opportunities:
   - correctness
   - tests
   - maintainability
   - security
   - performance
   - documentation
6. Implement only the highest-value low-risk improvement first.
7. Run validation.
8. Self-review the diff.
9. Write final evidence.

## Stop Conditions

- Project boundary is unclear.
- Tests cannot be run and no alternative validation is possible.
- Improvement requires large architecture rewrite without explicit approval.
