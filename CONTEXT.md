---
name: CONTEXT.md
description: DiJiang 项目领域术语表
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '30b07129-de05-4c4d-b49f-ff569e1eba2b'
  PropagateID: '30b07129-de05-4c4d-b49f-ff569e1eba2b'
  ReservedCode1: '396f5820-f615-4fa7-9e74-166e78f33d4c'
  ReservedCode2: '396f5820-f615-4fa7-9e74-166e78f33d4c'
---

# DiJiang 领域语言

本文件定义 DiJiang 项目的核心术语。所有 skill 和文档使用这些术语保持一致。

## 术语

### Skill

可被 agent 加载的能力单元。每个 skill 是 `skills/<bucket>/<name>/SKILL.md` 文件。

- **User-invoked skill**：只能由用户显式调用（`disable-model-invocation: true`），负责编排和入口。
- **Model-invoked skill**：可由模型自动触发或用户调用，承载可复用的工程纪律。

User-invoked skill 可以调用 model-invoked skill，但永远不能调用另一个 user-invoked skill。

### Bucket

Skill 的分类桶。DiJiang 使用两个 bucket：
- **engineering**：代码工作相关的 skill（实现、测试、调试、审查、设计等）
- **productivity**：通用工作流工具（文字润色等）

### .dijiang/

项目本地状态目录（gitignored）。包含：
- `tasks/` — 任务存储（每个任务一个目录，含 task.json + prd.md + design.md）
- `memory/` — 持久记忆（JSONL 文件）
- `spec/` — 编码规范
- `config.toml` — 项目配置
- `active_task.txt` — 活跃任务指针

### Task

一个被追踪的工作单元。存储在 `.dijiang/tasks/<id>/` 中。

**状态机**：`planning → in_progress → completed → archived`，另有 `paused` 状态。

**task.json schema**（简化）：
```json
{
  "id": "unique-id",
  "title": "任务标题",
  "status": "planning|in_progress|completed|archived|paused",
  "createdAt": "ISO 8601",
  "completedAt": "ISO 8601 or null",
  "branch": "git 分支名",
  "notes": "备注",
  "meta": {}
}
```

### CONTEXT.md

项目领域术语表。定义项目使用的核心词汇及其关系，帮助 agent 理解项目语言。参考 `docs/references/` 中的领域建模参考。

### ADR

架构决策记录。存储在 `docs/adr/`。仅在决策难以逆转、缺少背景会令人意外、且确实比较过替代方案时创建。

### dj-setup

初始化 DiJiang 项目状态或从旧版迁移的 skill。每个项目运行一次。

### dj-memory

项目管理记忆的 skill（model-invoked）。其他 skill 通过 `Call the Skill tool with "dj-memory"` 调用。

### dj-dispatch

任务分流和技能路由的 skill。读取任务状态和用户意图，推荐合适的 dj-* skill。

### dj-grill

需求对齐的 skill。逐轮拷问式访谈，走遍决策树的每个分支。

### dj-check

交付质量闸门的 skill。代码审查、功能完整性、回归风险检查。作为流程内置的质量门，**只报告不改代码**。回归的改前/改后护送由 `dj-regression-guard` 承担。

### dj-hunt

bug 排查的 skill。根因定位、修复、反馈环验证。修复后的回归保护接入 `dj-regression-guard` 三明治协议。

### dj-regression-guard

改码三明治回归协议的 skill（model-invoked）。对每次改码任务执行 **改前基线 → 修改 → 专项验证 → 改后回归**。任务级轻量协议，作为所有改码入口（dj-implement/dj-tdd/dj-hunt）的前置纪律。

### dj-fullstack-testing

全栈回归测试引擎（model-invoked）。承担 **全量模式**：功能枚举、API 深度验证（契约/限流/幂等/落盘）、UI 交互、UX 评估、覆盖度核对。`dj-regression-guard` 引用其 diff 映射表、冒烟最小集与 `docs/project-understanding.md` 文档格式（引用不复制）。

### 回归三明治

改码三明治回归协议的形象名称。流程：改前基线（快速层）→ 改码（TDD 红绿循环）→ 专项验证（红测转绿）→ 改后回归（快速层增量对比）。区分两类失败：**专项失败 = 改错了**；**回归失败 = 改多了波及别处**。

### 快速层

任务改前/改后要双跑的测试子集，目标耗时 < 3 分钟。每个项目首次改码任务时定义并持久化到 `docs/project-understanding.md`（由 `dj-fullstack-testing` 维护）。

### 回归基线

任务改前的快速层结果快照（`.temp/regression-baseline-<ts>.json`），记录 commit-id、PASS/FAIL、耗时。任务级瞬时数据，只用于本次任务的增量比对，任务结束即删。

### 测试债台账

`docs/testing/test-debt.md`，记录零基建或补全不完整任务欠下的测试债（模块名 | 欠账 | 建议补全方式 | 日期）。非阻塞，但改码任务涉及欠账模块时顺带偿还。

### Worktree-First

Git 安全工作流原则：所有代码变更在独立 worktree 中进行，主工作区永远干净。