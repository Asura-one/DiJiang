---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'abd3f07f-7d9e-41eb-85fc-5429ffc677b3'
  PropagateID: 'abd3f07f-7d9e-41eb-85fc-5429ffc677b3'
  ReservedCode1: 'dc11b5af-9902-4db5-9191-c58b88572f03'
  ReservedCode2: 'dc11b5af-9902-4db5-9191-c58b88572f03'
---

# dj-dispatch: 完整路由表

SKILL.md 保留每阶段核心行与判定规则；不确定某句话路由到哪时读本表全量匹配。

## Pre-build（动手前）

| 用户说的话 | 路由到 |
|---|---|
| "有个想法/想细化一下/想对齐" | `dj-grill` |
| "分析/推理/决策/取舍" | `dj-reason` |
| "设计/做页面/UI" | `dj-design` |
| "结构设计、放在哪、模块划分" | `dj-codebase-design` |
| "术语、统一语言" | `dj-domain-modeling` |
| "调研、研究、技术选型" | `dj-research` |
| "写 PRD、产品文档、把需求写下来" | `dj-output` |

## Build（实现中）

| 用户说的话 | 路由到 |
|---|---|
| "实现/写代码/做功能/加个接口" | `dj-implement` 或 `dj-tdd`（改码护送走 `dj-regression-guard`） |
| "复刻/仿站/对标/clone" | `dj-remix` |
| "原型/验证一下/探一下" | `dj-prototype` |
| "极简/少写/简单点" | 对应实现 skill，并应用 workflow 的最小变更约束 |
| "写个脚本/做个小工具" | `dj-script` |
| "拆分任务、切任务、分批做" | `dj-split` |
| "模式/重复/抽象" | `dj-pattern` |

## Post-build（交付前）

| 用户说的话 | 路由到 |
|---|---|
| "审查/帮我看看代码" | `dj-check`（PR/分支审查档） |
| "检查/验收/质量门禁" | `dj-check` |
| "全量测试/全面回归/系统测试" | `dj-fullstack-testing` |
| "回归保护/改前基线/三明治" | `dj-regression-guard` |
| "审计/扫一下/过度工程" | `dj-audit` |
| "债务/技术债/标记/健康检查/配置检查" | `dj-audit`（选择 debt 或 health profile） |
| "代码规则/纪律/karpathy" | 对应实现或审查 skill，并应用 workflow 通用约束 |
| "文档/写文档/PRD/说明" | `dj-output` |

## Content（内容进出）

| 用户说的话 | 路由到 |
|---|---|
| "润色/改文字/去 AI 味" | `dj-write` |

## 运维

| 用户说的话 | 路由到 |
|---|---|
| "交接/换 session" | `dj-handoff` |
| "写 skill、创建技能、技能指南" | `dj-meta` |
| "出错了/不工作/报错" | `dj-hunt` |
| "总结经验/沉淀技能/自进化" | `dj-evolve` |
| "有哪些流程/该走什么" | `dj-ask` |