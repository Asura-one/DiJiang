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

- 一个功能改完、验证通过后，由 `dijiang-finish-work` 统一提交
- 不在实现中途 commit
- 不把文件名堆进 commit message
