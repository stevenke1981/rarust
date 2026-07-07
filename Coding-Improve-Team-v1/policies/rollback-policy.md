# Rollback Policy

## Trigger

- Tests fail after repeated attempts.
- Scope expands unexpectedly.
- Change breaks unrelated behavior.
- Security issue appears.

## Procedure

1. Stop editing.
2. Identify changed files.
3. Preserve logs.
4. Roll back minimal failing change.
5. Re-run baseline validation.
6. Document the rollback.
