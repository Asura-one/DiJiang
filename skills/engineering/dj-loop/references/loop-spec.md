# LOOP-SPEC 模板（Fitness 通过后复制填写）

> 摘要：无人值守 loop 的设计规范模板，dj-loop Design 阶段产出物；agent-risk-review.md 负责对本文件的权限/工具/升级/审计面做风险复核，两者不要重填。

```markdown
# LOOP-SPEC: <loop-name>

## Name & Status
- Loop name / Owner / Status（draft|active|paused|retired）

## Problem & Goal
- 处理什么重复任务？为什么是 loop 而不是一次性任务？
- 主目标 / Done means（可验证陈述）/ 明确不许优化的方向

## Task Fit（Fitness 结论回填）
- 周频以上重复？有客观验证？环境可复现？有 review 路径？——任一 no 写明为什么仍是 loop

## Trigger
- 类型（schedule|event|condition）/ 频率上限 / 防重入策略（上轮未完成时新触发怎么办）

## Inputs & Context
- 必需输入 / 稳定上下文源 / 运行态文件位置（loop-state.md 模板）/ 不可信外部输入（标注消毒方式）

## Skill / Memory / State
- 依赖哪些 skill / 相关长期记忆 / state 与 trace 的位置分工（three-asset-boundaries.md）

## Tools & Permissions
- 文件 / Git / CI / 告警各访问面 / 本 loop 最高风险工具 / 显式禁区（删除、force push、对外发布永不自动化）

## Maker-Checker & Objective Gates
- maker / checker 是否结构分离 / 主 gate（强验证）/ 次门 / 最终裁定权（evidence-gate.md）

## Budget & Stop
- 单轮 token/时间上限 / 重试上限 / 连续失败 N 轮熔断点

## Escalation
- 何时升级 / 升级给谁 / 哪些动作永远人批 / steering 与 follow-up 两类干预点怎么进

## Review Capacity
- 预期产出量 / 评审人 / 积压超限怎么办（丢、降频、还是停）

## Retirement
- 退役条件（连续 N 轮零 accepted change / 任务消失 / 成本超收益）/ 清理动作 / 升格进 skill/memory 的经验

## Approval
- 技术负责人 / 风险评审人 / 复验日期
```

来源：琅嬛 templates/LOOP-SPEC.md 精简改写（去掉多阶段 HARNESS 与 Shared Memory 节，并入 DiJiang 三槽位引用）
