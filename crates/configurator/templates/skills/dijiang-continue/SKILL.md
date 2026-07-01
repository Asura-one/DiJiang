---
name: dijiang-continue
description: "Resume work on the current task: find active task and phase, load artifacts, then report the appropriate dj-* route."
triggers:
  - session:start
---

# Continue Session

Resume work on the current DiJiang task. This skill reconstructs context and reports the next workflow route; it does not implement or finish work directly.

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Existing DiJiang task state, prior task artifacts, workspace journal, and current user request |
| 输出 | Restored context summary, current phase, loaded artifacts, and one phase-appropriate `dj-*` route |
| 非目标 | Do not edit code, create a new plan, or close a task from this skill |

## Steps

### 1. Load State

```bash
dijiang status
dijiang task current
dijiang workflow-state --json
git status --short --branch
```

Output: active task name, task status, workflow phase, repository branch, and dirty state.

If `dijiang task current` reports no active task, do not invent prior context. Report `follow-up: dj-dispatch` for classification of the current user request.

### 2. Load Memory Context

Restore project context from DiJiang workspace memory when present.

```bash
dijiang mem list
```

If memory commands fail, continue from task artifacts and mention `memory unavailable` in the restored context summary.

### 3. Read Active Task Artifacts

Read the active task directory in this order:

1. `task.json`
2. `prd.md` when present
3. `design.md` when present
4. `implement.md` when present
5. `check.md` or handoff artifacts when present

Output: task goal, status, known decisions, verification state, and unresolved blockers.

If `task.json` is missing, stop and output `blocking: task state corrupt; follow-up: dj-hunt`. Continuing would create false context.

### 4. Read Journal Context

Read `.dijiang/workspace/{{developer}}/` only for recent entries that reference the active task.

If the journal conflicts with `task.json`, prefer `task.json` and record the conflict in the context summary.

### 5. Select the Next Skill

| Phase / status | Skill | Required loaded context |
|---|---|---|
| requirements alignment | `dj-grill` | task goal and open questions |
| document creation | `dj-output` | PRD/design/doc target |
| implementation | `dj-implement` or `dj-tdd` | implement plan and verification loop |
| investigation / debugging | `dj-hunt` | symptom, reproduction, evidence |
| review / verification | `dj-check` | diff, requirements, validation output |
| completed | `dijiang-finish-work` | verification summary and version decision |

If the phase is ambiguous, output `follow-up: dj-grill` for alignment unless there is a concrete failure signal; concrete failures output `follow-up: dj-hunt`.

## Failure Handling

| Trigger | First action | Fallback |
|---|---|---|
| No active task | Output `follow-up: dj-dispatch` | Ask for task selection only if dispatch classification cannot infer intent |
| `task.json` missing | Stop normal continuation | Output `blocking: task state corrupt; follow-up: dj-hunt` |
| Artifact referenced but absent | Mark it missing in summary | Continue only when the next skill can operate without it |
| Journal contradicts task status | Prefer `task.json` | Record conflict and output `follow-up: dj-hunt` when unsafe |
| Git dirty state predates this session | List dirty paths | Require worktree/scope decision before implementation |
| Workflow state command fails | Fall back to `task.json.status` | Mark workflow state as degraded in summary |

## 🔴 CHECKPOINT · Context Restored

离开该 skill 前先报告：

```text
Active task: <name>
Status / phase: <status> / <phase>
Loaded artifacts: <paths>
Missing artifacts: <paths or none>
Dirty state: <summary>
Route: <dj-skill>
Next action: <one sentence>
```

🛑 STOP if active task state is corrupt, route selection is ambiguous, or continuing would require guessing prior intent.

## Anti-patterns

| Do not | Do this instead |
|---|---|
| Do not treat missing task files as empty requirements | Stop and output `blocking: task state corrupt; follow-up: dj-hunt` |
| Do not continue implementation from memory alone | Load task artifacts first |
| Do not overwrite the current phase because a journal says something older | Prefer `task.json` and record conflicts |
| Do not create a new task while an active task exists | Resume or explicitly route through dispatch |
| Do not finish work from continue mode | Output `follow-up: dijiang-finish-work` for completed tasks |
