---
name: dj-prd
description: >
summary: 将需求对齐结果转化为结构化 PRD 文档
phases: [align]
risk: low
  将需求对齐结果转化为结构化 PRD 文档。衔接 dj-grill 的输出，产出可供评审和后续拆分的形式化文档。
  Use after a dj-grill session, when alignment output needs to become a formal PRD document.
  触发词：写 PRD、PRD、产品文档、形式化需求、把对齐结果写下来。
---

# PRD: 需求对齐 → 产品文档

将 `dj-grill` 的对齐输出（需求摘要/对话记录）转化为结构化 PRD 文档。输出的 PRD 可直接供 `dj-split` 拆分为独立任务。

## 输入

- `dj-grill` 输出的需求摘要（目标、范围、方案、约束、验收标准）
- 或 grilling 对话记录中的关键结论

## 工作流

### 1. 读取对齐结果

确认我们有足够的信息来写 PRD：
- **目标** — 要解决什么问题？
- **用户** — 谁会使用这个功能？
- **方案** — 确认了哪个方向？
- **约束** — 技术/时间/兼容性限制？

如果关键信息缺失，不编造——留 `TBD` 标记并注明需要谁确认。

### 2. 编写 PRD

按以下模板输出 `.dijiang/prd/` 下的文档：

```markdown
# PRD: <功能名称>

## Problem Statement
<要解决的问题，从用户视角描述>

## Solution
<方案概述>  // 一句话说清楚做什么

## User Stories
- As a <角色>, I want <功能> so that <价值>

## Implementation Decisions
- <重要的技术选型和理由>
- <不需要写全部细节，只写关键取舍>

## Testing Decisions
- <测试策略：手动/自动、E2E/单元>

## Out of Scope
- <明确不包含的能力>

## Acceptance Criteria
- <可验证的验收条件，每条独立可测>

## Notes
<补充信息、已知风险、待确认项>
```

### 3. 对抗式自审

写完 PRD 后快速检查：
- 每个 User Story 是否独立可验证？
- Out of Scope 是否明确到不会产生误解？
- Acceptance Criteria 是否有人能实际执行验收？

### 4. 保存

- 保存到 `.dijiang/prd/<功能名>-<YYYYMMDD>.md`
- 给用户输出路径，建议下一步加载 `dj-split`

## 边界

- 不替用户做未确认的假设。不确定的地方写 `TBD`
- PRD 不是设计文档——不写实现细节、接口签名、数据结构
- 不写代码

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 编造缺失的需求信息 | 留 `TBD` 标记 |
| PRD 写成技术设计文档 | 聚焦用户问题和行为 |
| 直接从对话跳到实现 | 先出 PRD → 拆分 → 再实现 |
