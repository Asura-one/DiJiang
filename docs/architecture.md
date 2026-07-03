# DiJiang 架构

## 概述

DiJiang 是一个 Rust 原生的 AI 编码助手工作流层。它提供 CLI 二进制 `dijiang`、一组可组合的 skill（`dj-*`），以及面向 Pi 平台的扩展集成，用于管理项目任务生命周期、记忆持久化、模板配置和 agent channel 编排。

项目结构为 Cargo workspace，包含 4 个 crate，各司其职。

## Crate 关系图

```
dijiang（workspace 根目录）
├── crates/cli/           # 入口：CLI 二进制、dispatch、finish-work
├── crates/task/          # 任务状态模型、路由/版本/能力门禁
├── crates/mem/           # 跨平台记忆持久化
└── crates/configurator/  # Init、模板、平台配置、更新
```

### 依赖流向

```
cli ──→ task ──→ （独立，不依赖 DiJiang 其他 crate）
  │
  ├──→ mem ──→ （独立，仅有平台适配器）
  │
  └──→ configurator ──→ （独立，仅有模板注册表）
```

三个库 crate（`task`、`mem`、`configurator`）互相独立，不共享 DiJiang 内部依赖。`cli` 是唯一依赖全部三个 crate 的集成层。

## Crate 职责

### `cli`（二进制：`dijiang`）

单一 `main.rs`（约 4000 行）包含所有 CLI 命令处理器和 dispatch 引擎。子命令使用 `clap` 组织：

| 命令组 | 子命令 |
|--------|--------|
| `status` | `status`、`status --compat` |
| `start` | `start <name>` |
| `dispatch` | `dispatch <prompt>` |
| `task` | `task list/current/status/archive/prune` |
| `finish-work` | `finish-work --verification --docs-sync --version-impact [--commit] [--push]` |
| `mem` | `mem list/sync/findings/learn/correction/archive/tactic/tactics/record/pattern/patterns/stats/backup/evolve/finetune` |
| `channel` | `channel spawn/list/send/status/stop/execute/execute-all` |
| `template` | `template list/pull/validate` |
| `skills` | `skills [--sync]` |
| `workflow-state` | `workflow-state [--json]` |
| `skill-body` | `skill-body <name>` |
| `doc-sync` | `doc-sync check [--base]` |
| `spec-sync` | `spec-sync check/record` |
| `init` | `init <name> [--force] [--platforms]` |
| `migrate` | `migrate` |
| `update` | `update [--force] [--from-github]` |

`cli` 中的关键架构组件：
- **Dispatch 引擎**（`dispatch_route`、`apply_route_gate`）：将自然语言提示分类为 skill 路由，执行工况状态门禁（Route Gate Phase 1）。
- **Git Gate** 集成（`ensure_task_worktree`、`evaluate_worktree_readiness`）：对代码修改类任务执行 worktree 隔离。
- **Finish Work**（`cmd_finish_work`）：验证、doc-sync、版本号递增、commit、本地集成、push、worktree 清理 —— 破坏性操作使用批准门禁（Phase 4）。
- **Channel 执行**（`cmd_channel_execute`）：为并行/隔离工作派生子 agent 进程。

### `task`（库：`dijiang-task`）

核心任务生命周期和工况约束。无外部依赖（仅 `serde`、`serde_json`、`thiserror`、`chrono`）。

| 模块 | 职责 |
|------|------|
| `types` | `TaskRecord`、`TaskStatus` — 状态模型，保持 Trellis 向后兼容 |
| `store` | JSON 任务持久化至 `./.dijiang/tasks/<id>/task.json` |
| `route_gate` | 工况 capsule 检测和路由约束（`evaluate_route`） |
| `git_gate` | Worktree 就绪评估（`evaluate_worktree_readiness`） |
| `capability_gate` | 高风险操作批准（integrate/push/cleanup） |
| `skill_manifest` | Skill 注册表、body 缓存、渐进式 skill 注入的懒加载 |
| `workflow_state` | Session 日志记录、对等窗口面、近期记忆注入 |
| `doc_sync` | `analyzer` + `mapper` — 从 git diff 检测哪些长期文档需要更新 |
| `spec_sync` | SHA256 的 spec 文件 checksum 追踪（`check` / `record`） |

### `mem`（库：`dijiang-mem`）

跨平台记忆持久化，使用平台特定适配器。

| 模块 | 职责 |
|------|------|
| `types` | 记忆模型（findings、lessons、tactics、patterns） |
| `store` | JSONL 本地持久化至 `~/.dijiang/mem/` |
| `memory` | 五层记忆架构（working → episodic → semantic → procedural → meta） |
| `adapter` | 抽象 `PlatformMemory` trait |
| `pi` / `claude` / `codex` / `opencode` / `hermes` | 平台特定记忆适配器 |
| `jsonl` | JSONL 文件 I/O 工具 |
| `registry` | 平台记忆发现和路由 |

### `configurator`（库：`dijiang-configurator`）

项目初始化、模板管理、平台配置、自更新。

| 模块 | 职责 |
|------|------|
| `init` | 项目脚手架：`.dijiang/`、`.pi/`、模板生成 |
| `types` | `PlatformKind` 枚举、配置模型 |
| `registry` | 平台插件注册 |
| `template_registry` | 从内嵌/远程源加载模板 |
| `templates` | init 使用的内置模板内容 |
| `dj_skills` | init 时生成 `dj-*` skill 文件 |
| `pi` / `claude` / `codex` / `cursor` / `opencode` / `hermes` | 平台特定配置生成 |
| `update` | 自更新机制（hash 比较 + GitHub 下载） |
| `changelog` | CLI 中显示变更日志 |

## 数据流

### 任务生命周期

```
用户提示 → dispatch_route()
  → dispatch_route_for_active_task() | dispatch_route()（分类器）
    → apply_route_gate()（执行工况约束）
      → evaluate_worktree_readiness()（Git Gate，若为代码路由）
        → ensure_task_worktree()（按需供应 worktree）
          → 路由决策 + 上下文注入 agent 提示
```

### Finish Work 流程

```
cmd_finish_work()
  → ensure_finish_preconditions()（验证脏树、任务状态）
    → update_workspace_version()（按需语义版本递增）
      → perform_finish_commit()（可选 --commit）
        → perform_finish_integration()（可选 --integrate，带批准门）
          → auto_cleanup_worktree()（移除 worktree，删除分支）
            → append_session_closure()（日志 + 归档任务）
```

### 记忆流程

```
cmd_mem_findings() → current_project_memory()
  → dijiang_mem::ProjectMemory::append_finding()
    → store::append_jsonl()（本地持久化）
      → adapter sync（平台特定推送）

cmd_mem_backup() → 项目记忆 → ~/.dijiang/mem/（全局存储）
```

### Doc-Sync 流程

```
cmd_doc_sync_check()
  → doc_sync::analyzer::analyze_diff()（git diff → 变更事件）
    → doc_sync::mapper::map_events_to_docs()（变更事件 → 受影响文档）
      → 输出：文档路径 + 置信度 + 触发证据
```

### Spec-Sync 流程

```
cmd_spec_spec_check()
  → spec_sync::check()（SHA256 比较）
    → 输出：已变更的 spec 文件列表

cmd_spec_sync_record()
  → spec_sync::record()（更新 checksum 数据库）
```

## Skill 系统

DiJiang 有两层 skill：`dj-*` skill（原子工作能力）和 `dijiang-*` skill（session 包装器）。

### 文件结构

```
.pi/
├── skills/                  # dj-* skill SKILL.md 文件集合
│   ├── dj-dispatch/SKILL.md
│   ├── dj-grill/SKILL.md
│   ├── dj-implement/SKILL.md
│   ├── dj-check/SKILL.md
│   ├── dj-hunt/SKILL.md
│   ├── dj-output/SKILL.md
│   ├── dj-ponytail/SKILL.md
│   ├── dj-tdd/SKILL.md
│   ├── dj-script/SKILL.md
│   ├── dj-design/SKILL.md
│   ├── dj-prototype/SKILL.md
│   ├── dj-audit/SKILL.md
│   ├── dj-debt/SKILL.md
│   ├── dj-health/SKILL.md
│   ├── dj-pattern/SKILL.md
│   ├── dj-karpathy/SKILL.md
│   ├── dj-review/SKILL.md
│   ├── dj-write/SKILL.md
│   ├── dj-handoff/SKILL.md
│   ├── dijiang-start/SKILL.md      # session 包装器
│   ├── dijiang-continue/SKILL.md    # session 包装器
│   └── dijiang-finish-work/SKILL.md # session 包装器
├── agents/                   # 子 agent 定义
│   ├── dijiang-check.md
│   ├── dijiang-implement.md
│   └── dijiang-research.md
├── extensions/dijiang/        # Pi 扩展
│   └── index.ts
├── prompts/                   # Pi prompt 模板
│   ├── dijiang-start.md
│   └── dijiang-finish-work.md
└── settings.json              # Pi 配置：注册 skills、extensions、prompts
```

### Skill 加载机制

1. **注册**：`.pi/settings.json` 的 `"skills": ["./skills"]` 配置告诉 Pi 引擎从此目录加载 skill。
2. **清单**：`dijiang skills` 列出所有可用 `dj-*` skill；`dijiang workflow-state --json` 将当前 capsule 对应的 skill 清单注入 agent 提示。
3. **懒加载**：默认只注入 skill 清单（名称 + 描述 + 风险等级）。完整 SKILL.md body 在路由引擎选定目标 skill 后才按需加载，通过 `dijiang skill-body <name>` 获取。
4. **同步**：`dijiang skills --sync` 将内置 skill 模板同步到 `.pi/skills/`。

### Skill 的路由目标

每个 skill 在 `task/src/skill_manifest.rs` 中注册 manifest 元数据，包括 `name`、`description`、`risk`（低/中/高）、`capsule`（适用工况）。Route Gate 在 dispatch 时根据 active task status 和 capsule 匹配，确定允许、重定向还是阻断。

## Workflow 路由系统

### 规范工作流

```
none
  └─ dispatch: dijiang start <name> 或 dj-dispatch
planning
  └─ align: dj-grill，必要时 dj-output
in_progress
  ├─ implement: dj-implement / dj-tdd / dj-hunt / dj-script / dj-design
  └─ check: dj-check
completed
  └─ finish: dijiang finish-work --verification ... --docs-sync ... --version-impact ...
archived
  └─ closed: 只读；如需继续则重新 dijiang start <task>
paused
  └─ resume: dijiang-continue → 回到 planning 或 in_progress
```

### Route Gate（运行时路由门禁）

Route Gate 将上述工作流规则从纯提示级别提升为运行时硬约束。在 `crates/task/src/route_gate.rs` 中实现：

```
active_task.route_decision(request_intent) →
  allow:    路由正常，进入目标 skill
  redirect: 路由重定向到 dj-grill / dijiang-continue
  block:    阻断并提示重新 start
```

具体规则：

| 任务状态 | 实现类请求 | 文档类请求 | 其他 |
|---------|-----------|-----------|------|
| `planning` | redirect → dj-grill | allow → dj-output | allow |
| `in_progress` | allow | allow | allow |
| `paused` | redirect → dijiang-continue | redirect → dijiang-continue | redirect → dijiang-continue |
| `completed` | block → 需重新 start | block → 需重新 start | block → 需重新 start |
| `archived` | block → 需重新 start | block → 需重新 start | block → 需重新 start |

新任务（无 active task）不套用 Route Gate，保留 dispatch 分类器原始行为。

### 子 Agent 系统

DiJiang 定义三个 Pi 子 agent，每个消费 `dijiang workflow-state --json` 和 `<dijiang-target-skill ...>` 上下文作为路由入口：

| 子 agent | 文件 | 职责 |
|---------|------|------|
| dijiang-check | `.pi/agents/dijiang-check.md` | 质量审查、审计、技术债、健康报告 |
| dijiang-implement | `.pi/agents/dijiang-implement.md` | 特性实现、TDD、原型、脚本 |
| dijiang-research | `.pi/agents/dijiang-research.md` | 技术调研、bug 排查、分类 |

子 agent 加载时先读 `workflow-state` 获取运行时路由上下文，再根据 `<dijiang-target-skill>` 决定使用哪个 `dj-*` skill。skill 清单由 workflow-state 的 `Skill Manifests` 注入。

## Pi Extension（平台集成层）

DiJiang 在 Pi 中通过 `.pi/extensions/dijiang/index.ts` 扩展实现运行时集成。该扩展是 Pi 与 DiJiang 之间的桥梁，在 Pi 的 agent 生命周期事件中自动注入 DiJiang 路由上下文。

### 注册方式

`.pi/settings.json`：
```json
{
  "enable_skill_commands": true,
  "extensions": ["./extensions/dijiang/index.ts"],
  "skills": ["./skills"],
  "prompts": ["./prompts"],
  "agents": []
}
```

### 生命周期 Hook

| Hook | 触发时机 | 扩展行为 |
|------|---------|----------|
| before_agent_start | agent 启动前 | 1. 通过 `dijiang workflow-state` 刷新状态栏和 widget |
| | | 2. 通过 `dijiang dispatch --json --hook-event` 分类提示 |
| user_prompt_submit | 用户提交提示 | 同上 |
| tool_call | 任意工具调用 | 向 bash 命令注入 `DIJIANG_CONTEXT_ID` 环境变量 |
| tool_result | 工具返回结果 | 1. 刷新状态栏和 widget |
| | | 2. bash 命令失败 → 注入 `<dijiang-route>` 路由到 dj-hunt |
| | | 3. 验证命令通过且有脏 diff → 注入 `<dijiang-route>` 路由到 dj-output |
| session_start | session 开始 | 刷新状态栏和 widget |
| session_shutdown | session 关闭 | 刷新状态栏和 widget |

### UI 组件

扩展提供两个 UI 组件，信息源来自 `dijiang workflow-state --json`：

**状态栏（Status Bar）**：两个条目
- `dijiang-task` — 显示 `{任务标题} [{capsule}]`
- `dijiang-capsule` — 显示 capsule 状态

**Widget**：显示详细信息行
```
任务: {任务标题} | 状态: {idle/in_progress/completed} | Capsule: {capsule} | Gate: {ready/provisioned/blocked}
```

### 自动路由注入

扩展在 `tool_result` hook 中实现两项自动化路由：

1. **bash 命令失败** → 注入 `<dijiang-route>` 消息（类型 `dijiang_route`），路由到 `dj-hunt`，附带失败命令内容。
2. **验证/检查通过且有脏 diff** → 注入 `<dijiang-route>` 消息，路由到 `dj-output`，附带命令内容。

每次注入去重（按 session key + command 组合），避免同一问题重复路由。消息通过 `deliverAs: "steer"` 发送，确保被 agent 优先处理。

### Prompt 模板

两个 Pi prompt 模板作为轻量检查清单：
- `/dijiang-start` — 读取 `dijiang task current` 和 `workflow.md`，注入 DiJiang 上下文
- `/dijiang-finish-work` — 验证、检查、文档同步、版本决策、记忆记录、收尾执行的步骤清单

## 关键设计决策

- **CLI 作为集成层**：所有跨 crate 编排在 `cli/main.rs` 中。库 crate 互不知晓。
- **门禁式工况约束**：路由约束在运行时硬编码（非仅提示级别）。active task 状态阻止无效转移。
- **Worktree 隔离**：代码修改任务自动供应隔离 git worktree。主 checkout 保持干净。
- **渐进式 skill 加载**：Skill body 不预先注入 agent 提示。先暴露清单，按需懒加载完整 body，通过 `dijiang skill-body <name>` 获取。
- **Trellis 向后兼容**：任务状态到 Trellis 状态的映射为有损转换。`.trellis/` 作为遗留读回退；`.dijiang/` 是主要状态路径。
- **Pi 扩展主导的运行时集成**：agent 不直接调用 CLI 路由，由 Pi 扩展在生命周期事件中自动注入路由上下文和 `<dijiang-route>`。agent 消费注入内容而非主动调用路由。
- **子 agent 职责分离**：check、implement、research 三个子 agent 分别处理质量、实现和调研，各自加载对应 skill 清单。agent prompt 首行必须读取 `workflow-state`。

## 项目目录结构

### `.dijiang/`（CLI 状态目录）

```
.dijiang/
├── tasks/<id>/task.json   # 每个任务的状态
├── spec/                   # 逐层编码规范
│   ├── backend/
│   ├── frontend/
│   ├── guides/
│   └── meta/               # ADR 模板、贡献指南
├── workspace/              # 开发者日志（每个 session 一个）
├── workflow.md             # 规范工作流投影
└── config.toml             # DiJiang 配置
```

### `.pi/`（Pi 平台目录）

如上文 Skill 系统的文件结构所示，`settings.json` 统一注册 skills、extensions、prompts。
