---
name: dijiang-start
description: "Initialize a DiJiang session: load project context, active task, workflow state, then delegate to dj-dispatch for task routing."
triggers:
  - session:start
---

# Start Session

Initialize a DiJiang-managed development session. This skill only loads context and routes the request; task execution belongs to the selected `dj-*` skill.

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | User request, current repository, DiJiang project state |
| 输出 | Active task context, loaded workflow/specs, and a selected `dj-*` route |
| 非目标 | Do not implement, edit code, commit, or finish the task from this skill |

## Steps

### 1. Load Project State

```bash
dijiang status
dijiang task current
git status --short --branch
```

Output: active task path, current phase, git branch, dirty state, and developer identity when available.

If `dijiang status` fails, run `pwd` and check whether `.dijiang/` exists. If `.dijiang/` is missing, stop and route to project initialization instead of inventing task state.

### 2. Read Workflow

```bash
test -f .dijiang/workflow.md && cat .dijiang/workflow.md
dijiang workflow-state --json
```

Output: canonical phase mapping and current workflow state.

If `.dijiang/workflow.md` is missing, read `AGENTS.md` and continue only when it contains a DiJiang block. If neither file exists, stop with `missing DiJiang workflow context`.

### 3. Discover Specs

```bash
test -d .dijiang/spec && find .dijiang/spec -maxdepth 2 -type f | sort
test -f .dijiang/spec/index.md && cat .dijiang/spec/index.md
```

Output: applicable spec files to load before routing or implementation.

`.trellis/` may exist in migrated projects as a legacy fallback, but new sessions must load `.dijiang/` first.

### 4. Initialize Session Memory

Record initial findings only when the user has already provided a concrete task description. Durable memory must include source and scope; raw guesses stay in the task context.

```bash
dijiang mem findings --finding "<initial task description; source=user; scope=current task>"
```

If memory commands fail, continue without writing memory and mention the failure in the handoff. Do not block session startup on memory persistence.

### 5. Delegate to dj-dispatch

Load `dj-dispatch` to classify the user's request and route to the appropriate `dj-*` skill.

| Request type | Routes to |
|---|---|
| New feature / unclear requirements | `dj-grill` |
| Feature implementation | `dj-implement` or `dj-tdd` |
| Bug / regression / crash | `dj-hunt` |
| Code review / quality check | `dj-check` |
| Documentation / PRD / design | `dj-output` |
| Refactoring / code quality | `dj-ponytail` |
| Prototype / spike | `dj-prototype` |
| Tech debt tracking | `dj-debt` |
| UI design | `dj-design` |
| Script / tooling | `dj-script` |

Output: one route, one next action, and the task artifacts that the routed skill must read.

## Failure Handling

| Trigger | First action | Fallback |
|---|---|---|
| `dijiang status` fails | Confirm repository root and `.dijiang/` presence | Stop and ask for project initialization or correct working directory |
| No active task exists | Route the user request through `dj-dispatch` | Create a task only through DiJiang start/dispatch flow |
| Active task exists but artifacts are missing | Read `task.json` and list present files | Mark missing artifacts explicitly before routing |
| Workflow state conflicts with task status | Prefer `.dijiang/tasks/<task>/task.json` as source of truth | Record conflict and route to `dj-hunt` if execution would be unsafe |
| Git tree is dirty before work starts | Report dirty files and keep them out of the new task scope | Use a dedicated worktree before implementation |

## 🔴 CHECKPOINT · Startup Complete

离开该 skill 前先报告：

```text
Active task: <name or none>
Workflow phase: <planning|in_progress|completed|paused|none>
Loaded specs: <paths or none>
Route: <dj-skill>
Next action: <one sentence>
```

🛑 STOP if the route is unclear, project state is missing, or continuing would require guessing task intent.

## Anti-patterns

| Do not | Do this instead |
|---|---|
| Do not implement code during startup | Route to `dj-implement`, `dj-tdd`, or `dj-hunt` first |
| Do not invent an active task when DiJiang reports none | Use `dj-dispatch` to create or select one |
| Do not ignore dirty git state | Report it and require a task worktree before edits |
| Do not write durable memory from guesses | Keep uncertain context in the task artifact |
| Do not bypass `dj-dispatch` for ambiguous user requests | Let dispatch classify and record the route |
