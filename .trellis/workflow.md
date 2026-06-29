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
