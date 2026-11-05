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
# 1. 复制 skills/ 整个目录到你的项目（保持内部结构——共读参考随技能走）
cp -r skills/ /path/to/your-project/

# 2. 在 agent 中运行 dj-setup 初始化
# Call the Skill tool with "dj-setup"
```

> 只装 skills 即完备：技能引用的纪律文档（反模式/证据门/决策阶梯等）在各技能 `references/` 内，随 `skills/` 整体迁移；`docs/` 是本仓库开发面文档（ADR/架构/发版），使用方无需复制。

新项目会创建 `.dijiang/` 目录（tasks/ + memory/ + spec/ + config.toml）。

## Skill 清单

### Engineering（37 skills）

#### 核心流程

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-ask` | user | 全局 flow 路由图 + 新任务分流（有哪些 workflow/该走什么/单请求路由） |
| `dj-grill` | user | 需求不清、范围需要对齐 |
| `dj-output` | user | PRD、design、spec 文档 |
| `dj-split` | user | PRD 拆分 / Issue 分诊（分诊档） |
| `dj-wayfinder` | user | 大块工作规划（决策票据地图） |
| `dj-implement` | model | 特性代码实现 |
| `dj-tdd` | model | 测试驱动开发 |
| `dj-hunt` | model | bug、回归、根因排查 |
| `dj-regression-guard` | model | 改码三明治回归协议（改前基线→改后回归） |
| `dj-fullstack-testing` | model | 全栈回归测试引擎（全量模式） |
| `dj-merge-conflict` | model | 合并冲突解决 |
| `dj-check` | model | 代码审查、质量门禁 |

#### 辅助能力

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-audit` | user | 全仓审计（含技术债/健康检查 profile） |
| `dj-reason` | model | 复杂判断、认知校准 |
| `dj-research` | model | 技术调研 |
| `dj-absorb` | user | 吸收外部材料 |
| `dj-loop` | user | 无人值守 loop 设计与评审（只设计不运行） |
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

#### Session 管理

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-setup` | user | 项目初始化或迁移 |
| `dj-memory` | model | 记忆管理（其他 skill 调用） |
| `dijiang-start` | user | 启动会话 |
| `dijiang-continue` | user | 继续会话 |
| `dijiang-finish-work` | user | 收尾工作 |

### Productivity（3 skills）

| Skill | 调用模式 | 触发 |
|-------|---------|------|
| `dj-write` | model | 文字润色、去 AI 味 |
| `dj-questionnaire` | user | 决策问卷生成 |
| `dj-wait-what` | user | 消息重述 |

## Canonical Workflow

```
none
  └─ dj-setup（初始化）或 dj-ask（新任务分流）
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
│   ├── engineering/       # 37 skills
│   │   ├── dj-grill/SKILL.md
│   │   ├── dj-implement/SKILL.md
│   │   ├── dj-tdd/SKILL.md
│   │   ├── ...
│   │   ├── dj-setup/SKILL.md       (初始化)
│   │   ├── dj-memory/SKILL.md      (记忆管理)
│   │   ├── dijiang-start/SKILL.md
│   │   ├── dijiang-continue/SKILL.md
│   │   └── dijiang-finish-work/SKILL.md
│   └── productivity/      # 3 skills
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
