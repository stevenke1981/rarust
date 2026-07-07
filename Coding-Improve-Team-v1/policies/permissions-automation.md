# Permissions and Automation Policy

## Goal

讓 agent 能持續執行合理的 build/test/read output 工作，同時避免危險操作。

## Usually Safe

- Read project files.
- Run build command.
- Run tests.
- Read test output.
- Create task docs.
- Write generated reports inside project.

## Needs Caution

- Installing dependencies.
- Running scripts from internet.
- Modifying global config.
- Deleting files.
- Network operations.

## Forbidden Unless Explicitly Requested

- Permanent deletion.
- Credential operations.
- Destructive disk commands.
- Force push.
