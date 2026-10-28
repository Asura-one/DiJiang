---
name: dj-split
description: "将 PRD 文档拆分为独立可执行的 task：衔接 dj-output 的输出，产出可路由的任务列表并同步 issue 跟踪器。 Use when a PRD is finalized and needs decomposition into independently executable tasks. 触发词：拆分任务、分任务、切成 task、work breakdown、分批做、任务拆解。"
disable-model-invocation: true
---

# Split: PRD → 可执行任务

将 PRD 拆分为独立可执行的任务——可独立实现、独立验证、独立交付。

Issue 跟踪器同步的完整机制、task 类 issue 模板、结题评论模板、关闭正规流程（三重证据+checklist 勾选）与平台陷阱 → `references/issue-sync.md`。

## 输入

- `.dijiang/prd/` 下的 PRD 文档，或用户提供的 PRD / issue / 功能描述

## 拆分原则

- **独立可发布** — 任务完成时系统仍可用（不破坏已有功能）
- **独立可验证** — 每个任务有自己的验收标准
- **合理颗粒度** — 不到一周一个任务，也不到一行代码一个任务
- **前后不依赖** — B 不因 A 未完成而无法开始（明确依赖的标 blocking）

## 工作流

### 1. 分析 PRD

提取：User Stories/功能点列表、依赖关系、数据模型变化（如需）、接口变化（如需）。

### 2. 生成任务列表

按优先级和依赖排序输出。每个任务格式：

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

列出推荐顺序（先做/可并行），如：Phase 1 基础能力（A、B 可并行）→ Phase 2 核心功能（C 依赖 A+B）→ Phase 3 增强（D、E 可并行，依赖 C）。

### 4. 输出

任务列表用于 dj-dispatch 路由；保存到 `.dijiang/tasks/` 或输出到对话中。

## 同步到 issue 跟踪器（可选）

若项目在本地任务外有真实跟踪器（Forgejo/Gitea/GitHub 等），每个任务**镜像**为一条 issue，本地任务为唯一权威源，不得只更新一侧：任务 → 创建 issue（目标/验收标准/依赖）并本地记录 issue 编号；状态变化/完成 → 补评论并关闭；依赖优先用跟踪器原生 blocking，否则正文标「Blocked by」。

**格式统一（必须）**：标题 `<前缀>-<编号> [task][P?] <一句话摘要>`，正文固定骨架 + 尾注本地任务路径，不得自创格式。**类型必标**：缺陷修复另挂 `bug`、需求实现另挂 `enhancement`；**标签必挂**（只用仓库已有标签，缺的先建，P 级约定色见 references）。**关闭正规流程**：三重证据核实（跟踪器状态+本地提交+代码实证，不信推测）→ 结题评论 → 同步勾选正文验收 checklist → PATCH closed → GET 复核 → 本地同步 completed。模板全文与 API/平台细节（假 404 陷阱、Windows curl 临时文件、token 同命令内使用）见 `references/issue-sync.md`。
**下一步**：工单就绪后，建议下一步逐张调用 `dj-implement`（每张工单从全新上下文启动；有 Forgejo 镜像时按 issue-sync 正规流程关闭）。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 依赖成环（A↔B） | 拆出共享前置任务 | 停下报告环 |
| PRD 有遗漏无法拆 | 报告遗漏，不改 PRD | 交 `dj-output` 补 |
| 跟踪器创建 issue 失败 | 按 `references/issue-sync.md` 排查 | 本地任务为准，标待同步 |

## 边界

- 不实现任务——那是 `dj-implement` / `dj-tdd` 的事
- 不估计工时——只做功能拆解
- 不修改 PRD——发现遗漏报告而不是改

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 任务颗粒度太大（一整天） | 拆分到半天可交付 |
| 任务互相依赖无法独立 | 重新划分边界使其独立 |
| 跳过 PRD 直接拆分 | 先确认需求文档化再拆 |
| 每个任务都给 dj-tdd | 简单的给 dj-implement，复杂行为给 dj-tdd |

**下一步**：工单就绪后，建议下一步调用 `dj-implement` 逐张执行（每张工单从全新上下文启动；有 Forgejo 镜像时按 issue-sync 正规流程关闭）。