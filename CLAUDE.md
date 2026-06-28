# dijiang-dev — DiJiang Project

This project uses DiJiang, an AI-native development workflow framework.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.trellis/tasks/` — Task records (JSON, Trellis-compatible format)
- `.trellis/spec/` — Project specifications and coding guidelines
- `.trellis/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration
- `crates/` — Rust workspace crates (core, cli, task, mem, configurator)

## Task Workflow

Tasks live in `.trellis/tasks/<name>/` with these artifacts:
- `prd.md` — Requirements (mandatory)
- `design.md` — Technical design (complex tasks)
- `implement.md` — Execution plan (complex tasks)
- `task.json` — Structured task record (24 Trellis-compatible fields)

### Phases

| Status | Phase | Action |
|--------|-------|--------|
| planning | plan | Read PRD, align requirements |
| in_progress | implement | Write code |
| completed | complete | Verify quality gate |
| archived | archive | Move to archive |

## Core Workflow

1. **plan**: Start by reading `prd.md`, `design.md`, `implement.md`. Load specs from `spec/`.
2. **implement**: Write code following the specs. Run `cargo build` to verify.
3. **check**: Run `cargo test`, verify types, lint.
4. **archive**: Commit changes when done.

## CLI Commands

- `dijiang status` — Project overview
- `dijiang task list` — All tasks
- `dijiang task current` — Active task
- `dijiang start <name>` — Start task
- `cargo build -p dijiang-cli` — Build CLI
- `cargo test` — Run all tests
