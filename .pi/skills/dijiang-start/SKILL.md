---
name: dijiang-start
description: "Initialize a DiJiang session: load project context, active task, workflow state, then delegate to dj-dispatch for task routing."
triggers:
  - session:start
---

# Start Session

Initialize a DiJiang-managed development session.

## Step 1: Load Project State

```bash
dijiang status
```

Note: active task path, current phase, git status, developer identity.

## Step 2: Read Workflow

```bash
cat .dijiang/workflow.md
```

DiJiang workflow: **dispatch → grill → output → implement/tdd → hunt ↔ check**

## Step 3: Discover Specs

```bash
ls .dijiang/spec/
cat .dijiang/spec/index.md
```

## Step 4: Initialize Memory Session

Record initial findings if the user has already given a task description:

```bash
dijiang mem findings --finding "<initial task description>"
```

## Step 5: Delegate to dj-dispatch

This skill only sets up session context. All task classification, routing,
and execution is handled by the `dj-*` skill ecosystem.
Load `dj-dispatch` to classify the user's request and route to the
appropriate `dj-*` skill:

| Request type | Routes to |
|---|---|
| New feature / unclear requirements | `dj-grill` (deep alignment) |
| Feature implementation | `dj-implement` or `dj-tdd` |
| Bug / regression / crash | `dj-hunt` (systematic investigation) |
| Code review / quality check | `dj-check` |
| Documentation / PRD / design | `dj-output` |
| Refactoring / code quality | `dj-ponytail` |
| Prototype / spike | `dj-prototype` |
| Tech debt tracking | `dj-debt` |
| UI design | `dj-design` |
| Script / tooling | `dj-script` |

Ask `dj-dispatch` to classify, then follow its routing.
