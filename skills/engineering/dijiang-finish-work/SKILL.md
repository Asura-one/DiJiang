---
name: dijiang-finish-work
name_cn: 收尾工作
description_cn: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。"
description: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。"
disable-model-invocation: true
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '98b087bd-b56f-4463-947d-538b55e7cfe9'
  PropagateID: '98b087bd-b56f-4463-947d-538b55e7cfe9'
  ReservedCode1: '7165379b-57b6-45fc-821e-42211c127833'
  ReservedCode2: '7165379b-57b6-45fc-821e-42211c127833'
---

# 收尾工作

收尾当前 DiJiang session。提交、归档只在这里发生。本 skill 不依赖 CLI，所有操作通过文件读写和 git 命令完成。

## 调用契约

加载本 skill 后，不要总结 skill，也不要询问是否继续。立即开始 finish-work 流程。

## 步骤

### 1. 验证状态和质量

先检查真实项目状态，不依赖对话历史。

```bash
test -f .dijiang/active_task.txt && cat .dijiang/active_task.txt
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
git log --oneline @{u}..HEAD 2>/dev/null || true
```

如果有活跃任务，读取 `.dijiang/tasks/<name>/task.json` 和产物文件。

如果代码或行为发生变化，收尾前运行相关测试、typecheck、lint。检查失败会阻塞 finish-work。

代码或行为变更必须在验证输出中包含 **Code Task TDD Contract** 证据：RED/Repro evidence、GREEN command、Regression scope 和 Exception。纯文档、文本、格式化或无代码变更可将这些字段设为 `n/a`，并写明原因。

### 清洁状态

没有活跃任务、没有变更文件、也没有未 push commit 时，直接报告清洁状态并停止。

### 2. Pre-finish Gate

暂存任何文件前先报告：

```text
任务: <name or none>
分支: <branch>
变更文件: <paths>
验证: <commands => result>
RED/Repro evidence: <command or n/a + reason>
GREEN command: <command> => <result>
Regression scope: <checks> => <result>
Exception: <none or justified gap>
文档同步: <updated / none / skipped; reason=...>
版本决策: <major|minor|patch|none; reason=...>
记忆: <written / skipped; reason=...>
提交模式: <yes/no; reason=...>
```

如果缺少验证、代码变更缺少 TDD evidence、变更工作缺少文档同步证据、缺少记忆决策、版本决策不清楚，停止。

### 3. 确认 Git 隔离

```bash
git worktree list 2>/dev/null
git rev-parse --show-toplevel
```

如果当前在主 checkout 且有代码变更，停止。diff 混入无关文件时，只暂存已审查路径。

### 4. 判断版本影响

| 决策 | 使用场景 |
|---|---|
| major | 不兼容的公开 API 或行为变更 |
| minor | 向后兼容的功能新增 |
| patch | 向后兼容的 bug 修复 |
| none | 文档、测试、内部 workflow 变更 |

只有存在版本元数据且决策不是 `none` 时，才更新版本文件和 CHANGELOG。

### 5. 同步产物

行为变化时，收尾前更新任务记录、spec、docs 或 changelog。

gate 必须说明文档同步决策和原因。

### 6. 提交已审查范围

提交前先执行 `git diff --stat HEAD` 获取变更事实；commit message 必须基于变更事实总结。

所有 commit message 使用中文编写，遵循 Conventional Commits 格式 `<类型>(<范围>): 中文描述`。

```bash
git add <reviewed paths>
git commit -m "<类型>(<范围>): 中文描述"
```

### 7. 清理 worktree 与分支

提交（及 merge，如有）完成后，清理本次任务使用的 worktree 与已合并分支：

```bash
git worktree list
git worktree remove <path>           # 已合并的 worktree
git worktree remove --force <path>   # 有未跟踪/残留文件时，先确认改动已提交再强制移除
git branch -d <branch>               # 只删除已合并分支
```

- 只清理已合并（或用户确认不再需要）的 worktree；未合并的保留，并向用户报告分支名与 worktree 路径。
- 主 checkout 永不清理。

### 8. 归档任务

如果有活跃任务，将 task.json 的 status 更新为 `archived`，并从 `.dijiang/active_task.txt` 移除指针。

```bash
# 更新 task.json status 为 archived
# 移除 .dijiang/active_task.txt
```

### 9. 记录记忆

调用 `dj-memory` skill 记录本次 session 的发现和经验。

Call the Skill tool with "dj-memory" to store findings and learnings.

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要从主 checkout 提交代码变更 | 只从任务 worktree 收尾 |
| 不要盲目 `git add .` | 暂存已审查路径 |
| 不要隐藏验证失败 | 报告命令和失败 |
| 不要写 "fixed bug" 这类模糊记忆 | 写入有 source、scope、已验证的 finding |
| 不要带着无关脏改关闭任务 | 拆分或清理范围 |
| 不要留着已合并的 worktree 和分支不管 | 提交/merge 后清理 worktree、删除已合并分支 |