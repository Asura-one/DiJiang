---
name: dj-dispatch
description: >
  通用任务分类器：识别任务类型，路由到对应 skill 执行。
  支持单一任务和混合任务（主类型驱动 + 串联执行）。
  Use when the user gives a new task, request, feature idea, bug report,
  or any command that isn't already in a specific skill workflow.
  触发词：新任务、帮忙、做一下、有个想法、有个需求。
---

参考规范：`.dijiang/references/decision-ladder.md`、`.dijiang/references/code-task-contract.md`。

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

### Pre-build（动手前）

| 用户说的话 | 路由到 |
|---|---|
| "有个想法/想细化一下/想对齐" | `dj-grill` |
| "分析/推理/决策/取舍" | `dj-reason` |
| "设计/做页面/UI" | `dj-design` |
| "结构设计、放在哪、模块划分" | `dj-codebase-design` |
| "术语、统一语言" | `dj-domain-modeling` |
| "调研、研究、技术选型" | `dj-research` |
| "写 PRD、产品文档、把需求写下来" | `dj-output` |

### Build（实现中）

| 用户说的话 | 路由到 |
|---|---|
| "实现/写代码/做功能/加个接口" | `dj-implement` 或 `dj-tdd` |
| "复刻/仿站/对标/clone" | `dj-remix` |
| "原型/验证一下/探一下" | `dj-prototype` |
| "极简/少写/简单点" | `dj-ponytail` + 对应的技能 |
| "写个脚本/做个小工具" | `dj-script` |
| "拆分任务、切任务、分批做" | `dj-split` |
| "模式/重复/抽象" | `dj-pattern` |

### Post-build（交付前）

| 用户说的话 | 路由到 |
|---|---|
| "审查/帮我看看代码" | `dj-review` |
| "检查/验收/质量门禁" | `dj-check` |
| "审计/扫一下/过度工程" | `dj-audit` |
| "债务/技术债/标记" | `dj-debt` |
| "健康检查/配置检查" | `dj-health` |
| "代码规则/纪律/karpathy" | `dj-karpathy` |
| "文档/写文档/PRD/说明" | `dj-output` |

### Content（内容进出）

| 用户说的话 | 路由到 |
|---|---|
| "润色/改文字/去 AI 味" | `dj-write` |

### 运维

| 用户说的话 | 路由到 |
|---|---|
| "交接/换 session" | `dj-handoff` |
| "写 skill、创建技能、技能指南" | `dj-meta` |
| "出错了/不工作/报错" | `dj-hunt` |

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
| **极简→任意** | `dj-ponytail` 裁剪原则 + 对应技能 |

## 边界

- 不处理需要用户身份确认的操作（发消息、删除、推送——这些要在技能中显式提示）
- 不主动执行只读检查之外的代码修改
- 不确定时默认路由到 `dj-grill` 对齐
- 低置信度时（匹配不足时），推荐用户调用 `/dj-ask` 进行对话式路由选择

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 路由决策必须显式输出（目标 skill + 理由），不默认路由
2. 混合任务取主要类型，不尝试并排路由
3. 低置信度时必须路由到 `dj-grill`，不可猜
4. 不记录路由历史——每次请求独立路由

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 低置信度时猜了一个 skill | 用户得到错误 skill 流程 | 不确定就 dj-grill |
| 同时路由到两个 skill | 上下文分裂，两个都做不好 | 取主要类型，串联执行 |
| 路由决策不输出理由 | 用户不知道为什么 call 了这个 skill | 输出目标 + 理由 |

参考规范：`.dijiang/references/output-markers.md`（输出标记）。
