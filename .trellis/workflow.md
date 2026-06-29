# Development Workflow

---

## Core Principles

1. **Plan before code** — figure out what to do before you start
2. **Specs injected, not remembered** — guidelines are injected via hook/skill, not recalled from memory
3. **Persist everything** — research, decisions, and lessons all go to files
4. **Incremental development** — one task at a time
5. **Capture learnings** — after each task, review and write new knowledge back to spec

---

## DiJiang Workflow

DiJiang uses a **dispatch → grill → output → implement/tdd → hunt ↔ check** workflow:

### Phase 1: Requirements (grill)
- Use `/dj-grill` to align on requirements
- One question at a time, with recommended answers
- Output: `prd.md`

### Phase 2: Document (output)
- Use `/dj-output` to create design docs, implementation plans
- Output: `design.md`, `implement.md`

### Phase 3: Implement
- Use `/dj-implement` for feature coding
- Use `/dj-tdd` for test-driven development
- Use `/dj-ponytail` for minimal, focused changes
- Output: working code + tests

### Phase 4: Investigate (hunt)
- Use `/dj-hunt` for bug investigation
- Systematic root cause analysis
- Output: fix + spec update

### Phase 5: Check
- Use `/dj-check` for quality review
- Use `/dj-audit` for whole-codebase over-engineering scans
- Output: verified changes

---

## Project Structure

```
.trellis/            # Task management + specs
├── tasks/           # Task directories (task.json, prd.md, design.md, …)
├── spec/            # Coding guidelines by package/layer
├── workspace/       # Developer journals
└── workflow.md      # This file

.dijiang/            # DiJiang configuration
└── config.toml

.pi/                 # Pi platform configuration
├── skills/          # DiJiang workflow skills
├── agents/          # Sub-agent definitions
├── prompts/         # Prompt templates
└── settings.json
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang status` | Show project and active task status |
| `dijiang start <name>` | Create and activate a new task |
| `dijiang task list` | List all tasks |
| `dijiang task current` | Show active task |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang mem list` | List memory sessions |

## Skill Routing

| Request type | Use |
|---|---|
| New feature / unclear requirements | `/dj-dispatch` → `/dj-grill` |
| Feature implementation | `/dj-implement` or `/dj-tdd` |
| Bug / regression | `/dj-hunt` |
| Code review | `/dj-check` |
| Documentation | `/dj-output` |
| Refactoring | `/dj-ponytail` |
| Prototype | `/dj-prototype` |
| UI design | `/dj-design` |
| Script / tool | `/dj-script` |


<!-- BEGIN DIJIANG-MANAGED BLOCK: do not edit between these markers -->
# Development Workflow

---

## Core Principles

1. **Plan before code** — figure out what to do before you start
2. **Specs injected, not remembered** — guidelines are injected via hook/skill, not recalled from memory
3. **Persist everything** — research, decisions, and lessons all go to files
5. **Capture learnings** — after each task, review and write new knowledge back to spec
6. **MUSE memory** — every task session is tracked for cross-session recall

## DiJiang Workflow
DiJiang uses the **dj-* skill ecosystem**, a full pipeline from dispatch to delivery:

### Phase 0: Dispatch & Memory (entry)
- Use `dj-dispatch` to classify any new task
- Automatically routes to the correct dj-* skill
- `dj-muse` creates a session for cross-task memory tracking
- Use `dj-grill` to align on requirements
- One question at a time, with recommended answers
- Output: `prd.md`

### Phase 2: Document (output)
- Use `dj-output` to create design docs, implementation plans
- Output: `design.md`, `implement.md`

### Phase 3: Implement
- Use `dj-implement` for feature coding
- Use `dj-tdd` for test-driven development
- Use `dj-ponytail` for minimal, focused changes
- Output: working code + tests

### Phase 4: Investigate (hunt)
- Use `dj-hunt` for bug investigation
- Systematic root cause analysis
- Output: fix + spec update

### Phase 5: Check
- Use `dj-check` for quality review
- Use `dj-audit` for whole-codebase over-engineering scans
- Output: verified changes

---

## Project Structure

```
.trellis/            # Task management + specs
├── tasks/           # Task directories (task.json, prd.md, design.md, …)
├── spec/            # Coding guidelines by package/layer
├── workspace/       # Developer journals
└── workflow.md      # This file

.dijiang/            # DiJiang configuration
└── config.toml

.pi/                 # Pi platform configuration
├── settings.json    # Platform settings (skills, prompts, extensions)
├── skills/          # Project-level skills (delegates to global dj-*)
├── agents/          # Sub-agent definitions
└── prompts/         # Prompt templates
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang status` | Show project and active task status |
| `dijiang start <name>` | Create and activate a new task |
| `dijiang task list` | List all tasks |
| `dijiang task current` | Show active task |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang task archive <name>` | Archive a task (set status + timestamp) |
| `dijiang task prune --days N` | Delete archived tasks older than N days |
| `dijiang mem list` | List memory sessions |
| `dijiang mem sync` | Sync all platform sessions to ~/.dijiang/mem/ |
| `dijiang template list` | List built-in + cached templates |
| `dijiang template pull <source>` | Pull template from gh:owner/repo or URL |
| `dijiang template validate <path>` | Validate a template manifest |
## Skill Routing

| Request type | Use |
| Request type | Use |
|---|---|
| New feature / unclear requirements | `dj-dispatch` → `dj-grill` |
| Feature implementation | `dj-implement` or `dj-tdd` |
| Bug / regression | `dj-hunt` |
| Code review / quality | `dj-check` |
| Whole-codebase audit | `dj-audit` |
| Technical debt assessment | `dj-debt` |
| Codebase health report | `dj-health` |
| Security review | `dj-review` |
| Documentation / specs | `dj-output` |
| Session management | `dj-muse` |
| Handoff between sessions | `dj-handoff` |
| Refactoring | `dj-ponytail` |
| Prototype | `dj-prototype` |
| UI design | `dj-design` |
| Script / tool | `dj-script` |
| Pattern research | `dj-pattern` |
| Write documentation | `dj-write` |
| Long code discussion | `dj-karpathy` |

<!-- END DIJIANG-MANAGED BLOCK -->
