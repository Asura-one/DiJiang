---
name: dijiang-finish-work
description: "收尾当前 session：质量验证、版本决策、范围一致的提交、可用时 push/merge、归档任务。"
---
# Finish Work

收尾当前 DiJiang session。提交、push、merge、归档和 worktree 清理只在这里发生。

## 入口边界

| 入口 | 职责 | 执行语义 |
|---|---|---|
| `/dijiang-finish-work` | Pi prompt checklist | 只把收尾检查清单注入对话，不读取状态、不运行命令、不归档任务 |
| `/skill:dijiang-finish-work` | Agent skill workflow | 加载本文件后必须按 Invocation Contract 立即检查状态、git 隔离、版本和 docs-sync；只有 gate 完成后才调用 CLI |
| `dijiang finish-work ...` | DiJiang CLI | 执行 finish-work state transition：有 active task 时归档任务并清理 session；无 active task 时跳过归档但仍可验证、记录、commit/push/integrate |

`dijiang finish-work` 可以在没有 active task 时执行，此时只跳过 DiJiang task 归档和 active-task 清理；验证、docs-sync、version decision、session closure、commit、push/integrate 仍按参数执行。active task 指向缺失 artifact 时表示 task state 已陈旧，必须先修复，不要把 prompt 或 skill 当作任务存储的替代品。

## Invocation Contract

When this skill is loaded through `/skill:dijiang-finish-work`, do not summarize the skill or ask whether to continue. Start the finish-work workflow immediately.

First actions, in order:

1. Run `dijiang task current`, `git status --short --branch`, `git worktree list`, `git diff --stat HEAD`, `git diff --name-only HEAD`, and `git log --oneline @{u}..HEAD` when an upstream exists.
2. Read the active task artifacts when a task exists: `.dijiang/tasks/<name>/task.json`, `prd.md`, `design.md`, and `implement.md` when present.
3. Classify the state as `ready-to-finish`, `no-task-finish`, `blocking`, or `nothing-to-archive`.
4. Use `no-task-finish` when there is no active task but `git diff --name-only HEAD` shows reviewed changes. In this mode, `dijiang finish-work` may still run; it will skip task archive and active-task cleanup while preserving validation, docs-sync, version decision, commit, push, and integration semantics.
5. Use `nothing-to-archive` only when there is no active task, no changed files, and no unpushed commits.
6. If `nothing-to-archive`, output the Clean Status format below and stop. Do not print the pre-finish gate, do not ask for confirmation, and do not call `dijiang finish-work`.
7. If blocking, stop with `blocking: <reason>` and the exact command or file that proves it.
8. If ready or no-task, produce the pre-finish gate from step 2, including validation, docs-sync evidence, version decision, commit mode, and integration mode.

Only call `dijiang finish-work ...` after the pre-finish gate is complete and the git scope is reviewed. If there is no active task, pass the same verification/docs/version/commit flags and expect task archive to be skipped. If validation, docs-sync evidence, version decision, or review scope is missing, stop instead of committing.
## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Verified diff, task status, validation output, docs/spec sync evidence, version decision, and reviewed paths |
| 输出 | Completed task, scoped commit when requested, archived memory/session, and push/merge/worktree cleanup when explicitly requested |
| 非目标 | Do not fix new bugs, broaden scope, or include unrelated files during finish work |

## Steps

### 1. Verify State and Quality

Start by inspecting the actual project state. Do not rely on conversation history alone.

Required commands:

```bash
dijiang task current
git status --short --branch
git worktree list
git diff --stat HEAD
git diff --name-only HEAD
git log --oneline @{u}..HEAD
```

If code or behavior changed, run the relevant test, typecheck, lint, or `dj-check` before finishing. A failed check blocks finish-work unless the blocker is recorded and the task is intentionally left unfinished.

Required output: active task state, changed files, unpushed commits, validation commands, pass/fail result, and unverified areas.

### Clean Status Output

When there is no active task, no changed files, and no unpushed commits, use exactly this shape:

```text
Clean: no finish-work action needed.
Active task: none
Uncommitted changes: none
Unpushed commits: none
Docs/spec sync: skipped; reason=no changed files
Version decision: none; reason=no changed files
Memory: skipped; reason=no new verified, reusable finding
Current branch: <branch status line from git status --short --branch>
Action: skipped dijiang finish-work because there is nothing to archive, commit, push, integrate, document, version, or remember.
```

### 2. 🔴 CHECKPOINT · Pre-finish Gate

暂存任何文件前先报告：

```text
Task: <name or none; mode: dijiang-finish / no-task-finish / nothing-to-archive>
Branch/worktree: <branch> / <path>
Changed files: <paths>
Validation: <commands => result>
Docs/spec sync: <updated / none / skipped; reason=...>
Version decision: <major|minor|patch|none; reason=...>
Memory: <written / skipped; reason=...>
Commit mode: <--commit yes/no; reason=...>
Integration mode: <--push/--integrate yes/no; reason=...>
```

🛑 STOP if validation is missing, docs/spec sync evidence is missing for changed work, memory decision is missing, the current directory is the main checkout for integration, changed files include unrelated work, or the version decision is unclear.

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

When behavior changed, update task notes, spec, docs, or changelog before finishing. `dijiang finish-work --commit` requires `--docs-sync "<evidence>"`; use `--docs-sync "none: <reason>"` only after checking that no docs/spec/changelog update is needed.

The gate must state the docs/spec decision with a reason: `updated; reason=<files>` when artifacts changed, `none; reason=<checked scope>` when no docs update is needed for changed work, or `skipped; reason=no changed files` only for clean state.

The gate must state the version decision with a reason. Use `none; reason=no publishable behavior change` for internal/docs/test/workflow changes, or `none; reason=no changed files` for clean state.

The gate must state the memory decision with a reason. Use `written; reason=<finding>` only for verified, reusable project knowledge. Use `skipped; reason=no new verified, reusable finding` when the work has no durable lesson. Memory entries must pass source, scope, confidence, freshness, conflict, and actionability checks; if they do not pass, keep them in task notes.

### 6. Commit Reviewed Scope

Use CLI automation only after the scope has been reviewed:

```bash
dijiang finish-work \
  --verification "<commands or manual checks>" \
  --docs-sync "<updated docs/spec or none: reason>" \
  --version-impact <major|minor|patch|none> \
  --commit \
  --commit-message "<type>(<scope>): <actual behavior change>"
```

`--commit` stages the current reviewed diff with `git add --all`, writes finish journals, commits the resulting diff, and prints the commit hash. It archives the task only when an active task exists; without an active task, task archive and active-task cleanup are skipped. Do not use `--allow-dirty` with `--commit`.

If no code or artifact commit is needed, record `commit: none` and run finish-work without `--commit`.

### 7. Push and Integrate When Available

Push and integration are explicit flags on top of `--commit`:

```bash
dijiang finish-work \
  --verification "<commands>" \
  --docs-sync "<evidence>" \
  --version-impact <major|minor|patch|none> \
  --commit \
  --push \
  --integrate \
  --main-branch main \
  --remote origin
```

`--push` pushes the task branch. `--integrate` merges the task branch into the main branch worktree with `--no-ff`, removes the task worktree, and deletes the merged branch. If credentials, remote policy, CI, conflicts, or user permission block integration, stop integration, report the blocker, and preserve the branch and worktree.

### 8. Close DiJiang State

```bash
dijiang task status <name> completed
dijiang finish-work --verification "<commands or manual checks>" --docs-sync "<docs/spec evidence>" --version-impact none --commit
dijiang mem findings --finding "<key decisions and learnings; source=task; scope=project; confidence=verified>"
dijiang mem archive
```

Skip durable memory when the finding lacks future actionability.

## Failure Handling

| Trigger | First action | Fallback |
|---|---|---|
| `dj-check` fails | Stop finish-work and output `blocking: validation failed; follow-up: implementation or investigation` | Record blocker and leave task in progress |
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
