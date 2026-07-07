# Rarust — Lessons Learned

## 2026-07-07 — clap global vs subcommand short-option conflicts panic at parse time

Context:
- While building the `rarust` CLI, `rarust --help` / any subcommand call panicked
  at `Cli::parse()` with a clap builder error. Root cause: the same short flag was
  declared both as a **global** arg on `Cli` and again on a subcommand struct.
  - Global `-v`/`--verbose` (Count) collided with `ListArgs`/`TestArgs`/`RepairArgs` `-v`.
  - Global `-q`/`--quiet` collided with `TestArgs` `-q`.
  - `CreateArgs` used `short = 'h'` which clap reserves for `--help`.

Lesson:
- In clap derive, a short/long flag must be **unique across the whole command tree**
  when one side is `global = true`. Global flags should be the single source of truth.
- For verbosity/quiet, declare them only on the root `Cli` (global) and **pass the
  values into each command's `execute()` function** (as already done for `json` and
  `no_progress`), instead of re-declaring them per subcommand.
- Never use `short = 'h'` — clap always reserves `-h` for help.

Evidence:
- Panic reproduced: `thread 'main' panicked at rarust\src\cli.rs:17:18`.
- Fixed by removing `verbose`/`quiet` fields from `ListArgs`/`TestArgs`/`RepairArgs`,
  changing `CreateArgs` `--header-encrypt` to long-only, and threading `cli.quiet`
  into `test::execute`. After fix, `cargo test --test cli_smoke` → 3 passed and
  `rarust --help` renders correctly.

Apply When:
- Building or extending a clap-derive CLI with both global options and subcommands.
- Debugging a clap "panic at parse" that mentions duplicate arguments.
