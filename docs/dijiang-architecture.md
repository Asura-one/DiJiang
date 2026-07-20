# DiJiang 架构：Skill · CLI · Agent · Hook · Workflow

> 日期：2026-07-19
> 范围：DiJiang 核心概念及其关系，与 Trellis 实现对比

---

## 一、概念总览

```
┌─────────────────────────────────────────────────────────────┐
│                       AI Agent 会话                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │               Workflow State（per-turn）               │  │
│  │  ┌──────┐  ┌──────────┐  ┌────────┐  ┌──────────┐   │  │
│  │  │Task  │  │Route Gate│  │  Git   │  │  Loop    │   │  │
│  │  │State │  │          │  │  Gate  │  │  State   │   │  │
│  │  └──────┘  └──────────┘  └────────┘  └──────────┘   │  │
│  │  ┌──────────────┐  ┌────────────────────────┐       │  │
│  │  │Skill Manifests│  │Target Skill（compact） │       │  │
│  │  └──────────────┘  └────────────────────────┘       │  │
│  │  ┌──────────────┐  ┌──────────┐                     │  │
│  │  │Agent（compact│  │  Memory  │                     │  │
│  │  └──────────────┘  └──────────┘                     │  │
│  └───────────────────────────────────────────────────────┘  │
│                         │ Dispatch                              │
│                         ▼                                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    dj-dispatch                        │  │
│  │  分类请求 → 选择 Capsule → 确定 Skill → 注入上下文      │  │
│  └───────────────────────────────────────────────────────┘  │
│                         │                                      │
│          ┌──────────────┼──────────────┐                     │
│          ▼              ▼              ▼                     │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │ dj-grill   │ │dj-implement│ │ dj-check   │  ... skills  │
│  │（对齐）    │ │（实现）    │ │（审查）    │              │
│  └────────────┘ └────────────┘ └────────────┘              │
│                         │                                      │
│          ┌──────────────┘              │                     │
│          ▼                             ▼                     │
│  ┌────────────┐               ┌────────────┐              │
│  │ Agent FC   │               │ Agent FC   │              │
│  │（immediate │               │（channel   │              │
│  │  inline）  │               │  spawn）   │              │
│  └────────────┘               └────────────┘              │
│                         │                                      │
└─────────────────────────┼──────────────────────────────────────┘
                          │ Hook injection（每轮）
                          ▼
              ┌────────────────────────┐
              │  Platform Hook Runner  │
              │ 注入 Workflow State    │
              └────────────────────────┘
```

## 二、概念定义与关系

### 2.1 Skill（技能）

**DiJiang 定义**：一个 Skill 是 markdown 文件（`SKILL.md`），描述 AI 在特定阶段应遵循的行为标准、步骤和约束。Skill 是 DiJiang 的核心执行单元。

| 属性 | 说明 |
|------|------|
| 存放位置 | `.pi/skills/dj-*/SKILL.md` |
| 编译时清单 | `crates/task/src/skill_manifest.rs` |
| 命名格式 | `dj-{name}`（如 `dj-implement`、`dj-check`） |
| 风险级别 | low / medium / high（用于 route gate 准入判断） |
| 生命周期阶段 | align / implement / check / finish / resume / idle |

**关系**：
- Skill 由 `dj-dispatch` 根据请求类型和 `route_gate.rs` 的 capsule 选择
- Skill 的知识注入方式：workflow-state 中包含 **Skill Manifests**（摘要列表）+ **Target Skill**（当前选中的 skill 摘要）
- Skill 可以引用 Agent（主 AI 直接执行 skill 定义，或在需要时 channel spawn 子 agent）

**Trellis 对比**：
- Trellis skill 以 `trellis-{name}` 命名（如 `trellis-implement`、`trellis-check`）
- Trellis skill 通过 platform-specific skill 目录分发（如 `codex/skills/*/SKILL.md`），不在 workflow-state 中枚举
- Trellis 无编译时 skill manifest，无 risk 系统，无 route gate 准入控制
- DiJiang 优势：编译时清单 + 路由门控 + 全量注入 vs Trellis 按需加载

### 2.2 CLI（命令行接口）

**DiJiang 定义**：`dijiang` 是一个 Rust 二进制文件（`crates/cli/`），提供所有基础设施操作。

| 命令类别 | 示例 | 职责 |
|---------|------|------|
| 项目 | `init`、`update` | 创建 `.dijiang/`、部署 agents/skills/hooks 到平台目录 |
| 任务 | `task create`、`task start`、`task archive` | 任务生命周期管理 |
| 会话 | `start`、`finish-work` | Session 生命周期 |
| 状态 | `status`、`workflow-state` | 查询项目状态、输出上下文注入 |
| 通道 | `channel spawn`、`channel list` | 多 agent 编排 |
| 记忆 | `mem` | 持久化记忆（tactic/pattern/finding） |
| 路由 | `dispatch` | 请求分类 + skill 选择 |
| 工具 | `skills`、`template` | 枚举可用 skill、管理模板 |

**关系**：
- CLI 是 **Agent 的操作接口**：Agent 读取 `dijiang workflow-state` 获取上下文，调用 `dijiang task start` 推进状态
- CLI 是 **Hook 的数据源**：Hook 脚本调用 `dijiang workflow-state --json` 获取 JSON payload 注入会话
- CLI 是 **Skill 的部署器**：`dijiang init` / `dijiang update` 将 skill/agent/hook 文件写入平台目录
- CLI 不自包含业务逻辑——大多数命令委托给 `dijiang-task`、`dijiang-mem`、`dijiang-configurator` 等 crate

**Trellis 对比**：
- Trellis CLI 是 npm 包（`@mindfoldhq/trellis`），日常任务管理委托给 `.trellis/scripts/` 中的 Python 脚本
- Trellis Python 脚本从 `.trellis/` 读取配置，不依赖 CLI 可执行路径
- DiJiang 将全部逻辑编码在 Rust CLI 中，无辅助脚本层（Rust 二进制必须保证在 `$PATH` 中可用）
- 关键差异：**Trellis 有脚本层，DiJiang 无**。脚本层允许 AI "python3 .trellis/scripts/task.py current" 直接操作，无需 CLI 在 PATH 中

### 2.3 Agent（代理）

**DiJiang 定义**：Agent 是一个 markdown 文件（`{name}.md`），定义 AI 角色的行为规范。有两类：

| 类型 | 文件 | 用途 | 注入方式 |
|------|------|------|---------|
| **平台 Agent** | `.pi/agents/dijiang-{name}.md` | sub-process channel spawn | `dijiang channel spawn {name}` 时加载 |
| **工作流 Agent** | 编译时 `agent_manifest.rs` | route gate 解析 + 上下文注入 | 仅 name + summary 注入 `workflow-state` |

**当前 5 个 Agent**：

| Agent | 名称 | 职责 |
|-------|------|------|
| Architect | `dijiang-architect` | 架构评审与设计回检 |
| Planner | `dijiang-planner` | 任务分解与结构化规划 |
| Implementer | `dijiang-implementer` | 代码实现与变更推进 |
| Checker | `dijiang-checker` | 质量审计与回归审查 |
| Researcher | `dijiang-researcher` | 技术调研与上下文收集 |

**关系**：
- Agent 是 **Skill 的执行者**：当 skill 决策需要子代理时，主 AI 通过 `channel spawn {agent}` 将子任务委派出去
- Agent 的知识仅按需加载：workflow-state 只注入 `name + summary`（compact 格式），完整 persona 只在 spawn 时读取
- Agent 文件同时存放在模板源和部署目录，通过 `dijiang init` / `dijiang update` 同步
- Agent 命名使用 `dijiang-` 前缀，与 skill 的 `dj-` 前缀形成对称命名空间

**Trellis 对比**：
- Trellis agent 以 `trellis-{name}` 命名（如 `trellis-implement`、`trellis-check`）
- Trellis 只有 implement、check、research 三个 agent，无 architect/planner
- Trellis agent 定义在 `.trellis/agents/implement.md`，部署到平台目录（如 `.claude/agents/trellis-implement.md`）
- Trellis 的 agent 定义使用 YAML frontmatter + markdown 职责描述，结构一致
- DiJiang 扩展了 5 个 agent 角色（含 architect/planner），agent 定义合并了 persona + channel 引导

### 2.4 Hook（钩子）

**DiJiang 定义**：Hook 是平台事件触发脚本，由 `dijiang init` 或 `dijiang update` 部署到各平台目录。

| Hook | 触发时机 | 输出 |
|------|---------|------|
| `inject-workflow-state.py` | 每轮 user prompt 提交 | `<workflow-state>` 注入 AI 会话 |
| `session-start.js` | Session 启动 | 初始上下文 |

**关系**：
- Hook 是 **Workflow-State 的注入通道**：每轮交互前，platform runner 执行 hook，hook 调用 `dijiang workflow-state --json` 获取状态
- Hook 对 AI 透明：AI 不需要知道 hook 的存在或配置
- Hook 文件由 `configurator` crate 在 `init`/`update` 时写入各平台目录
- Hook 不是 skill、不是 agent——它是纯基础设施

**Trellis 对比**：
- Trellis 使用几乎相同的 `inject-workflow-state.py` hook
- Trellis hook 直接从 workflow.md 解析 `[workflow-state:STATUS]` 标签块，不依赖 CLI
- DiJiang hook 调用 `dijiang workflow-state --json` CLI 命令，依赖 CLI 在 PATH 中
- Trellis 有更丰富的 hook 生态：`session-start.py`（平台适配）、`inject-shell-session-context.py`、`inject-subagent-context.py`
- Trellis 的 `inject-workflow-state.py` 实现了平台主动检测（Cursor/Codex/Claude/Gemini/Qoder/CodeBuddy/Droid/Copilot/Kiro），DiJiang 的 hook 走统一 JSON 格式输出

### 2.5 Workflow（工作流）

**DiJiang 定义**：Workflow 是项目目录中的 `.dijiang/workflow.md`，定义阶段映射和流程规则。同时包括运行时 WorkflowState 结构。

**两种形态**：

| 形态 | 位置 | 职责 |
|------|------|------|
| **静态 workflow.md** | `.dijiang/workflow.md` | 人可读的流程定义 + `[workflow-state:STATUS]` 标签块 |
| **运行时 WorkflowState** | `WorkflowState` 结构体 | 编译时注入的 AI 上下文块 |

**WorkflowState 注入结构**（优化后）：

```
<dijiang-workflow-state>
会话：...（session key/source）
注入：#N，时间：...（injection metadata）
Loop：...（goal/progress/next_action）
Learned Memory：...（tactic/pattern read-back）
Circuit Breaker：none
最近记忆：...（recent session events）
Peer Sessions：...（other active windows）
Workflow 标签 [status]:（从 workflow.md 解析的面包屑文本）
<dijiang-agent name="xxx" summary="xxx" />        ← compact，仅 name+summary
活跃任务：...（id/title/status/task_path/guidance）
Route Gate：...（capsule/default_skill/recommended_path）
Git Gate：...（state/branch/worktree）
Skill Manifests：dj-grill(需求对齐)，dj-implement(代码实现)  ← 无 risk 信息
Target Skill：[dj-grill（需求对齐）] capsule=align；recommended_path=...  ← 仅摘要
加载上下文：读取 task.json；...
</dijiang-workflow-state>
```

**关系**：
- Workflow 是 **Skill 的路由框架**：Route Gate 根据胶囊（align/implement/check/finish）决定允许/阻止哪些 skill
- Workflow 是 **Hook 的数据源**：`workflow.md` 中的 `[workflow-state:STATUS]` 标签被 hook 读取并注入会话
- Workflow 是 **CLI 的输出**：`dijiang workflow-state` 命令是 workflow 状态数据的主要出口

**Trellis 对比**：
- Trellis workflow.md 使用 `[workflow-state:STATUS]...[/workflow-state:STATUS]` 标签块直接嵌入流程指令
- Trellis 的 `inject-workflow-state.py` 直接从 workflow.md 解析标签，无需 CLI 中介
- Trellis 没有复杂的 Route Gate / Git Gate / Skill Manifest 系统——所有流动编码在标签文本中
- Trellis 没有 `WorkflowState` 编译时结构体——状态蕴含在 workflow.md 的标签系统中
- DiJiang 优势：Route Gate capsule 实现精细的 skill 准入控制，Skill Manifest 让 AI 了解全部可用能力

---

## 三、关系网络总览

### 数据流

```
    dijiang init / update
           │
           ▼
    ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
    │  Skill 文件   │     │  Agent 文件   │     │  Hook 文件    │
    │ .pi/skills/  │     │ .pi/agents/  │     │ .pi/hooks/   │
    └──────┬───────┘     └──────┬───────┘     └──────┬───────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────────────────────────────────────────────────┐
    │               Platform Runtime（各平台）              │
    │  Hook 每轮调用：dijiang workflow-state --json         │
    └──────────────────────────┬───────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  WorkflowState      │
                    │（CLI 编译时构建）   │
                    │  skill_manifest.rs  │
                    │  agent_manifest.rs  │
                    │  route_gate.rs      │
                    │  git_gate.rs        │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │  AI Agent 会话      │
                    │  ← 读取注入上下文   │
                    │  → 调用 CLI 推进任务 │
                    │  → channel spawn    │
                    └─────────────────────┘
```

### 依赖关系矩阵

| 组件 | 依赖 | 被依赖 | 跨项目同步 |
|------|------|--------|-----------|
| **Skill** | `skill_manifest.rs`（编译时）、Route Gate | Skill 是 AI 的执行标准 | `dijiang update` 从模板部署 |
| **CLI** | `dijiang-task`、`dijiang-mem`、`dijiang-configurator` crate | Hook、Agent、WorkflowState、Skill 部署 | Rust 编译，无跨项目更新机制 |
| **Agent** | `agent_manifest.rs`（编译时）、Agent 文件 | Channel spawn 时的子进程使用 | `dijiang init`/`update` 从模板部署 |
| **Hook** | 各平台目录中的脚本文件 | 每轮注入 WorkflowState | `dijiang init`/`update` 从模板部署 |
| **Workflow** | `workflow.md`（静态）、`WorkflowState`（运行时） | Hook 从中读取标签；Route Gate 从中确定胶囊 | `dijiang update` 更新 |

---

## 四、优化前后对比

### 上下文注入 token 节省估算

| 注入块 | 优化前 | 优化后 | 节省 |
|--------|--------|--------|------|
| `<dijiang-agent>` | 完整 agent body（≈300-700 tokens） | `name + summary`（≈10 tokens） | **~30-70x** |
| `<dijiang-target-skill>` | 完整 skill body（≈100-800 tokens） | 摘要行（≈20 tokens） | **~5-40x** |
| `Skill Manifests` | 含 `risk=` 信息 | 无 risk | 边际 |
| **总计** | **~500-1600 tokens/轮** | **~50 tokens/轮** | **~10-30x 降低** |

### 收益

- 减少每轮上下文开销，AI 注意力集中在 task info + route gate 上
- 知识冗余消除：target skill 和 agent 完整内容已在 skill 定义和 agent 文件中，不需要每轮重复注入
- 与 Trellis 对齐：Trellis 也只注入最小面包屑，不注入 skill/agent 全文

---

## 五、Trellis vs DiJiang 对比总表

| 维度 | Trellis | DiJiang | 差异分析 |
|------|---------|---------|---------|
| **Skill** | 平台目录分发，无编译时 manifest | 编译时 `skill_manifest.rs` + 路由门控 | DiJiang 更结构化，有准入控制 |
| **Skill 命名** | `trellis-{name}` | `dj-{name}` | 对称但不同命名空间 |
| **Agent** | 3 个（implement/check/research） | 5 个（+architect/planner） | DiJiang 覆盖更广 |
| **Agent 注入** | 不注入 workflow-state | Compact `name+summary` | 优化后一致 |
| **CLI** | npm + Python 脚本层 | 纯 Rust 二进制 | Trellis 有脚本层优势（AI 直接调用） |
| **Hook** | 直接解析 workflow.md 标签 | 调用 `dijiang workflow-state --json` | Trellis 免 CLI 依赖 |
| **Workflow 状态** | `workflow.md` 标签蕴含 | 编译时 `WorkflowState` 结构体 | DiJiang 更可编程，Trellis 更简洁 |
| **Route Gate** | 无 capsule 概念 | `route_gate.rs` 4 capsule | DiJiang 特有，Skill 准入控制 |
| **Git Gate** | 无 | `git_gate.rs` | DiJiang 特有，git 状态提示 |
| **注入内容** | 仅 `<workflow-state>` 面包屑 | WorkflowState + Manifests + Agent compact | DiJiang 更丰富但控制得当 |
| **上下文节省** | 天然最小（注入标签文本） | 优化后接近最小 | 一致 |

---

## 六、演化方向（未实现）

### 6.0 核心问题：Rust CLI 还有必要吗？

DiJiang 的所有日常操作（task CRUD、dispatch、workflow-state、status、skills）都是 I/O 操作：
读 JSON、写 JSON、调 git、调 subprocess。Rust 在这些场景上的性能优势是微秒级的，但维护成本（编译时间、二进制分发、PATH 依赖）是持续累积的。

按照 Trellis 的分层方式，Python 脚本可以承担大部分日常操作：

```
┌─────────────────────────────────────────────────────┐
│               Rust CLI（dijiang）                    │
│                                                      │
│  dijiang init         项目创建（一次）              │
│  dijiang update       模板同步（偶尔）              │
│  dijiang channel      子进程编排（需要 IPC 控制）   │
│  dijiang finish-work  merge/commit（重型 git 操作） │
│  dijang mem evolve    memory finetune（CPU 密集）   │
└────────────────────┬────────────────────────────────┘
                     │ 只保留需要 Rust 优势的部分
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌──────────────────┐  ┌──────────────────────────┐
│  Python 脚本层    │  │  Hook 层                  │
│                   │  │                           │
│ task.py           │  │ inject-workflow-state.py  │
│ dispatch.py       │  │ session-start.py          │
│ workflow_state.py │  │                           │
│ status.py         │  │ 直接 import common/*      │
│ skills.py         │  │ 不依赖 CLI                │
│ memory.py         │  │                           │
│                   │  │ .dijiang/scripts/         │
│ .dijiang/hooks/   │  │                           │
└──────────────────┘  └──────────────────────────┘
```

#### 分层原则

| 层 | 保留 Rust 的理由 | 对应的 Trellis |
|----|-----------------|----------------|
| `dijiang init` | 项目引导、模板解压、git clone 后处理 | `trellis init`（npm） |
| `dijiang channel` | subprocess 生命周期管理、IPC、并行执行控制 | 无（Trellis 无 channel 系统） |
| `dijiang finish-work` | merge conflict 检测、复杂的 git 状态机 | 无独立命令（Python 脚本做） |
| `dijiang mem evolve` | 大量 JSON 文件聚合、Thompson sampling | 无 |
| **其余的命令** | **不需要 Rust** | Python 脚本 |

#### 可以移到 Python 的清单

| 当前 CLI 命令 | 对应脚本 | 文件大小估算 |
|--------------|---------|------------|
| `dijiang status` | `scripts/status.py` | ~30 lines |
| `dijiang task current/create/archive` | `scripts/task.py` | ~80 lines |
| `dijiang dispatch` | `scripts/dispatch.py` | ~60 lines |
| `dijiang workflow-state --json` | `scripts/workflow_state.py` | ~100 lines |
| `dijiang skills` | `scripts/skills.py` | ~40 lines |
| `dijiang mem findings/learn/recall` | `scripts/memory.py` | ~60 lines |

#### Rust CLI 不会消失

缩小范围后，Rust CLI 的存在理由更清晰：

1. **`dijiang init`** — 唯一的项目创建入口。没有它就没有 `.dijiang/`。类似 Trellis 的 `npx trellis init`。
2. **`dijiang channel spawn`** — 多 agent 编排核心。Python 的 subprocess 管理不如 Rust `Command` + 信号处理可靠，尤其管理多个并行 worker 时。
3. **`dijiang finish-work`** — commit/merge/push 流程涉及 git 操作链。Rust 的 `git_gate.rs` 状态机比 Python 脚本更健壮。
4. **`.dijiang/scripts/` 本身由 Rust 部署** — `dijiang init` 从模板生成脚本文件，`dijiang update` 同步更新。

**等价于**：Rust CLI = 基础设施部署器 + 重型操作引擎；Python 脚本 = 每日操作界面。

#### 分阶段实施路径

| 阶段 | 内容 | 动机 |
|------|------|------|
| **短期** | 文档记录分层设计，不动代码 | Context 注入优化已降低 CLI 每轮调用开销 |
| **中期** | Hook 层去 CLI 依赖（hook 直接解析 workflow.md 标签） | 最大的痛点：hook 失败 = 零上下文 |
| **长期** | task CRUD 和 dispatch 脚本化 | 如果 hook 去 CLI 依赖成功，再扩展 |

---

### 6.1 Python 辅助脚本层（借鉴 Trellis）

```
.dijiang/scripts/
├── task.py           # 任务 CRUD
├── dispatch.py       # 请求分类 + skill 路由
├── workflow_state.py # 上下文注入数据
├── status.py         # 项目状态查询
├── skills.py         # 技能枚举
├── memory.py         # 记忆查询与记录
└── common/           # 公共模块（路径解析、git 封装、JSON 工具）
```

**价值**：
- AI 可以 `python3 .dijiang/scripts/task.py current` 直接操作，无需 `dijiang` 在 `$PATH`
- 脚本从 `.dijiang/` 读取配置，不依赖环境变量
- 降低 AI 调用基础设施的心理门槛
- 避免 Rust 编译失败导致的停摆

---

### 6.2 Hook 直接解析 workflow.md（借鉴 Trellis）

当前 hook 通过 `dijiang workflow-state --json` 获取状态。这要求 CLI 在 PATH 中。
如果 CLI 不可用，hook 失败，每轮零上下文注入。

```python
# inject-workflow-state.py 直接解析 workflow.md 标签
content = Path(".dijiang/workflow.md").read_text()
tags = parse_tags(content)
breadcrumb = build_breadcrumb(tags, active_task)

# 任务运行时状态从 .dijiang/tasks/<task>/task.json 读取
# Git 状态从 git CLI 获取
# 不需要 dijiang 二进制介入
```

**价值**：
- 消除对 CLI 二进制的依赖——hook 永远不会因为 Rust 编译问题而失败
- hook 更轻量：纯 Python 文件操作，无 subprocess 调用到自身
- 与 Trellis 的设计一致

---

### 6.3 Session-scoped active task（借鉴 Trellis）

当前 DiJiang 使用 `DIJIANG_CONTEXT_ID` 环境变量 + workspace session 文件。Trellis 的 `active_task.py` 实现了多平台 session 身份解析（自动检测 Claude/Cursor/Codex/Gemini 等 15+ 平台）。

**价值**：
- 支持多窗口并行开发
- 跨平台 session 身份自动识别
- 更 robust 的活跃任务指针管理
