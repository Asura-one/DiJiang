---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '5f4030c5-7f4d-4229-ad7a-d62f8b8f5836'
  PropagateID: '5f4030c5-7f4d-4229-ad7a-d62f8b8f5836'
  ReservedCode1: '51b27821-3039-4388-90f7-f87b37779c2b'
  ReservedCode2: '51b27821-3039-4388-90f7-f87b37779c2b'
---

# DiJiang 架构

## 概述

DiJiang 是一个**纯 skill 架构**的工程工作流框架：没有编译、没有运行时、不绑定特定 agent。全部能力以 `SKILL.md` 文本形式存在，由任何支持 skill 机制的 agent（Claude、Codex、OpenCode 等）直接加载执行。

架构从 Rust CLI 运行时迁移而来，迁移决策见 **ADR-0005**（从 Trellis 兼容迁移到纯 Skill 架构）；回归纪律的引入见 **ADR-0006**（Regression Guard 与 Fullstack Testing）。

## 核心设计原则

1. **无编译、无运行时**：项目不含任何可执行代码，工作流逻辑全部以文本纪律承载。
2. **不绑定特定 agent**：无平台适配层，任何支持 skill 的 agent 均可消费。
3. **工作流分散在 skill 中**：路由、门禁、收尾逻辑不再集中于 CLI，而是分布在各个 skill 文本。
4. **纪律靠文本引导**：原运行时硬约束（Route/Git/Capability Gate）下沉为 skill 文本纪律，依赖模型遵守（见 ADR-005）。
5. **可组合、可改造**：skill 是小粒度原子单元，支持按需新增、拆分、重组。

## Skill 组织结构

```
skills/
├── engineering/    # 41 个工程 skill（代码工作相关）
│   ├── dj-ask/SKILL.md            # 全局 flow 路由图（战略层）
│   ├── dj-dispatch/SKILL.md       # 逐请求战术路由器
│   ├── dj-implement/SKILL.md      # 功能实现
│   ├── dj-tdd/SKILL.md            # 测试驱动开发
│   ├── dj-regression-guard/SKILL.md   # 改码三明治回归协议
│   ├── dj-fullstack-testing/SKILL.md  # 全量回归引擎
│   ├── dj-check/SKILL.md          # 交付质量门禁
│   ├── dj-hunt/SKILL.md           # Bug 排查
│   └── ...（共 41 个）
└── productivity/   # 5 个 skill：通用工作流工具（文字润色等）
```

### SKILL.md 文件结构

每个 skill 是 `skills/<bucket>/<name>/SKILL.md`，采用 mattpocock/skills 风格（见 ADR-005）：

- **frontmatter**：`name`（= 目录名）、`description`（含触发词）、可选 `disable-model-invocation`。
- **正文**：工作流步骤、纪律、铁律，以文本形式引导 agent 行为。

### User-invoked 与 Model-invoked

- **User-invoked skill**（`disable-model-invocation: true`）：只能由用户显式调用，承担编排与入口职责（如 `dj-setup`、`dj-grill`、`dj-handoff`）。
- **Model-invoked skill**：可由模型在满足触发条件时自动调用，承载可复用工程纪律（如 `dj-implement`、`dj-regression-guard`、`dj-memory`）。

约束：User-invoked skill 可调用 model-invoked skill，但永远不能调用另一个 user-invoked skill。

## 加载机制

1. **skill 即文件**：agent 通过 skill 加载工具读取 `SKILL.md`，无注册表、无编译清单。
2. **自动触发**：模型根据 `description` 中的触发条件自动匹配 model-invoked skill；`disable-model-invocation: true` 阻止自动触发。
3. **按需引用**：skill 之间通过"引用不复制"协作，跨技能细节引用 `docs/references/` 规范，避免重复文本。

## 工作流路由

路由分两层，互为补充：

| 层 | Skill | 职责 |
|----|-------|------|
| 战略层 | `dj-ask` | 全局 flow 路由图（主流程 / 入口匝道 / 独立工具 / 词汇层） |
| 战术层 | `dj-dispatch` | 逐请求分类路由：识别任务类型 → 路由到对应 skill（支持混合任务串联） |

任务状态机：`none → planning → in_progress → completed → archived`，另有 `paused`。

各阶段对应的核心 skill：

- **planning**：`dj-grill`（需求对齐）、`dj-output`（文档）、`dj-spec-bootstrap`、`dj-split`
- **in_progress**：`dj-implement` / `dj-tdd` / `dj-hunt`，改码过程由 **`dj-regression-guard`** 护送（改前基线 → 修改 → 专项验证 → 改后回归），全量回归由 **`dj-fullstack-testing`** 承载（见 ADR-006）
- **completed**：`dijiang-finish-work` 收尾；`dj-check` 为交付前质量门禁

## 安全约束（三层 Gate 下沉）

原运行时三层门禁已下沉为 skill 文本纪律（见 ADR-005）：

| 原门禁 | 现形态 |
|--------|--------|
| **Route Gate** | 任务状态约束分散在状态机与 `dj-dispatch`/`dj-ask` 的路由文本中 |
| **Git Gate** | `dj-git-guardrails` + AGENTS.md 的 Worktree-First 纪律：主目录保持干净、功能在独立 worktree 中开发、合并需用户确认 |
| **Capability Gate** | 破坏性操作（reset --hard / force push / clean -f）在 skill 文本中明确禁止，收尾执行需用户确认 |

## 项目目录结构

```
DiJiang/
├── skills/
│   ├── engineering/       # 41 个 skill
│   └── productivity/      # 5 个 skill
├── docs/
│   ├── adr/               # 架构决策记录（001-006）
│   ├── references/        # 跨技能参考文档
│   └── guide/             # 使用指南
├── CONTEXT.md             # 领域术语表（skill/bucket/task 等）
├── AGENTS.md              # Agent 路由索引（最小路由，非 workflow 定义）
├── CLAUDE.md              # Claude 项目上下文
├── CHANGELOG.md           # 变更日志
├── README.md              # 项目入口
└── .gitignore
```

> `.dijiang/`（tasks/ + memory/ + spec/ + config.toml）是项目本地状态目录，gitignored，由 `dj-setup` 初始化，不进入仓库。

## 关键设计决策

- **纯 skill 无运行时**（ADR-005）：移除全部 Rust 代码、Trellis 兼容层与平台适配层。
- **门禁下沉为文本纪律**：安全与工作流约束依赖 skill 文本与模型遵守，换取零依赖与全 agent 兼容。
- **回归纪律独立成 skill**（ADR-006）：`dj-regression-guard` 是任务级横切协议，被 `dj-implement`/`dj-hunt`/`dj-tdd`/`dj-check` 统一引用，回归真相单一。
- **引文不复制**：跨 skill 共享机制集中为 `docs/references/` 文档，skill 内仅引用。
- **CONTEXT.md 承载领域语言**，ADR 记录决策演化，AGENTS.md 只做最小路由索引。

## 相关文档

- `docs/adr/005-migration-from-trellis-to-pure-skills.md` — 纯 skill 架构迁移决策
- `docs/adr/006-regression-guard-fullstack-testing.md` — 回归守卫与全量回归引入
- `CONTEXT.md` — 领域术语表
- `AGENTS.md` — Skill 路由索引与调用方式