---
name: dj-dispatch
description: "通用任务分类器：识别任务类型，路由到对应 skill 执行。 支持单一任务和混合任务（主类型驱动 + 串联执行）。 Use when the user gives a new task, request, feature idea, bug report, or any command that isn't already in a specific skill workflow. 触发词：新任务、帮忙、做一下、有个想法、有个需求。"
disable-model-invocation: true
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '08dc53f4-2f8f-4263-b49a-591088e5a569'
  PropagateID: '08dc53f4-2f8f-4263-b49a-591088e5a569'
  ReservedCode1: '67fa12b0-50a0-4c81-8151-fac03d2512b0'
  ReservedCode2: '67fa12b0-50a0-4c81-8151-fac03d2512b0'
---

参考规范：`docs/references/decision-ladder.md`、`docs/references/code-task-contract.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 用户请求被正确路由到对应的 dj-* skill |
| **Done when** | 请求被分类并路由到对应 skill，路由决策显式输出 |
| **Evidence** | 路由决策输出（目标 skill 名称 + 路由理由） |
| **Output** | 下一执行者的 skill 名称 + 简要上下文 |

# Dispatch: 任务分类器

识别用户请求的类型，路由到对应的 `dj-*` skill。

参考 Waza RESOLVER 的分阶段路由 + 歧义消解 + 串联模式。

## 路由表

完整路由表（Pre-build / Build / Post-build / Content / 运维 五段全量关键词）见 `references/routing-table.md`。核心路由：

- **动手前**：想法/细化/对齐 → `dj-grill`；分析/权衡 → `dj-reason`；设计/UI → `dj-design`；结构/模块划分 → `dj-codebase-design`；术语 → `dj-domain-modeling`；调研选型 → `dj-research`；写 PRD → `dj-output`
- **实现中**：写代码 → `dj-implement`/`dj-tdd`（护送走 `dj-regression-guard`）；复刻 → `dj-remix`；原型 → `dj-prototype`；脚本 → `dj-script`；拆任务 → `dj-split`；模式 → `dj-pattern`
- **交付前**：审查/看代码 → `dj-check`（PR/分支审查档）；验收/质量门禁 → `dj-check`；全量回归 → `dj-fullstack-testing`；回归保护 → `dj-regression-guard`；审计/债务/健康 → `dj-audit`；文档 → `dj-output`
- **内容**：润色/去 AI 味 → `dj-write`
- **运维**：交接 → `dj-handoff`；写 skill → `dj-meta`；出错 → `dj-hunt`；沉淀/自进化 → `dj-evolve`；有哪些流程 → `dj-ask`

## 混合任务处理

用户一句话里可能包含多个任务类型。规则：

1. **取主要类型** — 用户最关心的是"写代码"还是"检查质量"
2. **串联执行** — 先做前序任务（如先 `dj-grill` 对齐，再 `dj-implement` 实现）
3. **不太明确的请求** — 用 `dj-grill` 问清楚再路由

## 歧义消解

多个 skill 都可能匹配时，按以下规则消解：

| 规则 | 说明 |
|---|---|
| **最具体优先** | "设计登录页" → `dj-design`，不是 `dj-implement` |
| **报错 vs 审查** | 有具体错误现象 → `dj-hunt`；已交付要检查 → `dj-check` |
| **判断 vs 调试** | "判断一下" + 报错 → `dj-hunt`；"判断" + 值不值得 → `dj-reason` |
| **兜底** | 两个都模糊时读相关 SKILL.md 的 `边界` 段，用排除法；还模糊就问用户 |

## 串联模式

| 模式 | 流程 |
|---|---|
| **需求→实现** | `dj-grill` → 用户说"实现" → `dj-implement` → 用户说"检查" → `dj-check` |
| **分析→实现** | `dj-reason` 出方案 → 用户说"实现" → 按方案实施 |
| **排错→检查** | `dj-hunt` 修 bug → 用户说"发布" → `dj-check` 做发布前检查 |
| **极简→任意** | 对应 skill 遵守 workflow 的最小变更和验证约束 |

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 任务跨多类型且无主次 | 交 `dj-grill` 澄清 | 停下不做 |
| 请求含需身份确认的操作 | 显式提示用户确认 | 停下等确认 |
| 请求超出所有 dj-* 覆盖 | 停下说明，不硬套 | 建议澄清目标 |

## 边界

- 不处理需要用户身份确认的操作（发消息、删除、推送——这些要在技能中显式提示）
- 不主动执行只读检查之外的代码修改
- 不确定时默认路由到 `dj-grill` 对齐
- 低置信度时直接路由到 `dj-grill` 对话式澄清

参考规范：`docs/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 路由决策必须显式输出（目标 skill + 理由），不默认路由
2. 混合任务取主要类型，不尝试并排路由
3. 低置信度时必须路由到 `dj-grill`，不可猜
4. 不记录路由历史——每次请求独立路由
5. 路由到代码任务时，遵守 **Code Task TDD Contract**：输出必须包含 RED/Repro evidence、GREEN command、Regression scope 和 Exception 说明。
6. 路由断言前先读目标技能的 SKILL.md 验证当前内容——路由表会过时，SKILL.md 是事实源。新增、改名或行为变更的技能必须同步更新本文件路由表。
## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 低置信度时猜了一个 skill | 用户得到错误 skill 流程 | 不确定就 dj-grill |
| 同时路由到两个 skill | 上下文分裂，两个都做不好 | 取主要类型，串联执行 |
| 路由决策不输出理由 | 用户不知道为什么 call 了这个 skill | 输出目标 + 理由 |

参考规范：`docs/references/output-markers.md`（输出标记）。