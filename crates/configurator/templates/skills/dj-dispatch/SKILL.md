---
name: dj-dispatch
description: >
  通用任务分类器：识别任务类型，路由到对应 skill 执行。
  支持单一任务和混合任务（主类型驱动 + 串联执行）。
  Use when the user gives a new task, request, feature idea, bug report,
  or any command that isn't already in a specific skill workflow.
  触发词：新任务、帮忙、做一下、有个想法、有个需求。
---

# Dispatch: 任务分类器

识别用户请求的类型，路由到对应的 `dj-*` skill。

## 第一层：任务类型识别

| 用户说的话 | 类型 | 路由到 |
|---|---|---|
| "实现/写代码/做功能/加个接口" | 功能实现 | `dj-implement` 或 `dj-tdd` |
| "出错了/不工作/报错" | Bug | `dj-hunt` |
| "审查/帮我看看代码" | Code Review | `dj-review` |
| "检查/验收/质量门禁" | 质量检查 | `dj-check` |
| "有个想法/想细化一下/想对齐" | 需求对齐 | `dj-grill` |
| "原型/验证一下/探一下" | 原型验证 | `dj-prototype` |
| "写个脚本/做个小工具" | 脚本工具 | `dj-script` |
| "极简/少写/简单点" | 极简模式 | `dj-ponytail` + 对应的技能 |
| "审计/扫一下/过度工程" | 全仓审计 | `dj-audit` |
| "文档/写文档/PRD" | 文档输出 | `dj-output` |
| "写 PRD、产品文档、把需求写下来" | PRD 产出 | `dj-prd` |
| "拆分任务、切任务、分批做" | 任务拆分 | `dj-split` |
| "债务/技术债/标记" | 技术债 | `dj-debt` |
| "模式/重复/抽象" | 模式识别 | `dj-pattern` |
| "分析/推理/决策/取舍" | 推理增强 | `dj-reason` |
| "润色/改文字/去 AI 味" | 文字润色 | `dj-write` |
| "健康检查/配置检查" | 健康检查 | `dj-health` |
| "交接/换 session" | 交接 | `dj-handoff` |
| "设计/做页面/UI" | UI 设计 | `dj-design` |
| "规范/纪律/karpathy" | 编码准则 | `dj-karpathy` |
| "写 skill、创建技能、技能指南" | 技能写作 | `dj-meta` |
| "调研、研究、技术选型" | 调研 | `dj-research` |
| "结构设计、放在哪、模块划分" | 代码结构设计 | `dj-codebase-design` |
| "术语、统一语言" | 领域建模 | `dj-domain-modeling` |
| "git 安全、推前检查" | Git 护栏 | `dj-git-guardrails` |

参考配套文件：`references/AGENT-BRIEF.md`（路由行为概述）、`references/OUT-OF-SCOPE.md`（路由边界）、`references/routing-table.md`（完整路由表）。
## 第二层：代码任务分级

如果是实现/bug/重构任务，判断复杂度并执行 TDD Contract：

- RED/Repro evidence：先写失败测试或复现条件
- GREEN command：实现/修复后验证通过的命令
- Regression scope：确认相关行为不受影响
- Exception：无法自动化验证时记录原因

如果是实现/bug/重构任务，判断复杂度：

| 级别 | 特征 | 流程 |
|---|---|---|
| **简单** | 改 1-2 个文件，不涉及架构变化 | 直接路由到 implement/hunt，遵守 Code Task TDD Contract |
| **中等** | 涉及多个文件，但模式明确 | 先 `dj-grill` 对齐，再路由 |
| **复杂** | 涉及架构、新模块、跨层依赖 | 先 `dj-grill` 对齐，可追加 `dj-reason` |

## 混合任务处理

用户一句话里可能包含多个任务类型。规则：

1. **取主要类型** — 用户最关心的是"写代码"还是"检查质量"
2. **串联执行** — 先做前序任务（如先 `dj-grill` 对齐，再 `dj-implement` 实现）
3. **不太明确的请求** — 用 `dj-grill` 问清楚再路由

## 边界

- 不处理需要用户身份确认的操作（发消息、删除、推送——这些要在技能中显式提示）
- 不主动执行只读检查之外的代码修改
- 不确定时默认路由到 `dj-grill` 对齐
- 低置信度时（匹配不足时），推荐用户调用 `/dj-ask` 进行对话式路由选择
