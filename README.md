# 帝江 (Dijiang)

> 浑敦无面目，是识歌舞。——《山海经·西山经》

融合 [mattpocock/skills](https://github.com/mattpocock/skills)、[ponytail](https://github.com/DietrichGebert/ponytail)、[Waza](https://github.com/tw93/Waza) 三大 skill 库的工程工作流。

## 定位

DiJiang 是独立的 Rust-native agent harness。它提供两层能力：

- **`dijiang` CLI**：管理项目初始化、任务生命周期、记忆持久化、模板/平台配置和 agent channel。
- **`dj-*` skills**：提供需求对齐、实现、排查、质量检查、审计、文档等原子工作能力。

DiJiang 可读取部分 Trellis 结构作为 legacy compatibility fallback，但 `.dijiang/*` 是 DiJiang 的主路径。

## Canonical Workflow

```
none
  └─ dispatch: dijiang start <name> 或 dj-dispatch
planning
  └─ align: dj-grill，必要时 dj-output
in_progress
  ├─ implement: dj-implement / dj-tdd / dj-hunt / dj-script / dj-design
  └─ check: dj-check
completed
  └─ finish: dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>
archived
  └─ closed: 只读；如需继续则重新 dijiang start <task>
paused
  └─ resume: dijiang-continue 后回到 planning 或 in_progress
```

`review` 不是 DiJiang 的正式 task status，也没有独立 CLI 入口。质量闸门统一由 `dj-check` 承担；轻量只读审查使用 `dj-review` skill。

## Skill 清单

### 核心流程

| 类别 | Skill | 触发 |
|---|---|---|
| Routing | `dj-dispatch` | 新任务分流和技能路由 |
| Alignment | `dj-grill` | 需求不清、范围需要对齐 |
| Planning docs | `dj-output` | PRD、design、implement、spec 与代码一致性 |
| Implementation | `dj-implement` | 特性代码实现 |
| Implementation | `dj-tdd` | 明确要求测试驱动或适合红绿重构 |
| Implementation | `dj-hunt` | bug、回归、根因排查 |
| Quality gate | `dj-check` | 代码审查、功能完整性、安全性、回归影响检查 |
| Review lens | `dj-review` | 轻量只读审查，不运行测试、不修改代码、不替代质量闸门 |

### 辅助能力

| 类别 | Skill | 触发 |
|---|---|---|
| Analysis reports | `dj-audit` | 全仓审计或过度工程扫描 |
| Analysis reports | `dj-debt` | 技术债追踪 |
| Analysis reports | `dj-health` | codebase / agent 配置健康检查 |
| Analysis reports | `dj-pattern` | 模式研究、重复抽象机会分析 |
| Style overlays | `dj-ponytail` | 极简、YAGNI、最小改动约束 |
| Style overlays | `dj-karpathy` | 长代码讨论和 LLM 编码纪律 |
| Implementation | `dj-design` | UI/UX 主导的设计实现 |
| Implementation | `dj-prototype` | 原型验证 |
| Implementation | `dj-script` | 脚本或工具编写 |
| Writing polish | `dj-write` | 文本润色、去 AI 味、proofread |
| Session transfer | `dj-handoff` | 跨 session 交接 |

## 全局约束：Git 安全工作流（Worktree-First）

所有涉及 git 操作的 skill 自动遵守以下规则：

1. 主工作区永远干净，只做同步，严禁在主目录上直接写代码。
2. 每个功能一个独立 worktree，所有开发、AI 调试均在 worktree 中进行。
3. 合并需用户确认，展示变更摘要后等待确认。
4. 回滚必须备份 + 确认，tag 备份 → 用户确认 → 执行。
5. 禁止自动执行破坏性操作：`reset --hard`、`force push`、`clean -f`、`rm -rf worktree` 等。
6. 提交信息遵循 Conventional Commits。
7. 版本号使用 Major.Minor.Revision。

## CLI 工具

`dijiang` 是 Rust 编写的命令行工具，管理项目生命周期、任务、记忆、模板和平台集成。

### init — 初始化项目

```bash
# 创建一个新项目
dijiang init my-project --yes

# 指定平台
dijiang init my-project --platforms pi,codex,cursor --yes

# 强制重新初始化
dijiang init my-project --force
```

### status — 查看项目状态

```bash
# 显示项目名、活跃任务、任务列表、平台状态
dijiang status

# 显示详细兼容诊断
dijiang status --compat
```

### task — 任务管理

```bash
dijiang start <name>                         # 创建并激活一个工作会话
dijiang task list                            # 列出所有任务
dijiang task current                         # 显示活跃任务
dijiang task status <name> <status>          # 更新任务状态
dijiang task archive <name>                  # 归档任务
dijiang task prune --days N                  # 删除超过 N 天的已归档任务
dijiang finish-work --verification "..." --docs-sync "..." --version-impact none --commit
```

### mem — 记忆管理

```bash
dijiang mem list                             # 列出跨平台会话
dijiang mem sync                             # 同步平台会话到 ~/.dijiang/mem/
dijiang mem findings --finding "..."         # 追加项目发现
dijiang mem learn --lesson "..."             # 记录项目学习
dijiang mem archive                          # 归档当前会话
dijiang mem tactic --name N --description D  # 添加全局策略
dijiang mem record --tactic T --outcome success --context C
```

### template / skills / channel

```bash
dijiang template list
dijiang template pull gh:owner/repo
dijiang template validate <path>
dijiang skills --sync
dijiang workflow-state --json
dijiang channel spawn <agent>
dijiang channel list
```

## 兼容性

- Pi ✅
- Codex ✅
- Cursor ✅
- Claude ✅
- OpenCode ✅
- Hermes ✅

## 测试验证

```bash
# 全量测试
cargo test

# 分 crate 测试
cargo test -p dijiang-task
cargo test -p dijiang-configurator
cargo test -p dijiang --test e2e

# 编译检查
cargo build
```

## 设计原则

1. **Predictability** — 每次运行走相同流程，而非产出相同结果。
2. **YAGNI** — 不需要的不写，stdlib 能做的不引入依赖。
3. **Fail-safe** — 破坏性操作必须确认，回滚必须备份。
4. **Composable** — skill 之间可串联，也可单独使用。
5. **Runtime-neutral** — 不绑定特定 agent runtime。
