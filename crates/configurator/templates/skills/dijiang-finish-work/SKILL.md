---
name: dijiang-finish-work
description: "收尾当前 session：质量验证、版本决策、范围一致的提交、可用时 push/merge、归档任务。"
---
# 收尾工作

收尾当前 DiJiang session。提交、push、merge、归档和 worktree 清理只在这里发生。

## 入口边界

| 入口 | 职责 | 执行语义 |
|---|---|---|
| `/dijiang-finish-work` | Pi prompt checklist | 只把收尾检查清单注入对话，不读取状态、不运行命令、不归档任务 |
| `/skill:dijiang-finish-work` | Agent skill workflow | 加载本文件后必须按 Invocation Contract 立即检查状态、git 隔离、版本和 docs-sync；只有 gate 完成后才调用 CLI |
| `dijiang finish-work ...` | DiJiang CLI | 执行 finish-work state transition：有 active task 时归档任务并清理 session；无 active task 时跳过归档但仍可验证、记录、commit/push/integrate |

`dijiang finish-work` 可以在没有 active task 时执行，此时只跳过 DiJiang task 归档和 active-task 清理；验证、docs-sync、version decision、session closure、commit、push/integrate 仍按参数执行。active task 指向缺失 artifact 时表示 task state 已陈旧，必须先修复，不要把 prompt 或 skill 当作任务存储的替代品。

## 调用契约

通过 `/skill:dijiang-finish-work` 加载本 skill 时，不要总结 skill，也不要询问是否继续。立即开始 finish-work 流程。

按顺序先执行：

1. 运行 `dijiang task current`、`git status --short --branch`、`git worktree list`、`git diff --stat HEAD`、`git diff --name-only HEAD`；存在 upstream 时运行 `git log --oneline @{u}..HEAD`。
2. 有 active task 时读取任务产物： `.dijiang/tasks/<name>/task.json`, `prd.md`, `design.md`, and `implement.md` when present.
3. 将状态分类为 `ready-to-finish`, `no-task-finish`, `blocking`, or `nothing-to-archive`.
4. 没有 active task 但 `git diff --name-only HEAD` 存在已审查变更时使用 `no-task-finish`。此模式仍可运行 `dijiang finish-work`；它会跳过任务归档和 active-task 清理，但保留验证、docs-sync、版本决策、commit、push 和集成语义。
5. 仅当没有 active task、没有改动文件、也没有未 push commit 时，才使用 `nothing-to-archive`。
6. 如果是 `nothing-to-archive`，输出下面的清洁状态格式后停止。不要打印 pre-finish gate，不要询问确认，不要调用 `dijiang finish-work`。
7. 如果 blocking，用 `blocking: <reason>` 停止，并给出证明它的具体命令或文件。
8. 如果 ready 或 no-task，按第 2 步输出 pre-finish gate，包含验证、代码工作的 TDD evidence、docs-sync 证据、版本决策、commit 模式、集成模式和 worktree 残留决策。
只有 pre-finish gate 完整且 git 范围已审查后，才调用 `dijiang finish-work ...`。没有 active task 时，传入同样的 verification/docs/version/commit 参数，并预期任务归档会被跳过。缺少验证、docs-sync 证据、版本决策、审查范围或 worktree 残留决策时，停止而不是提交。
## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | 已审查 diff、任务状态、验证输出、docs/spec 同步证据、版本决策和已审查路径 |
| 输出 | 完成的任务、按需生成的范围一致提交、归档的 memory/session，以及 push/merge/worktree 清理。除非 gate 记录 blocker 或明确保留原因，finish-work 后不得残留任务 worktree。 |
| 非目标 | 不修复新 bug、不扩大范围、不在收尾中包含无关文件 |

## 步骤

### 1. 验证状态和质量

先检查真实项目状态，不依赖对话历史。

必需命令：

```bash
dijiang task current
git status --short --branch
git worktree list
git diff --stat HEAD
git diff --name-only HEAD
git log --oneline @{u}..HEAD
```

如果代码或行为发生变化，收尾前运行相关测试、typecheck、lint 或 `dj-check`。检查失败会阻塞 finish-work，除非已记录 blocker 且任务有意保持未完成。

代码或行为变更必须在验证输出中包含 **Code Task TDD Contract** 证据：RED/Repro evidence、GREEN command、Regression scope 和 Exception。纯文档、文本、格式化或无代码变更可将这些字段设为 `n/a`，并写明原因。

必需输出：active task 状态、变更文件、未 push commit、验证命令、TDD evidence、通过/失败结果和未验证区域。

### 清洁状态输出

没有 active task、没有变更文件、也没有未 push commit 时，严格使用此格式：

```text
清洁: 无需执行 finish-work。
当前任务: none
未提交改动: none
未 push commit: none
文档/spec 同步: skipped; reason=no changed files
版本决策: none; reason=no changed files
记忆: skipped; reason=no new verified, reusable finding
当前分支: <branch status line from git status --short --branch>
动作: 跳过 dijiang finish-work，因为没有需要归档、提交、push、集成、记录文档、调整版本或写入记忆的内容。
```

### 2. 🔴 CHECKPOINT · Pre-finish Gate

暂存任何文件前先报告：

```text
任务: <name or none; mode: dijiang-finish / no-task-finish / nothing-to-archive>
分支/worktree: <branch> / <path>
变更文件: <paths>
验证: <commands => result>
RED/Repro evidence: <command/checklist or n/a + reason>
GREEN command: <command/checklist => result>
Regression scope: <commands/reference checks/sibling paths => result>
Exception: <none or justified gap>
文档/spec 同步: <updated / none / skipped; reason=...>
版本决策: <major|minor|patch|none; reason=...>
记忆: <written / skipped; reason=...>
提交模式: <--commit yes/no; reason=...>
集成模式: <--push/--integrate yes/no; reason=...>
worktree 残留: <auto-cleaned via commit / retained via --keep-worktree / n/a main checkout>
```

🛑 如果缺少验证、代码/行为/模板/脚本变更缺少 TDD evidence、变更工作缺少 docs/spec 同步证据、缺少记忆决策、当前目录是用于集成的主 checkout、变更文件包含无关工作、版本决策不清楚，或 worktree 残留未决，停止。

### 3. 确认 Git 隔离

```bash
git status --short --branch
git worktree list
git diff --stat HEAD
git diff --name-only HEAD
```

如果当前目录是主 checkout，停止。diff 混入无关文件时，只暂存已审查路径，或先拆分任务再收尾。

### 4. 判断版本影响

| 决策 | 使用场景 | 动作 |
|---|---|---|
| `major` | 不兼容的公开 API 或行为变更 | package 可发布时更新版本元数据 |
| `minor` | 向后兼容的功能新增 | package 可发布时更新版本元数据 |
| `patch` | 向后兼容的 bug 修复 | package 可发布时更新版本元数据 |
| `none` | 文档、测试、内部 workflow 或未发布 package 变更 | 不编辑版本文件 |

只有存在版本元数据且决策不是 `none` 时，才更新版本文件。

### 5. 同步产物

行为变化时，收尾前更新任务记录、spec、docs 或 changelog。`dijiang finish-work --commit` 要求提供 `--docs-sync "<evidence>"`；只有确认无需 docs/spec/changelog 更新后，才使用 `--docs-sync "none: <reason>"`。

gate 必须说明 docs/spec 决策和原因：产物变化时使用 `updated; reason=<files>`，变更工作无需文档更新时使用 `none; reason=<checked scope>`，只有清洁状态才使用 `skipped; reason=no changed files`。

gate 必须说明版本决策和原因。内部/docs/test/workflow 变更使用 `none; reason=no publishable behavior change`，清洁状态使用 `none; reason=no changed files`。

gate 必须说明记忆决策和原因。 成功的 `dijiang finish-work` 默认写入 session closure memory record；需与 durable memory 分开报告。 成功关闭时使用 `Memory closure: written; reason=finish-work default`。 只有经过验证且可复用的项目知识才使用 `Durable memory: written; reason=<finding|lesson|correction>`。 工作没有长期可用经验时，使用 `Durable memory: skipped; reason=no new verified, reusable finding`。 会改变未来行为的用户纠正必须用 `dijiang mem correction` 记录，并通过 source、scope、confidence、freshness、conflict 和 actionability 检查；未通过时保留在任务记录中。

### 6. 提交已审查范围

只有范围已审查后，才使用 CLI 自动化：

```bash
dijiang finish-work \
  --verification "RED/Repro evidence: <...>; GREEN command: <...>; Regression scope: <...>; Exception: <none or reason>; commands: <...>" \
  --docs-sync "<updated docs/spec or none: reason>" \
  --version-impact <major|minor|patch|none> \
  --commit \
  --commit-message "<类型>(<范围>): <中文改动描述>"
```

`--commit` 使用 `git add --all` 暂存当前已审查 diff，写入 finish journal，提交结果 diff，并打印 commit hash。只有存在 active task 时才归档任务；没有 active task 时跳过任务归档和 active-task 清理。不要将 `--allow-dirty` 与 `--commit` 一起使用。

如果不需要代码或产物提交，记录 `commit: none`，并在不带 `--commit` 的情况下运行 finish-work。

### 7. 可用时 Push 和集成

Push 和集成是在 `--commit` 基础上的显式参数：

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


`--integrate` 将任务分支以 `--no-ff` 合并到主分支后清理 worktree。`--push` 仅在远端可达且策略允许时推送任务分支。

对于任务 worktree，不传 `--keep-worktree` 时 finish-work 会自动清理 worktree（删除 worktree 和本地分支）。
`--integrate` 额外做合并操作后同样清理 worktree。`--keep-worktree` 仅在需要保留孤立证据时使用。

最终报告必须说明任务 worktree 已删除还是通过 `--keep-worktree` 有意保留。


### 8. 关闭 DiJiang 状态

```bash
dijiang task status <name> completed
dijiang finish-work --verification "<commands or manual checks>" --docs-sync "<docs/spec evidence>" --version-impact none --commit
dijiang mem findings --finding "<key decisions and learnings; source=task; scope=project; confidence=verified>"
dijiang mem archive
```

缺少未来可执行性的发现不写入 durable memory。

## 失败处理

| 触发条件 | 第一动作 | 回退 |
|---|---|---|
| `dj-check` 失败 | 停止 finish-work，并输出 `blocking: validation failed; follow-up: implementation or investigation` | 记录 blocker，并保持任务 in progress |
| 检测到主 checkout | 暂存前停止 | 将工作移到任务 worktree，或报告需要人工清理 |
| Diff 包含无关文件 | 只暂存已审查路径 | 将无关文件拆到其他任务 |
| 版本决策不清楚 | 重新读取任务范围和 package 元数据 | 只有没有可发布行为变化时才使用 `none` |
| Commit 失败 | 显示 `git status` 和 staged diff | 取消暂存，修正范围，并重试一次 |
| Push 被阻塞 | 报告精确 push blocker | merge 安全时继续本地集成和清理，让 main ahead of remote |
| Merge 被阻塞 | 保留 branch/worktree，并报告精确 blocker 和 worktree 残留原因 | 将集成交给用户或 CI owner |
| Memory 质量门禁失败 | 将 note 保留在任务产物中 | 不写入 durable memory |

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要从主 checkout 提交 | 只从任务 worktree 收尾 |
| 不要盲目暂存 `git add .` | 暂存已审查路径或 hunk |
| 不要在最终消息中隐藏验证失败 | 报告命令和失败 |
| 不要把远端不可达当成保留已合并 worktree 的理由 | 安全时本地 merge，清理已合并 worktree/branch，并将 push 作为独立 blocker 报告 |
| 不要写 “fixed bug” 这类模糊 memory | 写入有 source、scope、已验证且可执行的 finding |
| 不要带着无关脏改关闭任务 | 拆分或清理范围 |
