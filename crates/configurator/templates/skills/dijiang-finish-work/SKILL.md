---
name: dijiang-finish-work
description: "收尾当前 session：质量验证、版本决策、范围一致的提交、可用时 push/merge、归档任务。"
---
# Finish Work

收尾当前 DiJiang session。正常流程里，提交、push、merge 和删除任务 worktree 只在这里发生。

## Steps

1. **验证质量**：如果写过或改过代码，先运行 `dj-check`。检查失败时不得结束任务，除非已经明确记录阻塞原因。

2. **确认 Git 隔离**：
   ```bash
   git status --short --branch
   git worktree list
   git diff --stat HEAD
   git diff --name-only HEAD
   ```
   如果当前目录是主 checkout，或 diff 混入无关文件，停止收尾。

3. **决定版本影响**：
   - `major`：不兼容的公开 API 或行为变更。
   - `minor`：向下兼容的功能新增。
   - `patch`：向下兼容的问题修复。
   - `none`：文档、测试、内部流程，或未发布 package 的变更。
   只有项目存在版本元数据且版本决策不是 `none` 时，才更新版本文件。

4. **同步工件**：行为发生变化时，更新 task notes、spec、文档或 changelog。无需同步时，记录这个决策。

5. **提交范围一致的变更**：
   ```bash
   git add <reviewed paths>
   git diff --cached --stat
   git commit -m "<type>(<scope>): <actual behavior change>"
   ```
   commit 内容必须和任务一致，不能混入无关文件。commit message 写实际行为变化，不列文件名。

6. **可用时 push 与集成**：
   ```bash
   git push -u origin <task-branch>
   git checkout <main-branch>
   git merge --no-ff <task-branch>
   git push origin <main-branch> --tags
   git worktree remove <task-worktree-path>
   ```
   如果凭证、remote 策略、CI 或冲突阻塞 push/merge，停止并报告具体阻塞，保留分支和 worktree。

7. **关闭 DiJiang 状态**：
   ```bash
   dijiang task status <name> completed
   dijiang finish-work --verification "<commands or manual checks>"
   dijiang mem findings --finding "<key decisions and learnings>"
   dijiang mem archive
   ```
