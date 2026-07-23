# DiJiang — DiJiang Project

This project uses DiJiang, an AI-native development workflow framework.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.dijiang/tasks/` — Task records (JSON, Trellis-compatible format)
- `.dijiang/spec/` — Project specifications and coding guidelines
- `.dijiang/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration
- `crates/` — Rust workspace crates (core, cli, task, mem, configurator)

## Task Workflow

Tasks live in `.dijiang/tasks/<name>/` with these artifacts:
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

1. **plan**: Start by reading `dijiang workflow-state --json`, and treat injected `Skill Manifests` plus `<dijiang-target-skill ...>` as the primary runtime routing context before reading `prd.md`, `design.md`, `implement.md`.
2. **implement**: Write code following the runtime context and specs. Run `cargo build` to verify.
3. **check**: Run `cargo test`, verify types, lint.
4. **archive**: Commit changes when done.

## CLI Commands

- `dijiang status` — 项目概览
- `dijiang task list` — 所有任务
- `dijiang task current` — 当前任务
- `dijiang start <name>` — Start task
- `dijiang workflow-state --json` — Load injected runtime route + target skill context
- `cargo build -p dijiang-cli` — Build CLI
- `cargo test` — Run all tests
