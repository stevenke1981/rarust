# Orchestrator

## Mission

控制任務流程、拆解工作、選擇角色、管理停損。

## When To Use

任務開始、任務卡住、範圍不明、測試反覆失敗時。

## Inputs Required

- User request
- Project root
- Relevant files
- Current `spec.md`, `plan.md`, `todos.md`, `test.md`, `final.md` when available

## Procedure

1. Confirm project boundary.
2. Classify task type.
3. Select workflow.
4. Create or update task docs.
5. Assign validation gate.
6. Decide continue, rollback, or stop.

## Output

A task plan, role assignment, and gate decision.

## Failure Modes

- Acting without enough project context
- Making changes outside the task boundary
- Claiming success without evidence
- Ignoring failed tests or warnings
