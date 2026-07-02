<!-- DIJIANG:START -->
# DiJiang Project Instructions

This project uses DiJiang for task management and workflow.

## Project Structure

- `.dijiang/` — DiJiang project state and configuration
- `.dijiang/tasks/` — active and archived tasks
- `.dijiang/spec/` — coding guidelines
- `.dijiang/workspace/` — developer journals
- `.dijiang/workflow.md` — canonical workflow projection
- `.pi/` — Pi platform configuration

## Layer Boundaries

| Layer | Responsibility |
|-------|----------------|
| `dijiang` CLI | Project state, task lifecycle, memory persistence, templates, platform config, agent channels; `dijiang finish-work` is the only layer that mutates task archive/journal/commit/push/integration state |
| `dj-*` skills | Atomic work capabilities such as alignment, implementation, investigation, checking, docs, and reports |
| `dijiang-*` skills | Session wrappers for start, continue, and finish-work; `/skill:dijiang-finish-work` loads the finish-work skill and the agent must follow its Invocation Contract before calling CLI |
| `/dijiang-*` prompts | Lightweight Pi prompt checklists; they inject guidance but do not execute CLI state transitions |
| `AGENTS.md` | Minimal routing index for agents; not a second workflow definition |

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang init [name]` | Initialize DiJiang project state and platform configuration |
| `dijiang status` | Show project status |
| `dijiang status --compat` | Show compatibility diagnostics |
| `dijiang start <name>` | Create and activate a work session |
| `dijiang dispatch <prompt>` | Create or reuse an active task from a natural-language request and emit route context |
| `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | Finish current work with validation, docs/spec evidence, version decision, optional commit/integration, journal, and archive |
| `dijiang task list` | List active tasks |
| `dijiang task current` | Show active task |
| `dijiang task start <name>` | Create or activate a task record with low-level task semantics |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang task archive <name>` | Archive a task |
| `dijiang task prune --days N` | Prune old archived tasks |
| `dijiang mem list` | List platform sessions |
| `dijiang mem sync` | Sync platform sessions |
| `dijiang mem findings --finding "..."` | Append project finding |
| `dijiang mem learn --lesson "..."` | Record project lesson |
| `dijiang mem correction --correction "..." --lesson "..." --actionability "..."` | Record a user correction with memory quality metadata |
| `dijiang mem archive` | Archive current memory session |
| `dijiang mem tactic --name N --description D` | Add global tactic |
| `dijiang mem tactics --select N` | List or select tactics with Thompson sampling |
| `dijiang mem record --tactic T --outcome success --context C` | Record tactic outcome |
| `dijiang mem pattern --name N --description D` | Add a project pattern or standard operating procedure |
| `dijiang mem patterns` | List project patterns |
| `dijiang mem stats` | Show memory statistics |
| `dijiang mem backup` | Back up project memory to global storage |
| `dijiang mem evolve` | Analyze session memory and extract tactics |
| `dijiang mem finetune` | Run the slow memory fine-tuning loop |
| `dijiang template list` | List available templates |
| `dijiang template pull <source>` | Pull a template |
| `dijiang template validate <path>` | Validate a template |
| `dijiang skills` | List available `dj-*` skills |
| `dijiang skills --sync` | Sync project `dj-*` skills |
| `dijiang workflow-state --json` | Output workflow state for hooks/agents |
| `dijiang migrate` | Migrate legacy `.trellis/` state to `.dijiang/` |
| `dijiang channel spawn <agent>` | Spawn an agent channel |
| `dijiang channel list` | List active channels |
| `dijiang channel send <id> <message>` | Send message to a channel |
| `dijiang channel execute <id>` | Execute an agent in a channel |
| `dijiang channel execute-all` | Execute all active channels in parallel |
| `dijiang channel status <id>` | Check channel status |
| `dijiang channel stop <id>` | Stop a channel |
| `dijiang update` | Update managed DiJiang skills, agents, prompts, hooks, and workflow projections |
| `dijiang update --from-github` | Refresh global skills from GitHub before updating the project |

## Skill Routing

| Category | Use |
|----------|-----|
| New task / unclear request | `dj-dispatch` |
| Requirements alignment | `dj-grill` |
| Planning docs | `dj-output` |
| Feature implementation | `dj-implement` or `dj-tdd` |
| Bug / regression | `dj-hunt` |
| Code review / quality gate | `dj-check` |
| Lightweight read-only review | `dj-review` |
| Whole-codebase audit | `dj-audit` |
| Technical debt assessment | `dj-debt` |
| Codebase health report | `dj-health` |
| Pattern research | `dj-pattern` |
| Minimal focused changes | `dj-ponytail` |
| Prototype | `dj-prototype` |
| UI design | `dj-design` |
| Script / tool | `dj-script` |
| Writing polish | `dj-write` |
| Long code discussion | `dj-karpathy` |
| Session handoff | `dj-handoff` |
| Session findings / lessons | `dijiang mem findings` / `dijiang mem learn` |

## Workflow Routing

1. Read this file and `.dijiang/workflow.md` at session start.
2. Check active task with `dijiang task current`.
3. Read task artifacts: `task.json`, `prd.md`, `design.md`, `implement.md` when present.
4. Read relevant spec files from `.dijiang/spec/`.
5. Route by canonical task status:
   - none → `dijiang start <name>` or `dj-dispatch`
   - `planning` → `dj-grill`, optionally `dj-output`
   - `in_progress` → implementation skill, then `dj-check`
   - `completed` → `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>`
   - `archived` → read-only unless restarted with `dijiang start <task>`
   - `paused` → `dijiang-continue` then return to `planning` or `in_progress`

`review` is not a canonical task status. Use `dj-check` for quality verification.

Managed by DiJiang. Edits outside this block are preserved.
<!-- DIJIANG:END -->
