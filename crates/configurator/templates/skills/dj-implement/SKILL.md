---
name: dj-implement
description: >
  按计划实现特性代码，遵守 git 安全工作流。
  Use when the user wants to implement a feature, fix, or change based on a plan or issue.
  触发词：实现、写代码、implement、开始做、按计划做、开发。
---

# Implement: 实现代码

按 PRD / 设计文档 / issue 描述实现。不提交、不 push、不 merge——这些由 `dijiang-finish-work` 处理。

## 工作流

### 1. 准备

- 读取 PRD / 设计文档 / issue，理解要做什么。
- 核心行为不明确 → 停止，报告"需要需求对齐"。
- 查询项目历史教训：`dijiang mem recall --query "<任务摘要>"`，检查是否有相关的 findings/learnings/patterns。
- 确认在专用 worktree 中（主 checkout 只做同步）：
  ```bash
  git worktree add ../<项目名>-<分支名> -b <type>/<任务名> <base>
  ```
- 通知用户 worktree 路径，后续测试在 worktree 目录中执行（Makefile 等已有文件继承自仓库）。
- 遵守 **Code Task TDD Contract**：先固定行为和回归边界，再实现。
- 固定行为边界：先写一个失败测试（RED/Repro evidence）、复现步骤或人工复核清单。
- 确认测试环境可用，或记录不可用原因。

### 2. 实现

**只改必要的代码。** 每一行改动必须能追溯到用户请求、PRD 或验证失败。

顺序：
1. 先读现有代码，找已有模式。
2. 写最小失败证据（测试/复现/清单），确认它能证明目标命题。
3. 写最小可通过代码，让验证变绿。
4. 跑 regression scope，确认没有破坏相关行为。
5. 清理本改动引入的死 import、临时输出。
6. 重读改过的区域，确认局部一致。

复杂变更前，先问：「有没有不新增抽象、不新增依赖的简单方案？」

### 3. 验证

```text
Typecheck: <command> => <result>
RED/Repro evidence: <command> => <failed as expected>
GREEN command: <command> => <passed>
Relevant tests: <command> => <result>
Full tests: <command or not run + reason> => <result>
Regression scope: <commands or not run> => <result>
Regression risk: <low/medium/high + why>
Exception: <none or justified gap>
```

- typecheck → RED → GREEN → 相关测试 → 全量测试
- 有任何新失败 → 修好再交接

### 4. 收尾

准备给 `dijiang-finish-work` 的数据：
- `git status --short --branch`
- `git diff --stat HEAD`
- 已验证的命令和结果
- 版本决策建议：`major` / `minor` / `patch` / `none`

参考 `references/git-workflow.md` 查看 Git 工作流详细指南。

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 在 main 上直接写代码 | 在 worktree 中开发 |
| 顺手改无关代码、引入新依赖 | 只做任务范围内的事 |
| 跑完测试有失败不管 | 修好新失败再交接 |
| 验证没跑却写"通过" | 写 `not run` + 原因 |
