---
name: dj-implement
description: >
  按计划实现特性代码，遵守 git 安全工作流。
  Use when the user wants to implement a feature, fix, or change based on a plan or issue.
  触发词：实现、写代码、implement、开始做、按计划做、开发。
dispatch_intent: >
  按计划实现特性代码：PRD/设计文档 → worktree → 实现 → 验证 → 收尾。
when_to_use: 实现、写代码、implement、开始做、按计划做、开发
---

参考规范：`.dijiang/references/decision-ladder.md`（编码前的决策阶梯）、`.dijiang/references/code-task-contract.md`（代码任务合约）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 按计划完成的特性代码 + 通过验证 |
| **Done when** | 代码实现完成 + 测试验证通过 + regression 确认 |
| **Evidence** | git diff、测试结果、regression 检查日志 |
| **Output** | 实现完成报告 + 代码变更 diff |

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

### 1.5 决策阶梯门禁

编码前先走 **Ponytail 7-rung 决策阶梯**（`.dijiang/references/decision-ladder.md`），逐层自检：

1. **YAGNI** — 这行代码真的需要吗？
2. **复用现有** — 项目里已经有现成的了吗？
3. **标准库** — 语言 stdlib 能解决吗？
4. **原生特性** — 语言本身的特性够用吗？
5. **已有依赖** — 已有依赖提供需要的能力吗？
6. **一行代码** — 一行能搞定吗？
7. **最小代码** — 写最少能工作的代码

通过一层就不进下一层。跳过任意层需要注释说明原因。
如果需求明确适合极简路径，可叠加 `dj-ponytail` 加强纪律。

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

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）、`.dijiang/references/durable-context-preflight.md`（记忆预检）。

## Hard Rules

1. 必须在 worktree 中实现，不在 main 上直接改
2. 每轮改动只能解决一个关注点
3. 改代码前先查历史发现（`dijiang mem recall`）
4. 实现完成后必须通过 dj-check 质量门禁
5. 不改未读过的代码

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 不在 worktree 中改 | main 被污染 | 先 create worktree |
| 一次改太多文件 | 回滚困难和冲突 | 一次一个关注点 |
| 不改就交 | 测试不过、有警告 | 实现后跑 dj-check |
| 顺手改不相关代码 | reviewer 看不懂变动 | 只做任务指定的事 |

当任务需要从 spec 拆分为可执行单元时，参考 `references/ticket-decomposition.md` 的垂直切片方法。

参考规范：`.dijiang/references/output-markers.md`（输出标记）。
