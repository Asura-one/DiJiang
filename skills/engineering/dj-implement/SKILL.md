---
name: dj-implement
name_cn: 实现代码
description_cn: "按计划实现特性代码，遵守 git 安全工作流。用于实现特性、修复或基于计划/issue 的变更。"
description: "按计划实现特性代码，遵守 git 安全工作流。 触发词：实现、写代码、implement、开始做、按计划做、开发。"
---

深度细节：edit 冲突与 Python 补丁 → `references/edit-conflicts.md`；工具链陷阱（multiedit 部分落盘/CRLF 假 diff/uv 测试/stash 归因/go:embed/端口影子/PowerShell 判定）→ `references/tooling-pitfalls.md`；spec 拆分 → `references/ticket-decomposition.md`；Windows 主机 + WSL 混合工具链 → `references/windows-wsl-toolchain.md`；OpenAPI 契约先行与代码生成 → `references/openapi-codegen.md`。

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
- 确认在专用 worktree 并通知路径；**链式工单**（B 基于 A 分支）建法与基线陷阱 → `references/git-workflow.md`。
- 遵守 **Code Task TDD Contract**（`references/code-task-contract.md`）：非平凡改动先声明 scope；确认测试环境可用或记录原因。
- **上下文护栏**：任务产物设字节预算（单文件 32KB / 总 128KB）；超预算先读索引段，超限降级为索引行并说明，不硬截断。
- 任务仍 `planning` 时用原始请求重新走 dj-ask，不绕过 worktree check（见 dj-git-guardrails）。

### 1.5 决策阶梯门禁

走 **Ponytail 7-rung 决策阶梯**（`skills/engineering/dj-ponytail/references/decision-ladder.md`）。通过一层不进下一层；跳层须注释原因。

**shortcut 债务注释**：砍功能/接受限制时留 `shortcut: <上限>, <升级触发点>`；写不出触发点就不要简化。

### 2. 实现

**只改必要的代码**，每行可追溯到用户请求、PRD 或验证失败。顺序：读现有代码找已有模式 → 最小失败证据 → 最小可通过代码变绿 → regression scope → 清理死 import/临时输出 → 重读改过区域。

**改前/改后回归护送**：S0-S1（快速层+基线）确认后开始，S2-S3（专项+回归）由 `dj-regression-guard` 承接；本 skill 写专项（红测转绿），不重复实现基线与回归。

**配置值改动先查 git 状态**（`git check-ignore`）；example 模板项目的两层落地与两层冒烟 → `references/git-workflow.md`。

复杂变更前自问：「有没有不新增抽象、不新增依赖的简单方案？」

### 3. 验证

输出验证清单（Typecheck / RED·Repro / GREEN / Relevant·Full tests / UI 实测 / Regression scope·risk / Exception，各带命令与结果）：
typecheck → RED → GREEN → 相关 → 全量，有新失败修好再交接，没跑写 `not run`+原因；**UI/前端**用真实浏览器实测（改动先确认已在运行环境生效——容器/挂载热重载可能失效需重启）。
- **后端协议链路 / 异步攒批 / SFTP / 框架陷阱**的验证做法 → `references/verification-pitfalls.md`、`references/go-sftp-transfer.md`

### 4. 收尾

给 `dijiang-finish-work` 准备：git status/diff --stat、已验证命令与结果、版本决策建议。

**验收含 agent 无法执行的实测项** → 完成 agent 可验证部分即停，交付**观察 SOP**（worktree 保留待实测数据，禁纸面推演收尾）→ `references/verification-pitfalls.md`。

## Hard Rules（含反例）

1. 在 worktree 实现，不在 main 直接改（污染 main）
2. 每轮一个关注点（改太多 → 回滚困难）
3. 改码前查历史；不改未读过的代码
4. 完成后过 dj-check 门禁（不改就交 → 测试不过）
5. 只做任务范围内的事（顺手改无关 → reviewer 看不懂）
6. 验证没跑不写"通过"——`not run` + 原因
7. 连续编辑同一文件每条前重新 read；报 modified since last read → Python 补丁脚本（`references/edit-conflicts.md`）；文件被截断先 `git checkout --` 恢复
8. worktree 缺运行依赖 → 依赖借用主工作区（见 git-workflow.md）
