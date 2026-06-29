<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->


<!-- DIJIANG:START -->
# DiJiang Project Instructions

This project uses DiJiang for task management and workflow.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.trellis/tasks/` — active and archived tasks
- `.trellis/spec/` — coding guidelines
- `.trellis/workspace/` — developer journals
- `.pi/` — Pi platform configuration

## Available Commands

| Command | Description |
|---------|-------------|
| `/dj-dispatch` | Classify and route a task |
| `/dj-grill` | Requirements alignment |
| `/dj-implement` | Code implementation |
| `/dj-hunt` | Debug and investigate |
| `/dj-check` | Code review and verification |
| `/dj-muse` | Memory management |
| `/dj-output` | Document generation |
| `/dj-tdd` | Test-driven development |
| `/dj-design` | Design documentation |
| `dijiang status` | Show project status |
| `dijiang task list` | List active tasks |
| `dijiang task current` | Show active task |
| `dijiang mem list` | Show memory sessions |

## Workflow

1. Read this file and `.trellis/workflow.md` at session start
2. Check active task with `dijiang task current`
3. Read task artifacts (task.json, prd.md, design.md, implement.md)
4. Read relevant spec files from `.trellis/spec/`
5. Follow the task status phase to determine workflow:
   - `planning` → dj-grill (requirements alignment)
   - `in_progress` → dj-implement (implementation)
   - `review` → dj-check (verification)

Managed by DiJiang. Edits outside this block are preserved.
<!-- DIJIANG:END -->