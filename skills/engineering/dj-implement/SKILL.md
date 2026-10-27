---
name: dj-implement
name_cn: 实现代码
description_cn: "按计划实现特性代码，遵守 git 安全工作流。用于实现特性、修复或基于计划/issue 的变更。"
description: "按计划实现特性代码，遵守 git 安全工作流。 Use when the user wants to implement a feature, fix, or change based on a plan or issue. 触发词：实现、写代码、implement、开始做、按计划做、开发。"
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '4d003b51-2252-45d7-99df-934b36445d6b'
  PropagateID: '4d003b51-2252-45d7-99df-934b36445d6b'
  ReservedCode1: 'bfbfe6a7-89ad-4e9e-80a6-57fdf93df580'
  ReservedCode2: 'bfbfe6a7-89ad-4e9e-80a6-57fdf93df580'
---

参考规范：`docs/references/decision-ladder.md`（编码前的决策阶梯）、`docs/references/code-task-contract.md`（代码任务合约）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 按计划完成的特性代码 + 通过验证 |
| **Done when** | 代码实现完成 + 测试验证通过 + regression 确认 |
| **Evidence** | git diff、测试结果、regression 检查日志 |
| **Output** | 实现完成报告 + 代码变更 diff |

# Implement: 实现代码

按 PRD / 设计文档 / issue 描述实现。不提交、不 push、不 merge——这些由 `dijiang-finish-work` 处理。

## Recursion Guard

你已身为 dj-implement。直接执行实现工作，不得再调用或派发另一个 dj-implement / dj-tdd。实现完成后的质量门禁（dj-check）由主会话在交付时触发，不在本 skill 内自行调用。


## 工作流

### 1. 准备

- 读取 PRD / 设计文档 / issue，理解要做什么。
- 核心行为不明确 → 停止，报告"需要需求对齐"。
- 查询项目历史教训：Call the Skill tool with "dj-memory" to recall，检查是否有相关的 findings/learnings/patterns。
- 确认在专用 worktree 中（主 checkout 只做同步）：
  ```bash
  git worktree add ../<项目名>-<分支名> -b <type>/<任务名> <base>
  ```
- 通知用户 worktree 路径（测试在 worktree 中执行）。
- 遵守 **Code Task TDD Contract**：先固定行为和回归边界再实现——非平凡改动先声明 scope（必改行为/允许模块/明确不处理的相邻问题）；先写失败测试（RED/Repro evidence）或复现清单；确认测试环境可用或记录原因。
- **上下文护栏**：读任务产物设字节预算（参考：单文件 32KB / 总注入 128KB）。超预算先读索引/结论段按需展开；总量超限降级超出文件为索引行（路径+原因+大小）并说明降级清单——不硬截断、不静默丢弃。
- 任务仍 `planning` 时用原始用户请求重新走 dj-dispatch，不绕过 worktree check（见 dj-git-guardrails）。

### 1.5 决策阶梯门禁

编码前走 **Ponytail 7-rung 决策阶梯**（`docs/references/decision-ladder.md`，逐层：YAGNI → 复用现有 → 标准库 → 原生特性 → 已有依赖 → 一行代码 → 最小代码）。通过一层就不进下一层；跳层须注释原因。

**shortcut 债务注释**：砍功能/临时绕过/接受已知限制时，必须留 `shortcut: <上限>, <升级触发点>`（例：`shortcut: 单机锁, 并发>100 时换分布式锁`）。写不出触发点就不要简化——没有触发点的简化将默默腐烂。

### 2. 实现

**只改必要的代码。** 每一行改动必须能追溯到用户请求、PRD 或验证失败。

顺序：读现有代码找已有模式 → 写最小失败证据（测试/复现/清单）确保证明目标命题 → 最小可通过代码变绿 → 跑 regression scope 确认未破坏相关行为 → 清理死 import 与临时输出 → 重读改过区域确认局部一致。

**改前/改后回归护送**：S0-S1（快速层+改前基线）确认后开始改码，S2-S3（专项+改后回归）由 `dj-regression-guard` 承接；本 skill 负责写专项（红测转绿），不重复实现基线与回归。

复杂变更前自问：「有没有不新增抽象、不新增依赖的简单方案？」

### 3. 验证

```text
Typecheck: <command> => <result>
RED/Repro evidence: <command> => <failed as expected>
GREEN command: <command> => <passed>
Relevant tests: <command> => <result>
UI 实测: <command or n/a + reason> => <result>
Full tests: <command or not run + reason> => <result>
Regression scope: <commands or not run> => <result>
Regression risk: <low/medium/high + why>
Exception: <none or justified gap>
```

- typecheck → RED → GREEN → 相关测试 → 全量测试；有任何新失败 → 修好再交接
- UI/前端改动 → 真实浏览器实测（Playwright 操作界面）；实测前确认改动已在运行环境生效（容器/挂载环境热重载可能失效需重启）——typecheck/build 通过只是必要条件，不等于功能可见

### 4. 收尾

给 `dijiang-finish-work` 准备数据：`git status --short --branch`、`git diff --stat HEAD`、已验证的命令和结果、版本决策建议（major/minor/patch/none）。Git 工作流详见 `references/git-workflow.md`。

## Hard Rules（含反例）

1. 必须在 worktree 中实现，不在 main 上直接改（污染 main）
2. 每轮改动只解决一个关注点（一次改太多 → 回滚困难）
3. 改代码前先查历史发现（dj-memory recall）
4. 实现完成后必须通过 dj-check 质量门禁（不改就交 → 测试不过）
5. 不改未读过的代码；只做任务范围内的事（顺手改无关 → reviewer 看不懂）
6. 验证没跑不得写"通过"——写 `not run` + 原因

当任务需要从 spec 拆分为可执行单元时，参考 `references/ticket-decomposition.md` 的垂直切片方法。

参考规范：`docs/references/anti-patterns.md`、`docs/references/durable-context-preflight.md`、`docs/references/output-markers.md`。