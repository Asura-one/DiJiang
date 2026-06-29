<!-- DIJIANG:START -->
# DiJiang Project Instructions

This project uses DiJiang for task management and workflow.

## Project Structure

- `.dijiang/` — DiJiang project configuration
- `.dijiang/tasks/` — active and archived tasks
- `.dijiang/spec/` — coding guidelines
- `.dijiang/workspace/` — developer journals
- `.pi/` — Pi platform configuration

## Available Commands

| Command | Description |
|---------|-------------|
| `/dj-dispatch` | Classify and route a task |
| `/dj-dispatch` | Classify and route a task |
| `/dj-grill` | Requirements alignment |
| `/dj-implement` | Code implementation |
| `/dj-tdd` | Test-driven development |
| `/dj-hunt` | Debug and investigate |
| `/dj-check` | Code review and verification |
| `/dj-audit` | Whole-codebase audit |
| `/dj-debt` | Technical debt assessment |
| `/dj-health` | Codebase health report |
| `/dj-review` | Security review |
| `/dj-prototype` | Prototype development |
| `/dj-ponytail` | Minimal, focused changes |
| `/dj-design` | Design documentation |
| `/dj-output` | Document generation |
| `/dj-muse` | Memory management |
| `/dj-handoff` | Session handoff |
| `/dj-pattern` | Pattern research |
| `/dj-karpathy` | Long code discussion |
| `/dj-script` | Script / tool development |
| `/dj-write` | Write documentation |
| `dijiang status` | Show project status |
| `dijiang task list` | List active tasks |
| `dijiang task current` | Show active task |
| `dijiang task archive <name>` | Archive a task |
| `dijiang task prune --days N` | Prune old archived tasks |
| `dijiang template list` | List available templates |
| `dijiang template pull <source>` | Pull a template |
| `dijiang template validate <path>` | Validate a template |
| `dijiang mem list` | Show memory sessions |
| `dijiang mem sync` | Sync memory sessions |

## Workflow

1. Read this file and `.dijiang/workflow.md` at session start
2. Check active task with `dijiang task current`
3. Read task artifacts (task.json, prd.md, design.md, implement.md)
4. Read relevant spec files from `.dijiang/spec/`
5. Follow the task status phase to determine workflow:
   - `planning` → dj-grill (requirements alignment)
   - `in_progress` → dj-implement (implementation)
   - `review` → dj-check (verification)
   - `archived` → dj-check + dj-output (wrap-up & handoff)


Managed by DiJiang. Edits outside this block are preserved.
<!-- DIJIANG:END -->
