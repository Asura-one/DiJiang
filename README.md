---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '9d69ecbf-2f95-4703-8c8d-ab04575dc620'
  PropagateID: '9d69ecbf-2f95-4703-8c8d-ab04575dc620'
  ReservedCode1: '57bb34c3-abd1-4e67-b243-66e9efbf3249'
  ReservedCode2: '57bb34c3-abd1-4e67-b243-66e9efbf3249'
---

# 帝江 (DiJiang)

> 浑敦无面目，是识歌舞。——《山海经·西山经》

融合 [mattpocock/skills](https://github.com/mattpocock/skills)、[ponytail](https://github.com/DietrichGebert/ponytail)、[Waza](https://github.com/tw93/Waza) 三大 skill 库的工程工作流。

## 定位

DiJiang 是一套纯 skill 集合，提供 AI 辅助开发的工作流能力。无需编译，无需运行时，不绑定特定 agent。

- **`dj-*` skills**：提供需求对齐、实现、排查、质量检查、审计、文档等原子工作能力。
- **`dijiang-*` skills**：提供 session 管理（启动、继续、收尾）。
- **`dj-memory`**：项目记忆管理，其他 skill 可调用。
- **`dj-setup`**：项目初始化和旧版迁移。

## 安装

```bash
# 1. 复制 skills/ 目录到你的项目
cp -r skills/ /path/to/your-project/

# 2. 在 agent 中运行 dj-setup 初始化
# Call the Skill tool with "dj-setup"
```

新项目会创建 `.dijiang/` 目录（tasks/ + memory/ + spec/ + config.toml）。

## Skill 清单

### Engineering（41 skills）

#### 核心流程

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-dispatch` | user | 新任务分流和技能路由 |
| `dj-ask` | user | 全局 flow 路由图（有哪些 workflow/该走什么流程） |
| `dj-grill` | user | 需求不清、范围需要对齐 |
| `dj-output` | user | PRD、design、spec 文档 |
| `dj-spec-bootstrap` | user | Spec 初始生成 |
| `dj-split` | user | PRD 拆分 |
| `dj-wayfinder` | user | 大块工作规划（决策票据地图） |
| `dj-triage` | user | Issue/任务分诊 |
| `dj-implement` | model | 特性代码实现 |
| `dj-tdd` | model | 测试驱动开发 |
| `dj-hunt` | model | bug、回归、根因排查 |
| `dj-regression-guard` | model | 改码三明治回归协议（改前基线→改后回归） |
| `dj-fullstack-testing` | model | 全栈回归测试引擎（全量模式） |
| `dj-merge-conflict` | model | 合并冲突解决 |
| `dj-check` | model | 代码审查、质量门禁 |
| `dj-review` | model | 轻量只读审查 |

#### 辅助能力

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-audit` | user | 全仓审计（含技术债/健康检查 profile） |
| `dj-pattern` | user | 模式研究 |
| `dj-reason` | model | 复杂判断、认知校准 |
| `dj-research` | model | 技术调研 |
| `dj-absorb` | user | 吸收外部材料 |
| `dj-domain-modeling` | model | 领域建模 |
| `dj-codebase-design` | model | 代码结构设计 |
| `dj-design` | user | UI/UX 设计实现 |
| `dj-prototype` | user | 原型验证 |
| `dj-remix` | user | 站点再造 |
| `dj-script` | user | 脚本编写 |
| `dj-read` | model | URL/PDF 阅读 |
| `dj-ponytail` | model | 极简纪律、YAGNI |
| `dj-wizard` | model | 人类步骤交互向导 |
| `dj-handoff` | user | 跨 session 交接 |
| `dj-gov` | user | 知识治理收尾 |
| `dj-git-guardrails` | model | Git 护栏 |
| `dj-channel` | user | 多 agent 通道 |
| `dj-meta` | user | 架构自省 |
| `dj-session-insight` | user | Session 洞察 |

#### Session 管理

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-setup` | user | 项目初始化或迁移 |
| `dj-memory` | model | 记忆管理（其他 skill 调用） |
| `dijiang-start` | user | 启动会话 |
| `dijiang-continue` | user | 继续会话 |
| `dijiang-finish-work` | user | 收尾工作 |

### Productivity（5 skills）

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-write` | model | 文字润色、去 AI 味 |
| `dj-teach` | user | 跨会话教学 |
| `dj-questionnaire` | user | 决策问卷生成 |
| `dj-wait-what` | user | 消息重述 |
| `dj-writing-for-agents` | model | 为 agent 写文档的规范 |

## Canonical Workflow

```
none
  └─ dj-setup（初始化）或 dj-dispatch（创建任务）
planning
  └─ dj-grill（对齐），必要时 dj-output → dj-split
in_progress
  ├─ dj-implement / dj-tdd / dj-hunt / dj-design / dj-script
  ├─ dj-regression-guard（改码三明治护送）
  └─ dj-check（质量门禁）
completed
  └─ dijiang-finish-work（验证 + 提交 + 归档）
archived
  └─ 只读；如需继续则重新创建任务
paused
  └─ dijiang-continue（恢复）
```

## 全局约束：Git 安全工作流（Worktree-First）

1. 主工作区永远干净，只做同步，严禁在主目录上直接写代码。
2. 每个功能一个独立 worktree，所有开发均在 worktree 中进行。
3. 合并需用户确认。
4. 回滚必须备份 + 确认。
5. 禁止自动执行破坏性操作。
6. 提交信息遵循 Conventional Commits，使用中文编写。

## 项目结构

```
DiJiang/
├── skills/
│   ├── engineering/       # 41 skills
│   │   ├── dj-grill/SKILL.md
│   │   ├── dj-implement/SKILL.md
│   │   ├── dj-tdd/SKILL.md
│   │   ├── ...
│   │   ├── dj-setup/SKILL.md       (初始化)
│   │   ├── dj-memory/SKILL.md      (记忆管理)
│   │   ├── dijiang-start/SKILL.md
│   │   ├── dijiang-continue/SKILL.md
│   │   └── dijiang-finish-work/SKILL.md
│   └── productivity/      # 5 skills
│       └── dj-write/SKILL.md
├── docs/
│   ├── adr/               # 架构决策记录
│   ├── references/        # 跨技能参考文档
│   └── guide/             # 使用指南
├── CONTEXT.md             # 领域术语表
├── AGENTS.md              # Agent 路由索引
├── CLAUDE.md              # Claude 项目上下文
├── CHANGELOG.md           # 变更日志
└── .gitignore
```

## 设计原则

1. **Predictability** — 每次运行走相同流程。
2. **YAGNI** — 不需要的不写，stdlib 能做的不引入依赖。
3. **Fail-safe** — 破坏性操作必须确认，回滚必须备份。
4. **Composable** — skill 之间可串联，也可单独使用。
5. **Runtime-neutral** — 不绑定特定 agent，纯 skill 无运行时依赖。

## 参考

- [mattpocock/skills](https://github.com/mattpocock/skills) — 纯 skill 设计思路来源
- [ponytail](https://github.com/DietrichGebert/ponytail) — 极简工程纪律
- [Waza](https://github.com/tw93/Waza) — skill 库参考