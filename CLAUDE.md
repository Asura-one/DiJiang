# DiJiang — DiJiang Project

This project uses DiJiang, an AI-native development workflow framework.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.dijiang/tasks/` — Task records (JSON, Trellis-compatible format)
- `.dijiang/spec/` — Project specifications and coding guidelines
- `.dijiang/workspace/` — Developer journals and session traces
- `.pi/` — Pi agent platform configuration
- `crates/` — Rust workspace crates (cli, task, mem, configurator, mcp-server)

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

- `dijiang status` / `status --compat` — 项目概览 / 兼容性诊断
- `dijiang start <name>` / `dijiang dispatch <prompt>` — session 与路由
- `dijiang task list|current|start|status|archive|prune` — 任务生命周期
- `dijiang finish-work --verification ... --docs-sync ... --version-impact ...` — 收尾（可选 commit/push/integrate）
- `dijiang mem ...` / `dijiang channel ...` / `dijiang template ...` / `dijiang skills [--sync]`
- `dijiang workflow-state --json` / `dijiang skill-body <name>` — runtime route 与 skill body
- `dijiang doc-sync` / `dijiang spec-sync` / `dijiang bucket` / `dijiang context` / `dijiang commit` / `dijiang session`
- `dijiang init` / `dijiang migrate` / `dijiang update [--from-github]`
- `cargo build -p dijiang` — Build CLI binary
- `cargo test` — Run all tests
