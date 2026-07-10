# Git 安全工作流参考

## AI Coding 适配的工作流

由于 AI agent 负责写代码到 worktree，而用户在固定窗口测试，测试必须在 worktree 目录中执行。worktree 是完整的仓库检出，Makefile 等已有文件自动继承，无需额外创建。

```
┌─ AI agent 窗口 ─────────────────┐
│ 在 worktree 中开发              │
│ git worktree add ../proj-fix ... │
│ → 修改代码                     │
│ → 告知用户 worktree 路径        │
└─────────────────────────────────┘
         ↓
┌─ 用户测试窗口 ──────────────────┐
│ cd ../proj-fix                  │
│ make dev / make test            │
│ 浏览器/终端验证                 │
└─────────────────────────────────┘
```

## worktree 操作

```bash
# 创建 worktree（AI agent 执行）
git worktree add ../<项目名>-<分支名> -b <type>/<任务名> <base>

# AI agent 告知用户 worktree 绝对路径
echo "测试目录：$(pwd)"
```

## 提交

- 所有 commit message **基于变更事实总结**：先执行 `git diff --stat HEAD` 和 `git diff HEAD` 获取实际修改内容，描述行为变化而非堆文件名
- 所有 commit message **使用中文编写**，遵循 Conventional Commits 格式 `<类型>(<范围>): 中文描述`
  - 类型：`feat`（新功能）、`fix`（修复）、`refactor`（重构）、`docs`（文档）、`test`（测试）、`chore`（杂项）等
  - 范围：受影响的模块或目录名
  - 描述：一句话概括实际变更，不超过 72 字
- 一个功能改完、验证通过后，由 `dijiang-finish-work` 统一提交
- 不在实现中途 commit
- 不把文件名堆进 commit message
- **正面示例**：`feat(dj-finish-work): commit message 规则全面更新`
- **反面示例**：`update skill` / `fix bug` / `修改了 dj-finish-work 和 dj-implement 两个文件`
- 不把文件名堆进 commit message
