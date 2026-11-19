# dj-ask: 单请求路由完整关键词表 + 歧义消解

SKILL.md 保留核心路由规则；不确定某句话路由到哪、或多个 skill 都匹配时读本表。

## 歧义消解

| 规则 | 说明 |
|---|---|
| 最具体优先 | "设计登录页" → `dj-design`，不是 `dj-implement` |
| 报错 vs 审查 | 有具体错误现象 → `dj-hunt`；已交付要检查 → `dj-check` |
| 判断 vs 调试 | "判断一下" + 报错 → `dj-hunt`；"判断" + 值不值得 → `dj-reason` |
| 兜底 | 都模糊时读相关 SKILL.md 边界段排除法；仍模糊问用户 |

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
| "issue 堆积/工单分诊/帮我处理 issue" | `dj-split`（分诊档：先分诊再拆分/归位） |
| "大块工作/绿地规划/路线图/决策地图" | `dj-wayfinder` |

## Build（实现中）

| 用户说的话 | 路由到 |
|---|---|
| "实现/写代码/做功能/加个接口" | `dj-implement` 或 `dj-tdd`（改码护送走 `dj-regression-guard`） |
| "复刻/仿站/对标/clone" | `dj-remix` |
| "原型/验证一下/探一下" | `dj-prototype` |
| "极简/少写/简单点" | 对应实现 skill，并应用 workflow 的最小变更约束 |
| "写个脚本/做个小工具" | `dj-script` |
| "拆分任务、切任务、分批做" | `dj-split` |
| "模式/重复/抽象/历史修复" | `dj-audit`（含 git 考古与重复信号） |
| "merge 冲突/rebase 冲突/解决冲突" | `dj-merge-conflict` |

## Post-build（交付前）

| 用户说的话 | 路由到 |
|---|---|
| "审查/帮我看看代码" | `dj-check`（PR/分支审查档） |
| "检查/验收/质量门禁" | `dj-check` |
| "全量测试/全面回归/系统测试" | `dj-fullstack-testing` |
| "做个验证技能/功能地图/验证可复用/后续回归直接跑" | `dj-verify`（生成项目本地验证资产） |
| "回归保护/改前基线/三明治" | `dj-regression-guard` |
| "审计/扫一下/过度工程" | `dj-audit` |
| "债务/技术债/标记/健康检查/配置检查" | `dj-audit`（选择 debt 或 health profile） |
| "代码规则/纪律/karpathy" | 对应实现或审查 skill，并应用 workflow 通用约束 |
| "文档/写文档/PRD/说明" | `dj-output` |

## Content（内容进出）

| 用户说的话 | 路由到 |
|---|---|
| "润色/改文字/去 AI 味" | `dj-write` |
| "教我/我想学/跨会话学习一个主题" | `dj-teach`（状态化教学工作区） |
| "读链接/抓网页/读 PDF/总结这个页面" | `dj-read` |
| "吸收/借鉴/参考外部项目的模式/设计/结构" | `dj-absorb`（有选择吸收外部项目，融合到自有项目） |

## 运维

| 用户说的话 | 路由到 |
|---|---|
| "交接/换 session" | `dj-handoff` |
| "loop/无人值守/自动化循环/定时任务工作流" | `dj-loop`（只设计评审不运行） |
| "无人值守/回头检查/你看着办/我走了你搞定/长期任务要留轨迹" | `dj-decision`（可审计决策轨迹） |
| "写 skill、创建技能、技能指南" | `dj-meta` |
| "出错了/不工作/报错" | `dj-hunt` |
| "又犯同样的错/别老犯这错/防复发/让这类错不再犯" | `dj-correct`（逐级强制防复发） |
| "总结经验/沉淀技能/自进化" | `dj-evolve` |
| "整理文档/知识治理/文档和代码对不上/收尾时同步文档/洁癖清理工作区" | `dj-gov`（知识治理收尾：文档/规则/记忆与代码对齐） |
| "记忆/上次怎么解的/之前讨论过吗/项目经验/找历史发现" | `dj-memory`（跨会话记忆检索与写入） |
| "子代理/多 agent/派任务/生成 agent/协作通道" | `dj-channel`（多 agent 协作通道） |
| "配置/CI secret/凭据/点三方仪表盘/一次性迁移/手动步骤" | `dj-wizard`（生成交互式 bash 向导，人类走步骤） |
| "消息没接住/刚才没听懂/换个说法再讲/纠正理解" | `dj-wait-what`（用 CONTEXT.md 词汇重讲） |
| "问卷/需要问别人/给别人填的表/采访别人" | `dj-questionnaire`（拷问发送对象，反向 grill） |

## 单请求路由细则（SKILL.md 外溢）

| 情形 | 处置 |
|------|------|
| 混合任务 | 取主要类型串联执行（先 `dj-grill` 对齐再 `dj-implement`），不并排路由两个 skill |
| 低置信度 | → `dj-grill` 澄清，不可猜 |
| re-entry | 任务仍 `planning` 时，用原始用户请求重新走本 skill 路由，不绕过 worktree check（见 `dj-git-guardrails`） |
| 回述与 grill 分层 | grill 是需求不清的逐轮拷问门禁；回述是清晰任务的 3 秒防跑偏，说了就走 |
| 关键词或歧义拿不准 | 读本表上方各节（全量关键词表 + 歧义消解细则） |
