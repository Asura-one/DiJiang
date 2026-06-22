---
name: implement
description: >
  按计划实现特性代码，遵守 git 安全工作流。
  Use when the user wants to implement a feature, fix, or change based on a plan or issue.
  触发词：实现、写代码、implement、开始做、按计划做、开发。
---

# Implement: 实现代码

## 职责

按 PRD / 设计文档 / issue 描述实现代码。遵守深度模块设计原则和 git 安全工作流。

## 工作流

### 1. 准备

```
🔴 CHECKPOINT · 开始前确认
```

- 读取 PRD / 设计文档 / issue，理解要做什么
- 确认在 worktree 中（git-safety）：
  ```bash
  pwd  # 应在 ../<项目名>-<分支名> 中，而非主目录
  git branch --show-current  # 应在 feat/* 分支上
  # 如果在主目录 → git worktree add ../<项目名>-<分支名> -b feat/<任务简述>
  ```
- 确认测试环境可用（typecheck、test runner）

### 2. 实现

- **用 TDD**（如果项目有测试框架）：先写失败测试，再实现，再重构
- **不用 TDD**（脚本、原型、无测试框架）：直接实现，但每个逻辑单元一个 commit
- 遵循项目已有的代码规范（读 CLAUDE.md / AGENTS.md / .cursorrules）
- 引入新依赖前先问：stdlib 能做吗？已有依赖能做吗？（ponytail 精神）

### 3. 验证

- 跑 typecheck
- 跑相关测试文件
- 跑全量测试（最后）
- 有任何失败 → 修好再提交

### 4. 提交

```bash
# 确认不在主分支
git branch --show-current

# 提交
git add <具体文件>
git commit -m "feat: <摘要>"
```

- 每个逻辑单元一次 commit
- **Conventional Commits 规范**（必须遵守）：
  ```
  <type>(<scope>): <subject>
  
  <body>
  
  <BREAKING CHANGE>
  ```
  - type: `feat`/`fix`/`refactor`/`docs`/`test`/`chore`/`perf`/`ci`/`build`/`style`
  - scope: 可选，表示影响范围
  - subject: 简短描述，不超过 50 字符
  - body: 可选，详细描述
  - BREAKING CHANGE: 可选，不兼容变更说明
- 不在主分支上直接 commit
## 深度模块原则

实现时遵循以下设计原则：
- **深度模块**：大量行为放在小接口后面，一个模块 = 一个清晰的职责
- **接口即测试面**：通过公共接口测试，不测内部实现
- **最小接口**：暴露必要的最少信息
- **位置放对**：代码放在它该在的地方，不随意新建文件

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 测试跑不过（已有测试） | 检查是已有问题还是新引入 | 已有问题不阻塞，新问题必须修 |
| typecheck 报错 | 修类型错误 | 如果是第三方库类型问题，加 `@ts-ignore` + 注释 |
|| worktree 创建失败 | 分支名加后缀或换路径 | `git worktree list` 检查已有，询问是否复用 |
| PRD 描述不清无法实现 | 回到 grill 补充对齐 | 用最保守的实现，标注假设 |
| 依赖冲突 | 检查已有依赖是否能满足 | 用标准库替代，加 ponytail 标记 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
|| 在 main/master 上直接写代码 | 在 worktree 中开发 |
| 一个 commit 搞定所有改动 | 每个逻辑单元一个 commit |
| 引入新依赖不检查替代方案 | 先问 stdlib/已有依赖能不能做 |
| 跑完测试有失败不管 | 修好再提交 |
| commit 消息写"fix bug" | 用结构化格式 feat/fix/refactor |
| 测试通过就不管文档了 | 检查文档是否需要同步更新 |
