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
- **自动校验**：Call the Skill tool with "dijiang-finish-work" 提交前会检查 message 是否含中文字符，不含则拒绝；同时 `.git/hooks/commit-msg` 做 git 层面兜底
- 一个功能改完、验证通过后，由 `dijiang-finish-work` 统一提交
- 不在实现中途 commit
- 不把文件名堆进 commit message
- **正面示例**：`feat(dj-finish-work): commit message 规则全面更新`
- **反面示例**：`update skill` / `fix bug` / `修改了 dj-finish-work 和 dj-implement 两个文件`
- 不把文件名堆进 commit message

## 链式工单 worktree 基线陷阱

**链式工单**指工单 B 基于工单 A 的分支构建。`git worktree add` 快照的是**分支 HEAD**，不是 A 工作区的未提交改动——直接建 B 的 worktree 会拿到旧基线，A 的改动全部缺失。

正确顺序：先把 A 提交到**它的任务分支**，再建 B 的 worktree。提交到任务分支（非 main、不 push）是链式构建的必要环节，不算越权 finish-work。

若 `worktree add -b` 因分支已存在（指向旧 HEAD）失败：先 `git worktree remove` 掉误建 worktree，再 `git branch -f <branch> <新基线>` 移动指针，最后不带 `-b` 重新挂载。

## 配置值改动：gitignore 分层模式

改配置值（调参/开关）前先 `git check-ignore <runtime配置路径>` 确认其 git 状态。采用 example 模板约定的项目（`.gitignore` 含 `config/*.yaml` + `!config/*.example.yaml`），worktree 里**不存在** runtime 配置——改动必须分两层落地：

1. **tracked 层**（worktree，进 commit）：代码默认值（如 pydantic `Field(default=...)`）+ `*.example.yaml` 模板同步
2. **untracked 层**（主工作区直接改，不进 commit）：本地 runtime 实例——它才是实际生效的，只改 tracked 层会让本地继续读旧值

冒烟两层都要跑：worktree 副本（把本地配置复制进去）与主工作区实际文件各自加载解析、断言目标值。默认值上调前 grep 测试：fixture 显式传值的不受影响，但断言裸默认值的会挂。

## worktree 依赖借用（缺 node_modules/.venv 时）

worktree 缺 gitignore 掉的运行依赖，且非 NTFS 盘不支持 junction/symlink 借用时：

1. **typecheck/build**：临时把改动文件复制到主工作区对应路径执行，验证后 `git checkout --` 恢复（复制前确认主工作区该文件 clean）
2. **后端/测试**：主工作区 `.venv\Scripts\python.exe` 直接跑 worktree 代码（import 按 cwd 解析，无需复制 venv）
3. **UI 实测**：worktree 起后端（临时 config：新生成 Fernet key + 固定 JWT secret + 固定端口），主工作区起 vite dev（`VITE_PROXY_TARGET` 指向后端）+ 临时同步前端改动文件，测完恢复主工作区