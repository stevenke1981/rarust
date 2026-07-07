# Project Boundaries Policy

## Purpose

避免 agent 把目前任務寫到錯誤專案或混入其他專案。

## Rules

- Always identify project root before editing.
- Keep task docs in the same project.
- Do not modify sibling projects.
- Do not write generated output into user home directory unless requested.
- Do not use absolute paths from previous tasks unless user provided them again.

## Pass Criteria

- Changed files are inside project root.
- Every changed file is related to task.
- No unrelated generated files are left behind.
