---
name: dijiang-finish-work
description: "收尾当前 session：质量验证、版本决策、范围一致的提交、可用时 push/merge、归档任务。"
---
# Finish Work

收尾当前 DiJiang session。提交、push、merge、归档和 worktree 清理只在这里发生。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Verified diff, task status, validation output, version decision, and reviewed paths |
| 输出 | Completed task, scoped commit when needed, archived memory/session, and preserved branch/worktree if integration is blocked |
| 非目标 | Do not fix new bugs, broaden scope, or include unrelated files during finish work |

## Steps

### 1. Verify Quality

If code or behavior changed, run `dj-check` before finishing. A failed check blocks finish-work unless the blocker is recorded and the task is intentionally left unfinished.

Required output: validation commands, pass/fail result, and unverified areas.

### 2. 🔴 CHECKPOINT · Pre-finish Gate

暂存任何文件前先报告：

```text
Task: <name>
Branch/worktree: <branch> / <path>
Changed files: <paths>
Validation: <commands => result>
Version decision: <major|minor|patch|none>
Commit needed: <yes|no>
Integration allowed: <yes|no and reason>
```

🛑 STOP if validation is missing, the current directory is the main checkout, changed files include unrelated work, or the version decision is unclear.

### 3. Confirm Git Isolation

```bash
git status --short --branch
git worktree list
git diff --stat HEAD
git diff --name-only HEAD
```

If current directory is the main checkout, stop. If diff mixes unrelated files, stage only reviewed paths or split the task before finishing.

### 4. Decide Version Impact

| Decision | Use when | Action |
|---|---|---|
| `major` | incompatible public API or behavior change | update version metadata when package is publishable |
| `minor` | backward-compatible feature addition | update version metadata when package is publishable |
| `patch` | backward-compatible bug fix | update version metadata when package is publishable |
| `none` | docs, tests, internal workflow, or unpublished package change | do not edit version files |

Only update version files when version metadata exists and the decision is not `none`.

### 5. Sync Artifacts

When behavior changed, update task notes, spec, docs, or changelog. When no artifact needs syncing, record `docs/spec sync: none` with the reason.

Memory entries must pass source, scope, confidence, freshness, conflict, and actionability checks. If they do not pass, keep them in task notes.

### 6. Commit Reviewed Scope

```bash
git add <reviewed paths>
git diff --cached --stat
git diff --cached --name-only
git commit -m "<type>(<scope>): <actual behavior change>"
```

Commit content must match the task. The message describes behavior change, not a file list.

If no code or artifact commit is needed, record `commit: none` and skip to DiJiang status closure.

### 7. Push and Integrate When Available

```bash
git push -u origin <task-branch>
git checkout <main-branch>
git merge --no-ff <task-branch>
git push origin <main-branch> --tags
git worktree remove <task-worktree-path>
```

If credentials, remote policy, CI, conflicts, or user permission block integration, stop integration, report the blocker, and preserve the branch and worktree.

### 8. Close DiJiang State

```bash
dijiang task status <name> completed
dijiang finish-work --verification "<commands or manual checks>"
dijiang mem findings --finding "<key decisions and learnings; source=task; scope=project; confidence=verified>"
dijiang mem archive
```

Skip durable memory when the finding lacks future actionability.

## Failure Handling

| Trigger | First action | Fallback |
|---|---|---|
| `dj-check` fails | Stop finish-work and return to implementation or investigation | Record blocker and leave task in progress |
| Main checkout detected | Stop before staging | Move work to task worktree or report manual cleanup needed |
| Diff contains unrelated files | Stage only reviewed paths | Split unrelated files into another task |
| Version decision unclear | Re-read task scope and package metadata | Use `none` only when no publishable behavior changed |
| Commit fails | Show `git status` and staged diff | Unstage, fix scope, and retry once |
| Push/merge blocked | Preserve branch/worktree and report exact blocker | Leave integration for user or CI owner |
| Memory quality gate fails | Keep note in task artifact | Do not write durable memory |

## Anti-patterns

| Do not | Do this instead |
|---|---|
| Do not commit from the main checkout | Finish only from the task worktree |
| Do not stage `git add .` blindly | Stage reviewed paths or hunks |
| Do not hide failed validation in the final message | Report the command and failure |
| Do not push or merge without permission, credentials, and clean scope | Preserve branch/worktree and report blocker |
| Do not write vague memory such as "fixed bug" | Write source-scoped, verified, actionable findings |
| Do not close a task with unrelated dirty files | Split or clean scope first |
