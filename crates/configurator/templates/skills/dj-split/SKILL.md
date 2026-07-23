---
name: dj-split
description: >
  将 PRD 文档拆分为独立可执行的 task。衔接 dj-output 的输出，产出可直接用 dijiaang task start 的任务列表。
  Use after a PRD is written, when you need to break the work into independently implementable tasks.
  触发词：拆分任务、分任务、切成 task、work breakdown、分批做。
summary: 将 PRD 文档拆分为独立可执行的 task
phases: [align]
risk: low
  将 PRD 文档拆分为独立可执行的 task。衔接 dj-output 的输出，产出可直接用 dijiaang task start 的任务列表。
  Use after a PRD is written, when you need to break the work into independently implementable tasks.
  触发词：拆分任务、分任务、切成 task、work breakdown、分批做。
---

# Split: PRD → 可执行任务

将 PRD 拆分为独立可执行的任务。每个任务应可独立实现、独立验证、独立交付。

## 输入

- `.dijiang/prd/` 下的 PRD 文档
- 或用户提供的 PRD / issue / 功能描述

## 拆分原则

- **独立可发布** — 每个任务完成时系统仍然可用（不破坏已有功能）
- **独立可验证** — 每个任务有自己的验收标准
- **合理的颗粒度** — 不粗到一个任务做一周，不细到一个任务只改一行
- **前后不依赖** — B 不因 A 未完成而无法开始（除非明确有依赖的，标记为 blocking）

## 工作流

### 1. 分析 PRD

从 PRD 中提取：
- User Stories 或功能点列表
- 各功能的依赖关系
- 数据模型变化（如果需要）
- 接口变化（如果需要）

### 2. 生成任务列表

按优先级和依赖排序，输出任务列表。每个任务格式：

```markdown
## Task: <任务名>

**目标**：一句话说清楚这个任务做什么

**PRD 引用**：<PRD 中的对应 User Story 或章节>

**验收标准**：
- [ ] <可验证的条件 1>
- [ ] <可验证的条件 2>

**建议 skill**：dj-implement / dj-tdd

**依赖**：<无 / 依赖 Task X>
```

### 3. 建议执行顺序

列出推荐的开发顺序（哪些先做、哪些可以并行）：

```text
Phase 1（基础能力）：Task A, Task B（可并行）
Phase 2（核心功能）：Task C（依赖 A+B）
Phase 3（增强功能）：Task D, Task E（可并行，依赖 C）
```

### 4. 输出

- 任务列表可直接作为 `dijiang task start <name>` 的输入
- 保存到 `.dijiang/tasks/` 或输出到对话中

## 边界

- 不实现任务——那是 `dj-implement` / `dj-tdd` 的事
- 不估计工时——只做功能拆解
- 不修改 PRD——如果发现 PRD 有遗漏，报告而不是改

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 任务颗粒度太大（一整天） | 拆分到半天可交付 |
| 任务互相依赖无法独立 | 重新划分边界使其独立 |
| 跳过 PRD 直接拆分 | 先确认需求文档化再拆 |
| 每个任务都给 dj-tdd | 简单的给 dj-implement，复杂行为给 dj-tdd |
