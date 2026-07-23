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

### Runtime Route Gate

DiJiang 现在已经把一部分 workflow 规则从 skill 文本提升到了 runtime gate。当前已实现的是 Phase 1 Route Gate：

- 对已有 active task，`dijiang dispatch` 会根据 task status 和请求 intent 输出结构化 route decision。

- `planning` active task 的实现类请求会被 redirect 到 `dj-grill`；`dj-output` 仍允许作为 planning 阶段的文档产物入口。

- `paused` active task 会先 redirect 到 `dijiang-continue`。

- `archived` active task 会被 block，并提示重新 `dijiang start <task>`。

- 新建任务保留原有 classifier 行为，不强行套用 active-task route gate。

这套 gate 主要落在 `crates/task/src/route_gate.rs`、`crates/task/src/workflow_state.rs` 和 `crates/cli/src/main.rs`。CLI dispatch 输出会稳定包含 `action`、`reason` 和 `nextAction`。

### Phase Plan

- Phase 1: Route Gate，已完成。把 active task 的 workflow route 从 prompt 建议升级为 runtime hard gate。

- Phase 2: Git Gate，已完成最小闭环。把 active task 的实现类 dispatch 从纯 worktree 提示升级为 runtime readiness gate，支持 `ready / provisioned / blocked`，并区分“需要 provision”和“当前 runtime 不在正确 worktree”两类 blocked。

- Phase 3: Progressive Skill Loading，已部分落地。当前 runtime 已在 `dispatch` 与 `workflow-state` 两个 agent-facing 入口上复用同一套 shared body registry，相关的 Pi/Codex/OpenCode/Hermes agent prompt 也已统一改成优先消费这套 runtime context：先暴露 capsule-scoped skill manifests，再按 route 目标延迟展开单个或少量顺序/分叉 skill body，并已补上 risk/capsule 驱动的最小展开阈值；同时已提供 `dijiang skill-body` 作为兼容优先的执行期 lazy fetch 通路。当前仍未完成的只剩：切掉默认预注 body 的全链路切换。

- Phase 4: Approval / Capability Policy，已开始最小闭环。当前已对 `finish-work --integrate`、`finish-work --push` 与 merge 后 cleanup（worktree remove / branch delete）接入首批高风险 runtime approval gate：未显式批准时 block，显式批准后才允许继续集成、push 与清理。当前正式 phase plan 到 4 为止；`Phase 5+` 只出现在历史分析文档，不属于现行规范路线图。

### Finish Work 入口边界

| 入口 | 职责 |
|---|---|
| `/dijiang-finish-work` | Pi prompt checklist，只注入收尾步骤，不执行命令、不归档任务 |
| `/skill:dijiang-finish-work` | Agent skill workflow，加载 skill 后 agent 必须按 Invocation Contract 执行检查、版本决策和 git 隔离确认，需要真实状态变更时调用 CLI |
| `dijiang finish-work ...` | CLI state transition，有 active task 时归档任务；无 active task 时跳过归档但仍可完成验证、记录、commit/push/integrate |

`dijiang finish-work` 不再要求必须存在 active task；没有任务时不会凭对话内容自动创建或归档，只跳过 task archive / active-task cleanup。active task 指向缺失 artifact 时仍会报 stale state。

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
| Reasoning lens | `dj-reason` | 复杂判断、系统透镜和认知校准 |
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

当前这组 Git 规则已经有一部分下沉成 runtime hard gate。Phase 2 Git Gate 现在由 `crates/task/src/git_gate.rs` 提供 readiness evaluator，并由 `crates/cli/src/main.rs` 在 dispatch / active-task implementation route 中统一消费。当前已覆盖 `ready / provisioned / blocked`、缺失 task worktree metadata 时的 provision 决策、以及当前 runtime 仍在主 checkout 或错误 worktree 时的阻断。

后续增量：跨 worktree 的 `.dijiang` discovery 已由 `crates/cli/src/util.rs::resolve_dijiang_dir` 统一（worktree 本地 → 同仓 sibling → legacy 上溯；同时兼容 `.trellis`）。`finish-work` 使用同一 discovery，并对 integrate / push / cleanup 走 Phase 4 capability approval gate（`evaluate_capability`），不是 Phase 2 Git Gate evaluator 的同一条路径。

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
dijiang mem record --tactic T --outcome success --context C    # 记录策略事件
dijiang mem pattern --name N --description D [--cadence]        # 添加带元数据的工作流模式
# dijiang mem recommend  — 当前 CLI 无此子命令（mem 子命令见 `dijiang mem --help`）
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
# 注意：audit / cost / mcp 不是当前 dijiang 顶层子命令。
# MCP 入口为独立 crate dijiang-mcp（crates/mcp-server）；勿把下列伪命令当 CLI 文档。
# dijiang-mcp  # 独立二进制，非 `dijiang mcp` 子命令
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
