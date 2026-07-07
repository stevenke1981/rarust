# Controlled Coding Loop

## Goal

讓 agent 以可控方式自動寫碼。

## Stages

### 1. Intake

- Read user request.
- Classify task.
- Identify project root.

### 2. Boundary

- List files likely to change.
- List files that must not change.
- Identify build and test commands.

### 3. Spec

- Write `spec.md`.
- Define acceptance criteria.
- Define non-goals.

### 4. Plan

- Write `plan.md`.
- Split into small verifiable steps.

### 5. Todos

- Write `todos.md`.
- One checkbox per concrete step.

### 6. Implement

- Modify only relevant files.
- One step at a time.

### 7. Validate

- Run targeted tests.
- Run broader validation when appropriate.
- Record results in `test.md`.

### 8. Review

- Review diff.
- Check risk.
- Fix blocking issues.

### 9. Final

- Write `final.md`.
- Update lessons if useful.

## Stop Conditions

- Project root unclear.
- Requirements conflict.
- Same failure repeats 3 times.
- Changes exceed task boundary.
