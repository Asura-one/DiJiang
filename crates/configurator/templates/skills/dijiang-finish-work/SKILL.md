---
name: dijiang-finish-work
description: "收尾当前 session：质量验证、版本决策、范围一致的提交、可用时 push/merge、归档任务。"
---
# Finish Work

收尾当前 DiJiang session。提交、push、merge、归档和 worktree 清理只在这里发生。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Verified diff, task status, validation output, docs/spec sync evidence, version decision, and reviewed paths |
| 输出 | Completed task, scoped commit when requested, archived memory/session, and push/merge/worktree cleanup when explicitly requested |
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
Docs/spec sync: <updated / none + reason>
Version decision: <major|minor|patch|none>
Commit mode: <--commit yes/no>
Integration mode: <--push/--integrate yes/no and reason>
```

🛑 STOP if validation is missing, docs/spec sync evidence is missing for changed work, the current directory is the main checkout for integration, changed files include unrelated work, or the version decision is unclear.

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

Memory entries must pass source, scope, confidence, freshness, conflict, and actionability checks. If they do not pass, keep them in task notes.

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

`--commit` stages the current task diff with `git add --all`, archives the task, writes finish journals, commits the resulting diff, and prints the commit hash. Do not use `--allow-dirty` with `--commit`.

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
