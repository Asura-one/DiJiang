# Development Workflow

---

## Core Principles

1. **Plan before code** — align scope before implementation when requirements are unclear.
2. **Specs injected, not remembered** — guidelines are injected via hook/skill, not recalled from memory.
3. **Persist decisions** — task artifacts, findings, lessons, and handoffs are written to `.dijiang/`.
4. **One canonical workflow** — CLI, skills, AGENTS, prompts, and agents are projections of this model.

## DiJiang Canonical Workflow

DiJiang uses `dijiang` CLI for project state and `dj-*` skills for execution capability. `review` is not a canonical task status; quality verification is handled by `dj-check`.

| Task status | Workflow phase | Recommended entry | Output |
|-------------|----------------|-------------------|--------|
| none | dispatch | `dijiang start <name>` or `dj-dispatch` | Active task and routing decision |
| `planning` | align | `dj-grill`, optionally `dj-output` | `prd.md`, optionally `design.md` / `implement.md` |
| `in_progress` | implement | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-script` / `dj-design` | Working code, tests, verification notes |
| `in_progress` | check | `dj-check` | Verified diff and follow-up fixes |
| `completed` | finish | `dijiang finish-work --verification "..."` | Journal entry, archived task, cleared active session |
| `archived` | closed | Read-only, or restart with `dijiang start <task>` | No active work on archived task |
| `paused` | resume | `dijiang-continue` | Context restored, then return to `planning` or `in_progress` |

## Skill Taxonomy

| Category | Skills | Boundary |
|----------|--------|----------|
| Routing | `dj-dispatch` | Classify and route; do not implement directly |
| Alignment | `dj-grill` | Requirements alignment; do not write code |
| Planning docs | `dj-output` | PRD/design/implementation docs and code-doc alignment |
| Implementation | `dj-implement`, `dj-tdd`, `dj-hunt`, `dj-prototype`, `dj-script`, `dj-design` | Write code or investigate root causes |
| Quality gate | `dj-check` | Verify diff quality, completeness, safety, and regressions |
| Analysis reports | `dj-audit`, `dj-debt`, `dj-health`, `dj-pattern` | Produce reports; not a default delivery gate |
| Style overlays | `dj-ponytail`, `dj-karpathy` | Add constraints to another workflow path |
| Writing polish | `dj-write` | Polish prose; does not own engineering docs lifecycle |
| Session transfer | `dj-handoff` | Prepare handoff; does not replace finish-work journal |
| Session wrappers | `dijiang-start`, `dijiang-continue`, `dijiang-finish-work` | Load context, route, and close sessions |

## Project Structure

```
.dijiang/            # DiJiang project state
├── tasks/           # Task directories (task.json, prd.md, design.md, …)
├── spec/            # Coding guidelines by package/layer
├── workspace/       # Developer journals
├── workflow.md      # This file
└── config.toml      # DiJiang configuration

.pi/                 # Pi platform configuration
├── settings.json    # Platform settings
├── skills/          # Project-level skills
├── agents/          # Sub-agent definitions
└── prompts/         # Prompt templates
```

.trellis/ may be read only as a legacy compatibility fallback. New DiJiang templates should use `.dijiang/` as the primary path.

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang status` | Show project and active task status |
| `dijiang status --compat` | Show compatibility diagnostics |
| `dijiang start <name>` | Create and activate a work session |
| `dijiang finish-work --verification "..."` | Finish current work, write journal, archive task |
| `dijiang task list` | List all tasks |
| `dijiang task current` | Show active task |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang task archive <name>` | Archive a task |
| `dijiang task prune --days N` | Delete archived tasks older than N days |
| `dijiang mem list` | List platform sessions |
| `dijiang mem sync` | Sync platform sessions to `~/.dijiang/mem/` |
| `dijiang mem findings --finding "..."` | Append project finding |
| `dijiang mem learn --lesson "..."` | Record project lesson |
| `dijiang mem archive` | Archive current memory session |
| `dijiang mem tactic --name N --description D` | Add global tactic |
| `dijiang mem record --tactic T --outcome success --context C` | Record tactic outcome |
| `dijiang template list` | List built-in and cached templates |
| `dijiang template pull <source>` | Pull template from `gh:owner/repo` or URL |
| `dijiang template validate <path>` | Validate a template manifest |
| `dijiang skills --sync` | Sync project `dj-*` skills |
| `dijiang workflow-state --json` | Output workflow state for hooks/agents |
| `dijiang channel spawn <agent>` | Spawn an agent channel |
| `dijiang channel list` | List active channels |
| `dijiang channel send <id> <message>` | Send message to a channel |
| `dijiang channel execute <id>` | Execute an agent in a channel |
| `dijiang channel execute-all` | Execute all active channels in parallel |
| `dijiang channel status <id>` | Check channel status |
| `dijiang channel stop <id>` | Stop a channel |

## Routing Rules

| Request type | Use |
|--------------|-----|
| New task or unclear request | `dj-dispatch` |
| Requirements alignment | `dj-grill` |
| Feature implementation | `dj-implement` or `dj-tdd` |
| Bug or regression | `dj-hunt` |
| Code review / quality gate | `dj-check` |
| Whole-codebase audit | `dj-audit` |
| Technical debt assessment | `dj-debt` |
| Codebase health report | `dj-health` |
| Documentation / specs | `dj-output` |
| Handoff between sessions | `dj-handoff` |
| Minimal focused changes | `dj-ponytail` |
| Prototype | `dj-prototype` |
| UI design | `dj-design` |
| Script / tool | `dj-script` |
| Pattern research | `dj-pattern` |
| Writing polish | `dj-write` |
| Long code discussion | `dj-karpathy` |
| Session findings or lessons | `dijiang mem findings` / `dijiang mem learn` |
