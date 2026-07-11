# Trellis 架构深度分析与吸收建议

> 日期：2026-07-11
> 目标：基于对 Trellis 完整源代码的深度分析，识别核心架构优势，为 DiJiang 演进提供优先级有序的吸收路线图

参考源：`/Users/cimer/Project/DiJiang/referens/Trellis`
Trellis CLI：`@mindfoldhq/trellis`（npm），SDK：`@mindfoldhq/trellis-core`

---

## 一、Trellis 总体架构

```
┌─────────────────────────────────────────────────────────┐
│  平台层 (Platform Adaptors)                              │
│  Claude Code · Codex CLI · Cursor · OpenCode · Pi · ... │
└────────────────────────┬───────────────────────────────┘
                         │  injected context + skill dispatch
┌────────────────────────▼───────────────────────────────┐
│  技能层 (Bundled Skills)                                 │
│  trellis-start · trellis-brainstorm · trellis-before-   │
│  dev · trellis-implement · trellis-check · trellis-     │
│  finish-work · trellis-continue · trellis-session-      │
│  insight · trellis-channel · trellis-break-loop · ...   │
└────────────────────────┬───────────────────────────────┘
                         │  Python scripts + CLI
┌────────────────────────▼───────────────────────────────┐
│  脚本层 (Python Scripts)                                 │
│  task.py · get_context.py · add_session.py               │
│  init_developer.py · safe_commit.py                      │
│  common/: types.py · tasks.py · task_store.py            │
│  active_task.py · paths.py · io.py · config.py · git.py │
└────────────────────────┬───────────────────────────────┘
                         │  file system
┌────────────────────────▼───────────────────────────────┐
│  状态层 (.trellis/)                                      │
│  config.yaml · workflow.md · tasks/ · spec/ · workspace/ │
│  agents/ · .runtime/ · scripts/                          │
└─────────────────────────────────────────────────────────┘
```

### 架构特征

- **CLI 轻量，脚本重**：npm CLI 只做 init、update、channel 等基础设施操作；日常任务管理委托给 Python 脚本
- **平台无关 skill**：skill 定义是纯 markdown 文件，自动派发到所有支持的平台目录
- **workflow.md 即状态机**：内嵌 `[workflow-state:STATUS]` 面包屑，hook 逐轮解析
- **channel 运行时**：多代理通过 JSONL 事件日志协调，职责严格分离
- **会话作用域任务指针**：活跃任务绑定到 AI session ID，支持多窗口并行

---

## 二、关键亮点分析（按吸收优先级排列）

### [P0] 1. Python 脚本辅助层

Trellis 核心设计之一是 `.trellis/scripts/` 中的 Python 脚本。它们直接从 `.trellis/` 读取配置，不依赖 CLI 可执行文件路径。

**核心脚本清单**：

| 脚本 | 职责 |
|--------|----------|
| `task.py` | 任务 CRUD：create、start、archive、current、fix、add-subtask |
| `get_context.py` | 上下文摘要输出：git、task、spec、workflow、agent 定义 |
| `add_session.py` | 写入开发者日志到 workspace journal |
| `init_developer.py` | 首次运行时设置开发者身份 |
| `safe_commit.py` | 分阶段 git add + commit（journal → archive → spec） |
| `common/paths.py` | 路径工具：向上搜索 `.trellis/` 找仓库根 |
| `common/tasks.py` | 数据访问：load_task、iter_active_tasks |
| `common/task_store.py` | task CRUD 实现 + 生命周期钩子 |
| `common/active_task.py` | 会话作用域活跃任务指针（多平台 session ID） |
| `common/types.py` | TaskData TypedDict / TaskInfo dataclass |
| `common/io.py` | JSON 读写、目录创建 |
| `common/config.py` | `.trellis/config.yaml` 加载 |
| `common/git.py` | git 操作抽象 |

**设计原则**：
- 每个脚本是独立的 CLI 入口，AI 代理可 `python3 .trellis/scripts/task.py current` 直接调用
- 公共模块放在 `common/`，多脚本复用
- 从 `.trellis/` 读取配置而非环境变量，减少外部依赖

**DiJiang 差距**：所有操作通过 `dijiang` Rust CLI，AI 代理需要 `$PATH` 中有可执行文件。缺少辅助脚本层。

### [P0] 2. `[workflow-state:...]` 面包屑机制

workflow.md 内嵌 `[workflow-state:no_task]` / `[workflow-state:planning]` / `[workflow-state:in_progress]` 等封闭块。平台 hook 脚本逐轮解析这些标签，注入 AI 会话作为当前阶段指示。

```
[workflow-state:planning]
Load `trellis-brainstorm`; stay in planning.
Lightweight: `prd.md` can be enough. Complex: finish `prd.md`, `design.md`, and `implement.md`.
[/workflow-state:planning]

[workflow-state:in_progress]
Task is active — code in a worktree.
Before writing code: always load .trellis/spec/ relevant to the task.
Remember: spec-before-edit.
[/workflow-state:in_progress]
```

**关键设计点**：
- workflow.md 同时是人可读的文档和机器可解析的状态机
- 没有单独的 JSON 状态文件 —— 状态蕴含在 workflow.md 的标签中
- hook 脚本在每轮交互前解析当前应注入的 state block

**DiJiang 现状**：`dijiang workflow-state --json` 输出 JSON 格式的 workflow 状态，但这是 CLI 输出逻辑，workflow.md 本身没有状态标签。

### [P1] 3. 会话作用域活跃任务 + 多平台身份解析

`active_task.py` 实现了 AI session 级别的活跃任务指针，支持在 15+ 平台上正确识别会话身份。

**身份解析优先级**：
1. `TRELLIS_CONTEXT_ID` 环境变量（显式覆盖，由子进程设置）
2. Platform input hook 的 payload
3. 平台环境变量（`CLAUDE_SESSION_ID`、`PI_SESSION_ID`、`CURSOR_SESSION_ID`、`CODEX_SESSION_ID`、`OPENCODE_SESSION_ID` 等）
4. Cursor shell ticket 文件
5. 退化模式：无 session ID 时工作在 `.trellis/.runtime/sessions/global.json`

**核心数据结构**（`.trellis/.runtime/sessions/<context-key>.json`）：
```json
{
  "active_task": "refactor-auth",
  "started_at": "2025-06-15T10:30:00Z",
  "session_id": "claude_abc123"
}
```

**DiJiang 差距**：`DIJIANG_CONTEXT_ID` 环境变量已存在，但缺少类似 `active_task.py` 的通用会话身份解析模块。当前 session 文件在 `.dijiang/workspace/` 中，但缺少 Cursor shell ticket 等平台适配。

### [P1] 4. JSONL 上下文清单

每个任务目录创建时生成一个空 JSONL 文件（如 `implement.jsonl`），由 AI 在规划阶段填写 spec / research 文件引用。子代理（implement / check）通过此清单读取精确上下文。

**Trellis 的种子内容**：
```jsonl
{"file": ".trellis/spec/cli/backend/error-handling.md", "reason": "Error handling conventions for this task"}
```

**检查机制**：`task.py start` 命令要求 `implement.jsonl` 中至少有一条有效记录。若为空，提示用户先在 brainstorm 中填入。

**作用**：
- 子代理不遍历整个 spec 目录，只加载需要的文件
- 体现 planning gate：start 前必须完成上下文规划
- 记录的 reason 字段帮助子代理理解"为什么需要这个文件"

**DiJiang 现状**：任务目录中没有上下文清单。子代理上下文依赖 skill 定义，无法按任务精确控制。

### [P1] 5. 任务层级（parent / child）

Trellis 支持父子任务树，`task.py create --parent <dir>` 创建子任务。

**数据结构**：
```json
{
  "id": "refactor-auth",
  "parent": null,
  "children": ["add-oauth", "migrate-tokens", "update-middleware"],
  "package": "auth",
  "scope": "backend",
  "branch": "refactor/refactor-auth",
  "base_branch": "main",
  "priority": "high",
  "assignee": "dev1",
  "progress": {"completed": 2, "total": 5}
}
```

**进度展示**：`children_progress()` 输出 `[2/5 done]`。

**DiJiang 现状**：平面任务目录，无层级关系。

### [P2] 6. 可互换的工作流模板

`trellis init --workflow <template>` 支持多种 workflow：
- `native`：默认三阶段（Plan / Execute / Finish）
- `tdd`：测试驱动开发流程
- `channel-driven-subagent-dispatch`：多代理编排

**模板结构**：存储在 CLI 包中，每个模板是一组文件（workflow.md、scripts/、config.yaml），通过 `templates/<template>/` 目录组织。

### [P2] 7. Agent 定义文件

`.trellis/agents/implement.md` 和 `.trellis/agents/check.md` 是平台无关的 agent 定义。channel 运行时直接加载这些文件。

**格式**：YAML frontmatter + markdown 职责描述

```yaml
---
name: implement
description: Implement features per spec without git operations or testing.
mode: implement
labels: [trellis, implement]
instructions: |
  1. Create or edit code files only — no git operations.
  2. Run `python3 .trellis/scripts/add_session.py ...` after each session.
  3. Update implement.jsonl with files you read.
---
```

### [P2] 8. 注册表 spec 同步

`config.yaml` 支持 registry 来源：
```yaml
registry:
  spec:
    source: "https://example.com/trellis-specs"
    template: "ts-backend"
```

`trellis update` 从 registry 拉取更新。`template-hashes.json` 跟踪文件 hash，检测本地修改冲突。

### [P3] 9. 跨会话记忆（原始日志检索）

`trellis mem search <keyword> --phase brainstorm` 直接索引平台对话日志：
- `~/.claude/projects/<project>/conversation*.jsonl`
- `~/.codex/sessions/<project>-<session>-*.jsonl`
- `~/.pi/agent/sessions/<session>/output.jsonl`

**输出风格**：`mem` 的输出是原料（raw material），不是交付物。写入什么文件由 AI 根据现场判断决定。

### [P3] 10. 安全提交模式

`safe_commit.py` 包含 `.gitignore` 检测、`session_auto_commit` 开关、分阶段 add（journal → archive → spec）。

---

## 三、DiJiang 与 Trellis 架构对比

| 维度 | Trellis | DiJiang | 差距 |
|--------|---------|---------|------|
| **CLI** | npm CLI + Python 脚本双栈 | Rust CLI 单一入口 | 缺少辅助脚本层 |
| **Workflow** | 多模板可选（native/tdd/channel） | 单一 workflow | 缺少模板选择 |
| **任务模型** | 支持 parent/child 层级 | 平面目录 | 缺少层级 |
| **Session 身份** | 15+ 平台 + Cursor ticket | DIJIANG_CONTEXT_ID 环境变量 | 缺多平台和 Cursor 支持 |
| **上下文注入** | JSONL 清单 + workflow-state 面包屑 | skill 定义 + workflow-state | 缺 JSONL 清单 |
| **Agent 定义** | `.trellis/agents/*.md` | skill 文件 | 缺独立 agent 定义 |
| **跨会话记忆** | 读取原始平台日志 | internal memory | 缺平台日志检索 |
| **Spec 同步** | registry source + hash 冲突检测 | 静态 spec | 缺同步机制 |
| **多代理** | channel runtime + agent 定义 | dijiang channel | agent 定义待增强 |
| **提交控制** | session_auto_commit 可配置 | 全自动 | 缺半自动模式 |

---

## 四、吸收路线图

### P0（短期，当前任务范围）

1. **创建 `.dijiang/scripts/` Python 辅助脚本层**
   - `task.py` — 任务 CRUD 的 Python 封装
   - `get_context.py` — 上下文摘要输出
   - `add_session.py` — 日志记录
   - `common/` — 路径、类型、配置、git 工具模块

2. **workflow-state 面包屑**
   - 在 `.dijiang/workflow.md` 中嵌入 `[workflow-state:...]` 标签块
   - 更新 hook 脚本解析这些标签并注入 AI 会话

### P1（后续里程碑）

3. **JSONL 上下文清单**
   - 任务创建时生成空的 `context.jsonl`
   - task start 前检查非空
   - 子代理根据清单加载上下文

4. **任务层级**
   - task.json 增加 parent / children 字段
   - CLI 支持 `--parent` 参数和 `add-subtask` 子命令

5. **会话身份解析增强**
   - 增加 Cursor shell ticket 支持
   - 增加退化模式 global fallback

### P2（长期）

6. **Workflow 模板选择**
   - 支持 `dijiang init --workflow tdd`

7. **Agent 定义文件**
   - 创建 `.dijiang/agents/implement.md`、`check.md`

8. **Spec 注册表同步**
   - registry source + hash 冲突检测

### P3（远景）

9. **跨会话记忆增强**
   - 从原始平台日志检索

10. **安全提交模式**
   - `session_auto_commit` 配置开关
   - 分阶段 add

---

## 五、Python 脚本层设计（P0 实施方向）

### 目录结构

```
.dijiang/scripts/
├── __init__.py          # 空，使 common/ 可导入
├── task.py              # 任务 CRUD CLI 入口
├── get_context.py       # 上下文摘要输出
├── add_session.py       # 开发者日志记录
├── init_developer.py    # 开发者身份初始化
├── safe_commit.py       # git 提交辅助
└── common/
    ├── __init__.py      # 公共 API 聚合
    ├── paths.py         # 仓库根路径、tasks 路径、spec 路径
    ├── types.py         # TaskData TypedDict, TaskInfo dataclass
    ├── tasks.py         # load_task, iter_active_tasks
    ├── task_store.py    # create, start, archive, current
    ├── active_task.py   # 会话作用域活跃任务指针
    ├── io.py            # JSON 读写、目录创建
    ├── config.py        # .dijiang/config.yaml 加载
    └── git.py           # git 操作封装
```

### 核心原则

- **从 `.dijiang/` 发现仓库根**：`paths.py` 向上搜索 `.dijiang/` 目录，不依赖环境变量
- **可独立调用**：每个脚本是独立的 `python3 .dijiang/scripts/task.py current` 入口
- **兼容 CLI 双栈**：与 `dijiang` Rust CLI 共享 `.dijiang/` 状态目录，互不冲突
- **Session 身份桥接**：`active_task.py` 解析 `PI_CONTEXT_ID` / `DIJIANG_CONTEXT_ID` / 平台环境变量
- **纯 stdlib**：不引入外部 Python 依赖，AI 环境开箱可用

### API 原型

```python
# common/types.py
class TaskData(TypedDict, total=False):
    id: str
    title: str
    status: str  # planning | in_progress | completed | archived
    parent: NotRequired[str | None]
    children: NotRequired[list[str]]
    package: NotRequired[str]
    scope: NotRequired[str]
    branch: NotRequired[str]
    priority: NotRequired[str]
    assignee: NotRequired[str]

class TaskInfo(NamedTuple):
    """Immutable public view of a task."""
    id: str
    title: str
    status: str
    path: str
```

```python
# 使用示例
from common.paths import get_tasks_dir, get_repo_root
from common.tasks import load_task, iter_active_tasks
from common.task_store import create_task, start_task, archive_task
from common.active_task import get_active_task, set_active_task
from common.config import load_config
from common.io import read_json, write_json
from common.git import get_current_branch, is_dirty
```

---

## 六、风险与缓解

| 风险 | 缓解 |
|------|----------|
| 双栈（Rust CLI + Python scripts）路径冲突 | 共享 `.dijiang/` 文件系统，通过 lock 文件协调写入 |
| Python 版本 / 依赖不可用 | 脚本只使用 stdlib，不引入外部依赖 |
| Session 身份解析竞态（多个 AI 窗口写同一文件） | 会话文件基于 session_id 隔离，不共享写路径 |
| workflow-state 标签与 task.json 状态不一致 | 以 task.json 为事实源，workflow-state 标签是提示级快照 |

---

## 附录：Trellis 关键文件索引

| 文件 | 内容 |
|------|------------------|
| `packages/cli/src/templates/trellis/scripts/task.py` | 任务 CRUD CLI |
| `packages/cli/src/templates/trellis/scripts/common/task_store.py` | 任务存储实现 |
| `packages/cli/src/templates/trellis/scripts/common/active_task.py` | 会话作用域活跃任务 |
| `packages/cli/src/templates/trellis/scripts/common/types.py` | 类型定义 |
| `packages/cli/src/templates/trellis/scripts/common/paths.py` | 路径工具 |
| `.agents/skills/trellis-start/SKILL.md` | 启动 skill |
| `.agents/skills/trellis-brainstorm/SKILL.md` | 需求对齐 skill |
| `.agents/skills/trellis-check/SKILL.md` | 质量检查 skill |
| `.agents/skills/trellis-before-dev/SKILL.md` | 开发前上下文加载 skill |
| `.agents/skills/trellis-spec-bootstrap/SKILL.md` | spec bootstrap skill |
| `.agents/skills/trellis-channel/SKILL.md` | 多代理 channel skill |
| `.trellis/workflow.md` | 完整 workflow 定义 |
