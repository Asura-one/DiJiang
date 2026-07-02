---
name: dijiang-start
description: "Initialize a DiJiang session: load project context, active task, workflow state, then report the appropriate dj-* route."
triggers:
  - session:start
---

# Start Session

Initialize a DiJiang-managed development session. This skill only loads context and reports the selected route; task execution belongs to the selected `dj-*` skill.

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

If `dijiang status` fails, run `pwd` and check whether `.dijiang/` exists. If `.dijiang/` is missing, stop and output `follow-up: project initialization` instead of inventing task state.

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

### 5. Hand Off to dj-dispatch

Do not classify the request inside `dijiang-start`. Startup only loads context; task classification and level selection must be delegated to `dj-dispatch`, which is the single routing authority.

```bash
dijiang dispatch "<user request>" --json --hook-event session:start
```

When there is already an active task, read its `task.json` and the workflow-state output before deciding whether this is a continuation or a conflicting new request. Use the `dj-dispatch` session-start rules for that decision.

Only route directly to `dj-hunt` when project state is unsafe to interpret. Unsafe states are limited to missing/stale active task pointers, missing or unreadable `task.json`, invalid task status, or direct conflict between workflow-state and `task.json` about the active task or status. Missing optional artifacts such as `prd.md`, `design.md`, `implement.md`, or check notes is not a hunt trigger; mark them as absent and continue through `dj-dispatch` or the status-based route. Do not treat every `in_progress` task as a hunt task.

## Failure Handling

| Trigger | First action | Fallback |
|---|---|---|
| `dijiang status` fails | Confirm repository root and `.dijiang/` presence | Stop and ask for project initialization or correct working directory |
| No active task exists | Route the user request through `dj-dispatch` | Create a task only through DiJiang dispatch/start flow |
| Active task exists but artifacts are missing | Read `task.json` and list present files | Route to `dj-hunt` only when `task.json` is missing/unreadable or status is invalid; optional docs may be absent |
| Workflow state conflicts with task status | Prefer `.dijiang/tasks/<task>/task.json` as source of truth | Record conflict and output `follow-up: dj-hunt` if execution would be unsafe |
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
| Do not implement code during startup | Output the route returned by `dj-dispatch` first |
| Do not invent an active task when DiJiang reports none | Use `dj-dispatch` to create or select one |
| Do not ignore dirty git state | Report it and require a task worktree before edits |
| Do not write durable memory from guesses | Keep uncertain context in the task artifact |
| Do not duplicate `dj-dispatch` classification tables in startup | Delegate ambiguous and new-task routing to `dj-dispatch` |
