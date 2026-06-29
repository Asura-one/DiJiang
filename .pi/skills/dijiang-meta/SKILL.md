---
name: dijiang-meta
description: "Understand and customize the local DiJiang architecture inside a user project. Use when modifying .dijiang/ plus platform hooks, dj-* skills, commands, workflows, or agent harness configuration."
---

# DiJiang Meta

This skill is for local DiJiang users who have already run `dijiang init` in a project. After reading it, an AI should understand the DiJiang architecture, operating model, and customization entry points inside that user project, then modify the generated `.dijiang/` and platform directory files according to the user's request.

## Local Architecture

DiJiang manages the following directories inside a user project:

### Core Data Layer (`.dijiang/`)
- `config.toml`: Project metadata (name, developer, version, platforms) and directory paths
- `tasks/`: Task storage with JSONL session logs
- `spec/`: Coding guidelines (backend/, frontend/, guides/)
- `workspace/`: Per-developer journals and session traces
- `workflow.md`: Development workflow phases and skill routing rules
- `.runtime/`: Runtime state (active task tracking, session files)

### Skill Layer (`.pi/skills/`)
- `dijiang-start/`: Session initialization — loads project context, delegates to dj-dispatch
- `dijiang-continue/`: Resumes work on active task
- `dijiang-finish-work/`: Wraps up session, writes journal, archives
- `dj-{dispatch,grill,implement,tdd,hunt,check,...}/`: 19 core workflow skills

### Platform Directories
- `.pi/`: Pi agent platform (skills, agents, prompts, sessions)
- `.claude/`: Claude Code integration
- `.codex/`: Codex integration
- `.cursor/`: Cursor integration

## DJ-* Skills

DiJiang ships 19 embedded workflow skills that form the agent harness:

| Skill | Purpose |
|-------|---------|
| dj-dispatch | Task classifier — routes requests to appropriate dj-* skill |
| dj-grill | Requirements alignment — asks questions one at a time |
| dj-implement | Code implementation with verification |
| dj-tdd | Test-driven development |
| dj-hunt | Bug investigation and debugging |
| dj-check | Code review and verification |
| dj-audit | Over-engineering and anti-pattern scan |
| dj-debt | Technical debt assessment |
| dj-health | Codebase health report |
| dj-output | Document generation (PRD, design, specs) |
| dj-handoff | Session handoff and context transfer |
| dj-design | Design documentation |
| dj-ponytail | Minimal, focused changes |
| dj-prototype | Prototype development |
| dj-pattern | Pattern research |
| dj-karpathy | Long code discussion |
| dj-review | Security review |
| dj-script | Script/tool development |
| dj-write | Write documentation |

Skills are managed by the CLI:
- `dijiang skills`: List all available dj-* skills
- `dijiang skills --sync`: Write skills to current project's `.pi/skills/`
- `dijiang init`: Automatically writes skills during project initialization

## Customization

### workflow.md
Edit `.dijiang/workflow.md` to:
- Add/modify development phases
- Change skill routing rules
- Define custom workflows for features, bugs, reviews

### config.toml
Edit `.dijiang/config.toml` to:
- Change platform integrations (`platforms = ["pi", "claude", "cursor"]`)
- Adjust directory paths
- Update project metadata

### dj-* Skills
Skills are embedded in the DiJiang binary and written to `~/.dijiang/skills/` on first run. To customize:
1. Edit files in `~/.dijiang/skills/dj-*/SKILL.md`
2. Run `dijiang skills --sync` to update current project
3. Or manually edit `.pi/skills/dj-*/SKILL.md` in the project

## Backward Compatibility

DiJiang maintains backward compatibility with Trellis projects:
- `find_dijiang_dir()` falls back to `.trellis/` if `.dijiang/` doesn't exist
- `dijiang migrate` renames `.trellis/` to `.dijiang/` for legacy projects
- Status display shows "DiJiang project: detected" when infrastructure is present

## File References

When modifying DiJiang infrastructure, always:
1. Read the actual files in the user project first
2. Treat local content as authoritative
3. Follow existing patterns and conventions
4. Verify changes with `cargo test` or `dijiang status`
