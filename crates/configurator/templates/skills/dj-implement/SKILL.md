---
name: dj-implement
description: >
  按计划实现特性代码，遵守 git 安全工作流。
  Use when the user wants to implement a feature, fix, or change based on a plan or issue.
  触发词：实现、写代码、implement、开始做、按计划做、开发。
---

# Implement: 实现代码

## 职责

按 PRD / 设计文档 / issue 描述实现代码。遵守深度模块设计原则和 git 安全工作流。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Confirmed requirement, PRD/design/issue, active task, target worktree, validation commands, and affected modules |
| 输出 | 已验证的代码 diff、验证证据、版本建议，以及给 `dijiang-finish-work` 的 handoff 数据 |
| 非目标 | 不在实现阶段 commit、push、merge、清理 worktree，也不做无关重构 |

## 工作流

### 1. 准备

```text
🔴 CHECKPOINT · 实现门禁
任务: <task name>
源产物: <PRD / design / issue / user request>
Worktree: <path and branch>
预期文件/模块: <scope>
Behavior/Invariant: <要保护或新增的行为命题>
RED/Repro evidence: <先失败的测试、复现命令、fixture、trace 或人工可复核步骤>
GREEN command: <实现后必须变绿的最小命令或检查>
Regression scope: <可能受影响的调用方、兄弟路径、全量/相关测试范围>
Exception: <none，或无法自动化/纯机械变更/环境不可用的具体原因和替代检查>
将提交: no
```

- 读取 PRD / 设计文档 / issue，理解要做什么。
- 确认需求已经足够实现；如果核心行为不明确，停止并报告“需要需求对齐”，不要在 `dj-implement` 内自行切换 skill。
- 确认在专用 worktree 中（git-safety）：
  ```bash
  git status --short --branch
  git worktree list
  # 如果当前目录是主 checkout，先创建隔离 worktree，再修改文件：
  git worktree add ../<项目名>-<任务名> -b <type>/<任务名> <base-branch>
  ```
- 主 checkout 必须保持纯净；所有代码修改只发生在任务 worktree。
- 确认测试环境可用，或记录不可用原因。
- 遵守 **Code Task TDD Contract**：所有代码、行为、配置、模板或脚本变更，先固定行为和回归边界，再实现。

### 2. 实现

**Surgical Changes**：只改必要的代码，不顺手重构。每一行改动都必须能追溯到用户请求、PRD 或验证失败。

Implementation order:

1. Locate existing patterns and APIs before editing.
2. Define the behavior/invariant and the smallest feedback loop before coding.
3. Produce RED/Repro evidence first: a failing test, bug reproduction, fixture, trace, or manual checklist with expected output. If this is impossible, record `Exception` before coding.
4. Implement the smallest behavior-complete change that turns the GREEN command green.
5. Run the regression scope that can prove related behavior still holds.
6. Remove unused imports, dead branches introduced by this change, and temporary debug output.
7. Re-read the touched area after edits to verify local consistency.
### 2.1 第一性原理实现检查

在实现复杂功能、修 bug、改架构边界或引入新抽象前，先打断类比式实现，回到事实推导：

```text
问题本质：这次改动真正要满足的用户/系统行为是什么？
硬事实：已有接口、数据模型、运行约束、失败信号中哪些不能改？
隐藏假设：当前方案依赖了哪些未经验证的推断？
推导方案：从硬事实出发，最小可行实现是什么？
更简单方案：有没有不新增抽象、不新增依赖的路径？
取舍：选择当前方案会留下什么维护成本？
```

如果第一性原理推导得出的方案与 PRD / design 冲突，停止并报告需要对齐的冲突点；后续路由由 workflow 或用户显式决定。

Rules:
### 3. 验证

Use this matrix:

```text
Typecheck: <command or not applicable> => <result>
RED/Repro evidence: <command/checklist => failed as expected, or Exception reason>
GREEN command: <command/checklist => passed>
Relevant tests: <command> => <result>
Full tests: <command or not run + reason> => <result>
Manual check: <input/action or n/a> => <result>
Regression scope: <commands/reference checks/sibling paths => result>
Regression risk: <low/medium/high + why>
Exception: <none or justified gap>
```

- 跑 typecheck。
- 跑 RED/Repro 对应的最小反馈回路，确认它在修复前能证明目标命题；若不能自动化，保留人工可复核证据。
- 跑 GREEN command，确认实现后目标命题通过。
- 跑相关测试文件和 regression scope。
- 最后跑全量测试，除非项目规模或环境不可用；不可用时记录原因。
- 有任何新失败 → 修好再交接。

### 4. 收尾交接

实现阶段只交付已验证 diff，不提交。提交、版本号、push、merge、worktree 清理由 `dijiang-finish-work` 统一处理，避免中途 commit 和任务边界不一致。

交接前必须准备：
- `git status --short --branch`
- `git diff --stat HEAD`
- 已执行的验证命令
- 版本决策建议：`major` / `minor` / `patch` / `none`

**Conventional Commits 规范**（finish-work 提交时必须遵守）：
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
- 不在主分支上直接 commit；不把文件名堆进 commit message，正文只写实际行为变化。
## 深度模块原则

实现时遵循以下设计原则：
- **深度模块**：大量行为放在小接口后面，一个模块 = 一个清晰的职责
- **接口即测试面**：通过公共接口测试，不测内部实现
- **最小接口**：暴露必要的最少信息
- **位置放对**：代码放在它该在的地方，不随意新建文件

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 测试跑不过（已有测试） | 判断是已有问题还是新引入 | 已有问题记录为 pre-existing；新问题必须修 |
| typecheck 报错 | 修类型错误 | 第三方库类型问题才允许最小 suppress，并写明原因 |
| worktree 创建失败 | 分支名加后缀或换路径 | `git worktree list` 检查已有，明确复用或新建 |
| PRD 描述不清无法实现 | 停止并报告需要需求对齐 | 用最保守实现前必须标注假设并获得确认 |
| 依赖冲突 | 检查已有依赖是否能满足 | 用标准库替代，或停止并报告需要收窄方案 |
| 测试环境不可用 | 检查依赖安装和配置 | 标注 `not run`，手动检查关键逻辑并说明风险 |
| worktree 中发现 base 有新 commit | 先 `git fetch` 判断影响 | 需要 rebase/merge 时先说明冲突面，再继续 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 在 main/master 上直接写代码 | 在 worktree 中开发 |
| 实现中途频繁 commit | 完成验证和版本决策后，由 finish-work 做一次范围一致的提交 |
| 引入新依赖不检查替代方案 | 先问 stdlib/已有依赖能不能做 |
| 跑完测试有失败不管 | 修好新失败；已有失败明确标注 |
| commit 消息写"fix bug" | 只给 finish-work 准备版本和提交建议，不在 implement 中提交 |
| 测试通过就不管文档了 | 检查文档或 spec 是否需要同步更新 |
| 改到无关模块顺便清理 | 只报告无关问题，不在本任务改 |
| 验证没跑却写成通过 | 写 `not run` 和原因 |
