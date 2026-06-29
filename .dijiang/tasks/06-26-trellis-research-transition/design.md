# Trellis 架构深度分析与 DiJiang 对比

## 1. Trellis 源码架构全景

### 1.1 Monorepo 结构

```
Trellis/
├── packages/
│   ├── core/          # @mindfoldhq/trellis-core  — 核心逻辑库
│   │   └── src/
│   │       ├── channel/     # Worker 进程管理（事件溯源模式）
│   │       ├── mem/         # 跨会话记忆系统
│   │       ├── task/        # 任务生命周期管理
│   │       └── testing/     # 测试工具
│   └── cli/           # @mindfoldhq/trellis  — 命令行工具
│       └── src/
│           ├── configurators/  # 18+ 平台适配器
│           ├── commands/       # 用户可调用斜杠命令
│           ├── skills/         # 自动触发技能
│           └── templates/      # 平台无关上下文模板
├── scripts/           # 共享 Python 脚本
├── .trellis/          # Trellis 自身的 .trellis 配置（自举）
└── docs/
```

管理方式: pnpm workspace，两个包共享 `@mindfoldhq` scope。

### 1.2 Core 层详解

#### 1.2.1 Channel 模块 — 事件溯源 Worker 管理

**设计模式**: Event Sourcing（事件溯源）。所有 worker 操作通过 append-only event store 记录，state 是事件的 projection。

```
api/           # 公开 API
├── spawn.ts    # spawnWorker: resolve channel → start runtime → append "spawned" event
├── kill.ts     # 终止 worker
├── inbox.ts    # 读取 worker 消息
├── send.ts     # 向 worker 发送消息
├── read.ts     # 读取 channel state
├── list.ts     # 列出所有 channels
├── watch.ts    # 监听 channel 变化
├── context.ts  # Channel 上下文（runtime 注入点）
└── workers.ts  # Worker 状态查询

internal/      # 内部实现
├── store.ts    # Event store（持久化 + 内存缓存）
├── state.ts    # State projection（从 events 重建 state）
├── seq.ts      # 序列号管理（严格递增，一个 channel 一个 seq）
├── schema.ts   # Event / State 类型定义
└── runtime.ts  # Runtime 接口（抽象 process 启动/停止）
```

**关键设计决策**:
- Runtime 是注入点 — 不同平台注入不同的 runtime 实现
- Event store 支持持久化（文件系统）和内存两种后端
- Sequence 是 channel 维度的，保证每个 channel 内严格有序

**DiJiang 对比**:
- DiJiang 无等价物 — 无 worker 管理，无事件溯源
- 如需 CI/CD 集成或并行 agent 管理，channel 模块是关键基础设施
- **转型建议**: 阶段 2 引入，非优先

#### 1.2.2 Mem 模块 — 跨会话记忆系统

```
mem/
├── projects.ts    # 会话按项目（cwd）聚合
├── dialogue.ts    # 对话管理
├── sessions.ts    # 会话记录（title, agentId, tags, parent tree）
└── adapters/      # 平台适配器
    ├── claude.ts
    ├── codex.ts
    ├── pi.ts
    └── opencode.ts
```

**会话模型**:
```typescript
interface TrellisSessionRecord {
  sessionId: string;
  projectId: string;
  cwd: string;
  agentId: string;       // 平台标识
  workspaceId: string;
  title: string;
  tags: string[];
  parentSessionId: string | null;  // 支持会话树
  status: "active" | "completed" | "archived";
  startedAt: string;
  lastActiveAt: string;
  archivedAt: string | null;
}
```

**projects.ts 聚合逻辑**:
```typescript
listMemProjects() → 扫描所有 session
  → 按 cwd 分组
  → 统计每项目每平台 session 数
  → 按 lastActiveAt 排序
```

**Adapter 模式**: 不同平台的 session 格式不同，adapter 负责转换。
- Pi adapter: 读取 `.pi/sessions/` 目录
- Claude adapter: 读取 Claude session 文件
- 每个 adapter 实现统一的 `SessionAdapter` 接口

**DiJiang 对比**:
- DiJiang 的 `dj-muse/` 是 Go 实现的单平台记忆系统
- 无 adapter 模式，无多平台聚合
- **转型建议**: 阶段 3，将 dj-muse Go 实现重构为 adapter 模式，保留 Go 性能优势，增加 adapter 接口

#### 1.2.3 Task 模块 — 24 字段结构化任务

**核心 Schema** (TrellisTaskRecord):
```typescript
interface TrellisTaskRecord {
  // 标识
  name: string;           // slug
  title: string;          // 显示标题
  status: TaskStatus;     // planning|in_progress|completed|archived|paused
  priority: string;       // P0-P5

  // 开发者
  developer: string | null;
  assignee: string | null;

  // 时间
  createdAt: string | null;
  startedAt: string | null;
  completedAt: string | null;
  archivedAt: string | null;

  // 状态
  acceptanceCriteria: string | null;
  keyDeliverables: string | null;

  // 上下文
  source: string | null;          // 触发来源（session ID 等）
  sessionId: string | null;       // 关联会话
  sessionSummary: string | null;  // 会话摘要
  branch: string | null;          // Git 分支

  // 关系
  parentTask: string | null;     // 父任务
  subtasks: string | null;       // 子任务 JSON

  // 估计
  estimatedEffort: string | null;
  actualEffort: string | null;

  // 评审
  reviewStatus: string | null;
  reviewComments: string | null;

  // 标签
  tags: string | null;
  notes: string | null;
}
```

**Phase 投影** (phase.ts):
```typescript
inferTaskPhase(status) →
  "planning" → "plan"
  "in_progress" → "implement"
  "paused" → "implement"  // 暂停仍在实现阶段
  "completed" → "complete"
  "archived" → "archive"
  else → "unknown"
```

Phase 是 status 的粗粒度投影，不独立存储。

**路径解析** (paths.ts):
```
.trellis/tasks/{name}/
├── task.json            # TrellisTaskRecord
├── prd.md               # 需求文档（可选）
├── design.md            # 设计文档（可选，复杂任务）
├── implement.md         # 实施计划（可选，复杂任务）
├── check.jsonl          # 审查上下文（JSON Lines）
└── implement.jsonl      # 实现上下文（JSON Lines）
```

**DiJiang 对比**:
- DiJiang 的 `task.py` 有相似的 task.json 但字段较少（~12 字段）
- DiJiang 无 phase 投影概念
- **转型建议**: 阶段 0，对齐 TrellisTaskRecord schema（全部 24 字段），保持兼容

### 1.3 CLI 层详解

#### 1.3.1 Configurator 体系 — 平台适配核心

**平台分类**:
```typescript
// shared.ts — 全局平台注册表
interface PlatformMeta {
  key: string;
  label: string;
  configKey?: string;
  suggestions?: string[];
  agentCapable?: boolean;  // 支持子代理
  hasHooks?: boolean;       // 支持 session-start hooks
}
```

**分类含义**:
| 特征 | 含义 | 注入方式 |
|------|------|----------|
| `hasHooks=true` | 平台支持 session-start 生命周期钩子 | Push: hooks 自动注入上下文 |
| `hasHooks=false` | 平台无钩子 | Pull: agent 需主动加载上下文 |
| `agentCapable=true` | 支持子代理 | start 技能可过滤（hook 处理） |
| `agentCapable=false` | 无子代理 | 保留 start 技能作为备选 |

**Configurator 实现模式** (以 Pi 为例):
```typescript
export async function configurePi(context: TemplateContext) {
  // 1. 解析模板变量（skill 列表、command 列表等）
  const resolved = resolvePlaceholders(context);

  // 2. 为每个 skill 生成 .pi/skills/{skill_name}/SKILL.md
  for (const skill of resolved.skills) {
    writeFile(`.pi/skills/${skill.name}/SKILL.md`, skill.content);
  }

  // 3. 为每个 command 生成配置
  for (const cmd of resolved.agentCommands) {
    writeCommand(cmd);
  }

  // 4. 写入 platform-specific 文件（AGENTS.md、.pi/prompt-templates/ 等）
  writeAgentInstructions(resolved);
}
```

**支持的 18+ 平台**:
Claude Code, Cursor, Codex, Pi, Windsurf, Gemini CLI, Kilo Code, Kiro, Qwen Code, OpenCode, ZCode, Augment Code, Continue, Crush, Factory, GitHub Copilot, Roo Code, Amazon Q Developer CLI

#### 1.3.2 Template 系统 — 平台无关上下文注入

```typescript
// 核心概念: TemplateContext — 平台无关的上下文模型
interface TemplateContext {
  // 项目信息
  projectName: string;
  specDirs: string[];

  // 任务信息
  activeTask: TaskContext | null;
  tasks: TaskContext[];

  // 技能系统
  skills: SkillDef[];
  agentCommands: CommandDef[];

  // 平台信息
  platform: PlatformMeta;

  // 工作区
  workspace: WorkspaceContext;

  // 注入控制
  injectionMode: "push" | "pull";
  contextFiles: string[];
}
```

Templates 是基于 TemplateContext 渲染的模板，不同平台共享同一 TemplateContext 但生成不同的输出文件。

#### 1.3.3 Skills 系统 — 自动触发 vs 用户调用

Trellis skill 分两类:
1. **自动触发技能**: start, continue, finish-work, before-dev, brainstorm, check, break-loop, update-spec
2. **用户可调用技能**: 通过斜杠命令或 `/trellis:xxx` 触发

Skill 定义格式:
```markdown
---
name: trellis-start
description: Start a Trellis session
triggers:
  - session:start
---
# Trellis Start
...instructions...
```

**DiJiang 对比**:
- DiJiang 的 9 个 dj-* 技能 + 9 个 trellis-* 技能（共 ~18 个）
- DiJiang 的技能定义在 `.pi/skills/` 下，格式与 Trellis skill 不同
- DiJiang 的技能是独立设计的，不完全对齐 Trellis skill 系统
- **转型建议**: 
  - 保留 dj-* 技能体系（dispatch, grill, tdd, hunt, check, output, implement, design, muse, prototype, script）作为 DiJiang 差异化资产
  - 对齐 trellis-* 技能（start, continue, finish-work, before-dev, brainstorm, check, break-loop, update-spec）到 Trellis 标准
  - 建立 skill 注册和触发机制

#### 1.3.4 Hooks 系统 — 自动注入 vs 手动加载

**Class-1 (push) 工作流**:
```
Agent 启动
  → Hook 触发 (session:start)
  → 调用 get_context.py
  → 生成上下文 JSONL
  → 注入 Agent 系统提示
  → Agent 获得完整项目上下文
```

**Class-2 (pull) 工作流**:
```
Agent 启动
  → 无 hook，Agent 看到空上下文
  → 用户需调用 /trellis:start 或等效命令
  → trellis-start skill 触发
  → 加载上下文 JSONL
  → Agent 获得完整项目上下文
```

**DiJiang 当前状态**:
- DiJiang 仅 Pi 平台，Pi 属于 class-1（hasHooks=true）
- 上下文注入通过 `get_context.py` 实现（手动调用）
- 无 hook 机制 — 每次需要手动加载上下文
- 实际上 DiJiang 处于"有 hook 能力但未完全利用"状态

### 1.4 Python Scripts 层

Trellis 的 Python 脚本负责:
- `task.py`: 任务 CRUD、状态管理、生命周期钩子
- `get_context.py`: 上下文生成（项目信息、任务信息、git 状态、workspace 日志）
- `git_context.py`: Git 分支/变更上下文
- `workflow.py`: 工作流状态注入

这些脚本与 `trellis-core` 共享数据结构约定（task.json schema），但独立实现。

## 2. DiJiang 当前架构分析

### 2.1 范围界定

DiJiang **拥有自身完整的 `.dijiang/` 基础设施**（项目级）和 `~/.dijiang/`（用户级），以及 `.pi/` 平台配置。
.trellis/` 用于 Trellis 兼容格式，Pi 是运行时载体。
DiJiang 与 Trellis 是平行关系：DiJiang 以 `.dijiang/` 为主权目录，兼容 `.trellis/` 格式。

架构分层:
```
  ┌─────────────────────────────────┐
  │  DiJiang (dj-* 技能体系)        │  ← 差异化增强层
  │  - dj-dispatch / dj-grill        │
  │  - dj-tdd / dj-hunt / dj-check  │
  │  - dj-muse (Rust mem 模块)      │
  ├─────────────────────────────────┤

  │  `.dijiang/`（DiJiang 主权目录）│  ← 配置 / Workspace / Hooks / Spec

  │  - .dijiang/config.toml          │

  │  - .dijiang/workspace/  (日志)   │

  │  - .dijiang/hooks/     (注入点)  │

  │  - .dijiang/spec/      (规范)    │

  ├─────────────────────────────────┤

  │  `~/.dijiang/`（用户级全局存储）│  ← Mem 会话 / 全局配置 / 缓存

  │  - ~/.dijiang/mem/sessions.db    │

  │  - ~/.dijiang/config.toml        │

  ├─────────────────────────────────┤

  │  `.trellis/`（Trellis 兼容层）   │  ← 任务管理 / Spec（可读写兼容）

  │  - .trellis/scripts/  (task.py)  │

  │  - .trellis/tasks/    (task.json)│

  │  - .trellis/spec/     (规范)     │

  ├─────────────────────────────────┤

  │  `.pi/`（Pi 平台载体）          │  ← 运行时载体

  │  - .pi/skills/  (技能注册)      │

  │  - .pi/prompt-templates/        │

  └─────────────────────────────────┘
```

### 2.2 DiJiang 当前代码分布（迁移前）

```
├── .dijiang/          # （待建 — Rust CLI 创建）
├── .trellis/          # 当前 Python 基础设施（待迁移到 Rust）
│   ├── scripts/
│   │   ├── task.py          # Python 任务管理 CLI
│   │   ├── get_context.py   # 上下文生成
│   │   ├── get_developer.py # 开发者识别
│   │   ├── init_developer.py
│   │   └── common/          # 共享库
│   │       ├── config.py
│   │       ├── git_context.py
│   │       ├── hooks.py
│   │       └── task.py
│   ├── spec/          # 规范目录（已填充：backend 6篇, frontend 7篇, guides 3篇, meta 1篇）
│   │   ├── frontend/index.md
│   │   └── guides/
│   └── workspace/     # 工作日志
│
├── .pi/
│   └── skills/        # 技能定义
│       ├── dj-*/
│       └── trellis-*/
│
├── AGENTS.md
├── coding-workflow.md
└── .trellis/config.yaml
```

### 2.2 技能体系

**DiJiang 独有（dj-*）**:
| 技能 | 功能 | Trellis 等价物 |
|------|------|---------------|
| dj-dispatch | 任务类型识别 + S/M/L 分级 | 无（Trellis 无此概念） |
| dj-grill | 需求深度对齐（一次一问） | brainstorm（但力度不同） |
| dj-output | 文档产出（PRD/设计/实施） | 无独立技能 |
| dj-implement | 代码实现 | start + before-dev（但不自动） |
| dj-tdd | 测试驱动开发 | 无 |
| dj-hunt | 代码定位与排查 | 无（Trellis 依赖 agent 自身能力） |
| dj-check | 多维度审查 | check（部分重叠） |
| dj-design | 设计文档 | 无 |
| dj-muse | 记忆管理 | mem 模块 |
| dj-script | 脚本执行 | 无 |
| dj-ponytail | 代码质量 | 无 |
| dj-handoff | 任务交接 | 无 |
| dj-prototype | 原型开发 | 无 |

**Trellis 已有（在 DiJiang 中对应 trellis-*）**:
| 技能 | 功能 |
|------|------|
| trellis-start | 启动会话，加载上下文 |
| trellis-continue | 继续之前的会话 |
| trellis-finish-work | 完成工作，提交评审 |
| trellis-before-dev | 编码前准备（读规范、查任务） |
| trellis-brainstorm | 头脑风暴 |
| trellis-check | 审查 |
| trellis-break-loop | 跳出死循环 |
| trellis-update-spec | 更新规范 |
| trellis-session-insight | 会话洞见 |
| trellis-spec-bootstrap | 规范引导 |
| trellis-channel | Channel 管理 |
| trellis-meta | 元操作 |

## 3. 详细对比矩阵

### 3.1 架构层面

| 维度 | Trellis | DiJiang | 差距评估 | 优先级 |
|------|---------|---------|----------|--------|
| 技术栈 | TypeScript monorepo (Trellis) | Python + Go + TS 混合 → **全量 Rust** | 迁移成本高，但一次到位 | P0 |
| 包管理 | pnpm workspace | 无 monorepo 结构 | 需建立 | P1 |
| 构建系统 | tsc + esbuild | 无（脚本语言） | NA（如保留 Python/Go） | — |
| 测试 | vitest (core), 无 (cli) | 无 | 需建立 | P2 |
| 代码质量 | Husky + lint-staged | 无 | 需建立 | P2 |
| 发布 | npm publish + GitHub releases | 无 | 需建立 | P3 |

### 3.2 功能层面

| 维度 | Trellis | DiJiang | 差距 | 优先级 |
|------|---------|---------|------|--------|
| 平台支持 | 18+ configurators | 仅 Pi | **核心差距** | P0 |
| 注入机制 | Hooks auto-inject + pull | 手动 | **核心差距** | P0 |
| 任务模型 | 24 字段 TrellisTaskRecord | ~23 字段（命名差异） | 字段名对齐 + 补缺 | P1 |
| 任务分级 | Phase 投影（5 级） | S/M/L 三级 + 路由 | DiJiang 更优 | — |
| Worker 管理 | Event sourcing channel | 无 | nice-to-have | P2 |
| 记忆系统 | Adapter 模式多平台 | Go 单平台 + vNext 文档 | 多平台 adapter（文档已写好） | P1 |
| CLI 工具 | trellis init/start/check | 仅有 task.py | 需构建（TypeScript monorepo） | P1 |
| 模板系统 | 平台无关 TemplateContext | 基础拼接 | 需模板化（Trellis 框架层职责） | P1 |
| Skill 系统 | 自动触发 + 用户调用 | dj-* 独立设计 | 需建立注册/触发机制 | P0 |
| Hooks 系统 | Python hooks (pre/post) | 部分 hooks | 需完整实现（Trellis 框架层） | P1 |
| 规范系统 | per-package spec | 空壳 | 需填充（Trellis 框架层） | P1 |
| 规范系统 | per-package spec | 已有 backend(6)/frontend(7)/guides(3)/meta(1) | 继续补充 | P2 |

### 3.3 设计模式

| 模式 | Trellis 使用 | DiJiang 使用 | 建议 |
|------|-------------|-------------|------|
| Adapter | mem/adapters/, channel/runtime | 无 | 引入 |
| Event Sourcing | channel 模块 | 无 | 按需引入 |
| Strategy | configurators (per-platform) | 无 | 引入 |
| Template Method | shared.ts → platform configurators | 无 | 引入 |
| Registry | PLATFORM_FUNCTIONS map | 无 | 引入 |
| Dependency Injection | Runtime 注入点 | 无 | 引入 |

### 3.4 DiJiang 超越 Trellis 的能力

这些是 DiJiang 的**差异化资产**，转型中应保留并强化:

| 能力 | 说明 | Trellis 能覆盖吗 |
|------|------|-----------------|
| dj-dispatch | 智能任务分流（15 种类型 + S/M/L 分级） | 否 — Trellis 无任务分类概念 |
| dj-grill | 深度需求对齐（结构化提问 + 推荐答案） | 部分 — brainstorm 无结构化提问框架 |
| dj-tdd | TDD 驱动开发 | 否 |
| dj-hunt | 深度代码定位 | 否 — 依赖 agent 原生搜索能力 |
| dj-handoff | 多 agent 任务交接 | 否 |
| dj-ponytail | 代码质量审查 | 否 |
| 中文输出 | 全中文工作流 | 否 |
| Go 全链路记忆 | 高性能 session 处理 | 否 — TS 实现 |

## 4. 转型路线图

### 阶段 0: `.trellis/` 基础设施完善（小修）

**实际状态：已基本就绪。** `task.py` 当前支持 ~23 字段，
spec/ 已有 backend(6篇)/frontend(7篇)/guides(3篇)/meta(1篇)，
workspace/ 已有 journal 格式。

**剩余工作**:
1. 字段命名对齐（creator→developer/source, parent→parentTask）
2. 补缺字段（startedAt, archivedAt, acceptanceCriteria 等）
3. task.json 字段顺序对齐
4. workspace journal 格式确认

**验收**: `trellis status` / `trellis list` 正确输出（兼容，非依赖）。
**验收**: `trellis status` 和 `trellis list` 在 DiJiang 项目中正确输出。

### 阶段 1: DiJiang CLI + Mem 多平台（核心工程化，优先级最高）

**目标**: 构建 `dijiang` CLI（TypeScript monorepo），**同时完成 Mem 多平台 adapter**。

⚠️ Mem 多平台是阶段 1 的核心交付物，不是阶段 3 的后续任务。configurator 先做 Pi，但 Mem 必须能读取 Claude、Codex、Cursor 等平台的 session 数据。

**具体任务**:

**1A. Monorepo 初始化**
- 选项 A: TypeScript — **推荐**（可复用 Trellis 类型 + pnpm workspace）
- `packages/core/` — dijiang-core（任务管理、Skill 注册、Configurator 接口）
- `packages/cli/` — dijiang CLI
- tsconfig + esbuild 构建配置

**1B. Mem 多平台 Adapter（最高优先级）**
- 定义 `MemAdapter` 接口:
  ```typescript
  interface MemAdapter {
    provider: string;
    listSessions(): Promise<SessionRecord[]>;
    getSession(id: string): Promise<SessionRecord>;
    getDialogue(id: string): Promise<DialogueEntry[]>;
  }
  ```
- **Pi MemAdapter**: 复用现有 dj-muse Go 逻辑，通过 adapter 包装
- **Claude MemAdapter**: 读取 Claude session 文件格式
- **Codex MemAdapter**: 读取 `.codex/sessions/` 或等效路径
- **Cursor MemAdapter**: 读取 Cursor session 数据
- `dijiang mem list` — 跨平台聚合项目统计（按 cwd 分组、按 lastActiveAt 排序）

**1C. Configurator 体系（先 Pi）**
- 定义 DiJiangPlatformMeta（扩展 hasDJSkills 字段）
- 实现 ConfiguratorRegistry
- 第一个 configurator: Pi（完整实现，生成 `.pi/skills/dj-*/`）

**1D. CLI 命令**
- `dijiang init` — 初始化项目
- `dijiang start` — 启动会话
- `dijiang status` — 项目状态
- `dijiang mem list` — 跨平台记忆列表

**验收**:
- `dijiang init` 生成完整 `.trellis/` + `.pi/` 目录
- `dijiang mem list` 显示 Pi 和 Claude session（至少两个平台的 session 聚合显示）

### 阶段 2: 多平台 Configurator 扩展

**目标**: Configurator 从 Pi 扩展到 Cursor + Codex + Claude，使 dj-* 技能在更多平台可用。

注意：Mem adapter 已在阶段 1 就绪，此阶段 configurator 直接使用。

**具体任务**:
1. 实现 Cursor configurator（`.cursor/rules/`，hasHooks=true）
2. 实现 Codex configurator（`.codex/`，hasHooks=false，pull 模式）
3. 实现 Claude configurator（`CLAUDE.md` + `.claude/`，hasHooks=true）
4. 跨平台一致性测试

**验收**: 同一 DiJiang 项目在 Pi/Cursor/Codex/Claude 中均正常工作。

### 阶段 3: dj-* 技能增强与生态

### 阶段 3: dj-* 技能增强与生态

**目标**: 强化 DiJiang 差异化竞争力，新增技能、优化现有流程。

**已完成（超前交付）**:
- ✅ dj-dispatch: 自动激活（session:start）+ phase 映射
- ✅ dj-grill: Phase 标记 + 自动恢复
- ✅ dj-output: TemplateContext 模型 + Spec 更新合约（7 章节）
- ✅ dj-hunt: spec 晋升合约模板（7 章节）
- ✅ dj-muse: 多平台 Session 聚合（vNext 文档）

**后续**:
1. 新增 dj-audit（全仓审计）、dj-pattern（模式识别）、dj-review（代码评审）
2. 增强 dj-grill 提问策略（多轮追问深度）
3. 增强 dj-hunt 代码定位能力
4. 文档、社区建设、CI/CD


## 5. 架构决策记录

### ADR-1: 技术栈选择

**问题**: DiJiang 用什么语言实现？

**选项**:
- A: TypeScript（对齐 Trellis 生态，但仍有运行时依赖）
- B: Python（保留现有 task.py，但 CLI 启动慢 + Go dj-muse 仍分离）
- C: Go（单二进制，dj-muse 零迁移，但 Python task.py 需重写）
- D: **Rust（选定）**（单二进制，零运行时，类型最强，全部重写）

**决策**: **D — Rust**
**理由**:
- 单二进制分发，用户无需安装任何运行时
- 零启动延迟（~2ms vs Node ~50ms / Python ~30ms）
- 最强的类型安全（enum + Result + serde，task schema 校验不可绕过）
- clap（CLI 解析）+ serde（JSON/schema）+ thiserror（错误处理）生态成熟
- 跨平台编译一行搞定（`--target x86_64-unknown-linux-gnu`）
- 与 Trellis 不共享运行时，降低 AGPL 牵连风险

**代价**:
- Python task.py（7.6k 行）全部重写
- Go dj-muse（7.2k 行）全部重写
- TypeScript packages（554 行）废弃
- 开发速度初期低于 Python/TS

**缓解**: 按 crate 分步迁移（task → mem → core → cli），优先 task CRUD（纯文件 I/O，Rust 表达简洁），保持 `.trellis/tasks/` 的 task.json schema 不变

### ADR-2: Trellis 兼容性 vs DiJiang 差异化

**问题**: 是否需要 100% 兼容 Trellis？

**决策**: **协议兼容，能力超越**
- `.trellis/` 目录结构 100% 兼容（Trellis CLI 可读 DiJiang 项目）
- `task.json` schema 100% 兼容
- Skill 格式 100% 兼容
- CLI 命令命名可与 Trellis 不同（如 `dijiang grill` vs 无对应）
- dj-* skills 是 DiJiang 独家（Trellis configurator 不生成这些）

### ADR-3: 开源协议

**问题**: DiJiang 如何与 AGPL-3.0 的 Trellis 共存？

**决策**:
- DiJiang 可依赖 Trellis 的 npm 包并使用其 API（AGPL 允许链接）
- 如需修改 Trellis 源码并分发，DiJiang 也需 AGPL-3.0
- DiJiang 自有代码（dj-* skills）可任意协议
- 建议 DiJiang 自有代码使用 MIT，依赖声明包含 `@mindfoldhq/trellis`
- 建议 DiJiang 自有代码使用 MIT，依赖声明包含 `@mindfoldhq/trellis`

### ADR-4: DiJiang 范围界定与 Mem 优先级

**问题**: DiJiang 的范围是什么？Mem 多平台何时做？

**决策**:
- DiJiang = dj-* 技能体系 + `.trellis/` 基础设施 + `.pi/` 平台配置
- 架构分层: DiJiang 基础设施（骨架） → DiJiang 技能（能力注入） → 平台（运行载体）
- `.trellis/` 和 `.pi/` 都是 DiJiang 自有，与 Trellis CLI 兼容但不依赖
- Mem 多平台适配器是阶段 1 的核心交付物（P0），不推迟到后续阶段
- 理由: dj-* 技能需要跨平台 session 数据才能提供差异化能力（如 dj-grill 参考历史会话、dj-hunt 跨平台上下文）
- configurator 可滞后（先 Pi，后续扩展），但数据层必须先行

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Trellis API 不兼容更新 | 中 | 高 | 锁定版本，上游参与贡献 |
| dj-* 技能迁移后丢失功能 | 中 | 高 | 保留原始 skill 文件作为备份 |
| 多平台测试复杂度 | 高 | 中 | 优先 Pi → Codex，逐步扩展 |
| Rust 人才缺口 | 中 | 中 | cargo workspace 分 crate 迁移，降低单 crate 复杂度；项目规模适中（~15k 行），一人维护可行 |
| AGPL 协议风险 | 低 | 高 | 法律顾问审查，自有代码 MIT |
