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

参考：`docs/references/decision-ladder.md`、`docs/references/code-task-contract.md`。深度细节：Git 工作流（链式工单基线/配置分层/依赖借用）→ `references/git-workflow.md`；验证陷阱与观察 SOP → `references/verification-pitfalls.md`；edit 冲突与 Python 补丁 → `references/edit-conflicts.md`；SFTP → `references/go-sftp-transfer.md`；spec 拆分 → `references/ticket-decomposition.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 按计划完成的特性代码 + 通过验证 |
| **Done when** | 代码实现完成 + 测试验证通过 + regression 确认 |
| **Evidence** | git diff、测试结果、regression 检查日志 |
| **Output** | 实现完成报告 + 代码变更 diff |

# Implement: 实现代码

按 PRD / 设计文档 / issue 描述实现。不提交、不 push、不 merge——归 `dijiang-finish-work`。

## Recursion Guard

你已身为 dj-implement。直接执行实现，不得再调用或派发另一个 dj-implement / dj-tdd；dj-check 由主会话交付时触发。

## 工作流

### 1. 准备

- 读取 PRD / 设计文档 / issue；核心行为不明确 → 停止报告需要需求对齐。
- **新功能先搜全项目现有管理面**（API 路由/前端页面/CLI 子命令）与参考项目同类实现，优先扩展现有流程不新建平行入口；做完再向用户提功能形态问题。
- 查历史教训（dj-memory recall，看相关 findings/learnings/patterns）。
- 确认在专用 worktree（`git worktree add ../<项目名>-<分支名> -b <type>/<任务名> <base>`）并通知路径。**链式工单**（B 基于 A 分支）：先提交 A 到任务分支再建 B 的 worktree——快照的是分支 HEAD 不是未提交改动（细节见 git-workflow.md）。
- 遵守 **Code Task TDD Contract**：非平凡改动先声明 scope；先写失败测试或复现清单；确认测试环境可用或记录原因。
- **上下文护栏**：任务产物设字节预算（单文件 32KB / 总 128KB）；超预算先读索引段，超限降级为索引行并说明，不硬截断。
- 任务仍 `planning` 时用原始请求重新走 dj-dispatch，不绕过 worktree check（见 dj-git-guardrails）。

### 1.5 决策阶梯门禁

走 **Ponytail 7-rung 决策阶梯**（`docs/references/decision-ladder.md`：YAGNI → 复用现有 → 标准库 → 原生特性 → 已有依赖 → 一行代码 → 最小代码）。通过一层不进下一层；跳层须注释原因。

**shortcut 债务注释**：砍功能/接受限制时留 `shortcut: <上限>, <升级触发点>`；写不出触发点就不要简化。

### 2. 实现

**只改必要的代码**，每行可追溯到用户请求、PRD 或验证失败。顺序：读现有代码找已有模式 → 最小失败证据 → 最小可通过代码变绿 → regression scope → 清理死 import/临时输出 → 重读改过区域。

**改前/改后回归护送**：S0-S1（快速层+基线）确认后开始，S2-S3（专项+回归）由 `dj-regression-guard` 承接；本 skill 写专项（红测转绿），不重复实现基线与回归。

**配置值改动先查 git 状态**（`git check-ignore`）：example 模板约定的项目需 tracked/untracked 两层落地（默认值+模板进 commit；本地实例改主工作区），冒烟两层都跑（见 git-workflow.md）。

复杂变更前自问：「有没有不新增抽象、不新增依赖的简单方案？」

### 3. 验证

输出验证清单（Typecheck / RED·Repro / GREEN / Relevant·Full tests / UI 实测 / Regression scope·risk / Exception，各带命令与结果）：
typecheck → RED → GREEN → 相关 → 全量，有新失败修好再交接，没跑写 `not run`+原因；**UI/前端**用真实浏览器实测（改动先确认已在运行环境生效——容器/挂载热重载可能失效需重启）。
- **后端协议链路** → mock 服务绑**固定端口**走真实客户端全链路；**异步攒批** → 「轮询+deadline」替代固定 sleep；**SFTP** → `net.Pipe()` 内存 server（`references/go-sftp-transfer.md`）；框架陷阱（StaticFiles/FastAPI 联合注解）→ `references/verification-pitfalls.md`

### 4. 收尾

给 `dijiang-finish-work` 准备：git status/diff --stat、已验证命令与结果、版本决策建议。

**验收含 agent 无法执行的实测项**（如 30 分钟观察窗口）→ 完成 agent 可验证部分即停，交付**观察 SOP**（配置生效方式/指标/回报格式/岔路选项，结构见 verification-pitfalls.md），worktree 保留待实测数据，禁纸面推演收尾。

## Hard Rules（含反例）

1. 在 worktree 实现，不在 main 直接改（污染 main）
2. 每轮一个关注点（改太多 → 回滚困难）
3. 改码前查历史；不改未读过的代码
4. 完成后过 dj-check 门禁（不改就交 → 测试不过）
5. 只做任务范围内的事（顺手改无关 → reviewer 看不懂）
6. 验证没跑不写"通过"——`not run` + 原因
7. 连续编辑同一文件每条前重新 read；报 modified since last read → Python 补丁脚本（`references/edit-conflicts.md`）；文件被截断先 `git checkout --` 恢复
8. worktree 缺运行依赖 → 依赖借用主工作区（见 git-workflow.md）