---
name: CONTEXT.md
description: DiJiang 项目领域术语表
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'ff40879f-6d26-4db8-9de7-b7399118aff7'
  PropagateID: 'ff40879f-6d26-4db8-9de7-b7399118aff7'
  ReservedCode1: '06643d16-3d1a-43b2-8713-0d1fc32b9fe5'
  ReservedCode2: '06643d16-3d1a-43b2-8713-0d1fc32b9fe5'
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

质量门禁的 skill。代码审查、功能完整性、回归影响检查。

### dj-hunt

bug 排查的 skill。根因定位、修复、反馈环验证。

### Worktree-First

Git 安全工作流原则：所有代码变更在独立 worktree 中进行，主工作区永远干净。