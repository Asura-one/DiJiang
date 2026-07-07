# 用户指南

## Skill 选择流程

以下流程图帮助你根据不同场景选择正确的 `dj-*` skill。

### 按任务类型选择

```
你的任务是什么？
├── 新功能 / 明确需求
│   ├── 有测试要求 → dj-tdd
│   └── 无测试要求 → dj-implement
├── Bug / 回归
│   └── dj-hunt
├── 需求不明确 / 范围模糊
│   └── dj-grill → 对齐后再走实现路径
├── 审查现有代码
│   ├── 需要运行测试、检查完整性 → dj-check
│   ├── 只需要快速看一眼、不改代码 → dj-review
│   ├── 全仓审计 / 过度工程扫描 → dj-audit
│   ├── 技术债追踪 → dj-debt
│   └── 健康报告 → dj-health
├── 设计 / UI
│   └── dj-design
├── 原型验证
│   └── dj-prototype
├── 脚本 / 工具编写
│   └── dj-script
├── 文档
│   ├── 写 PRD / 设计文档 / API 文档 → dj-output
│   └── 文本润色、去 AI 味 → dj-write
├── 模式研究
│   └── dj-pattern
├── 复杂判断 / 系统透镜
│   └── dj-reason
├── 最小改动 / YAGNI 约束
│   └── dj-ponytail
├── 长代码讨论
│   └── dj-karpathy
└── Session 交接
    └── dj-handoff
```

### 按任务状态选择

| 当前任务状态 | 允许的 skill | 说明 |
|-------------|-------------|------|
| `none` | `dj-dispatch` | 先分流，不要直接干活 |
| `planning` | `dj-grill`、`dj-output` | 对齐阶段；实现类请求会被 Route Gate 重定向到 `dj-grill` |
| `in_progress` | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-script` / `dj-check` 等 | 实现阶段，按需选择 |
| `completed` | 无（走 `dijiang finish-work`） | 收尾，不使用 skill |
| `archived` | 无 | 只读，需 `dijiang start <task>` 重新激活 |
| `paused` | `dijiang-continue` | 恢复后回到 planning 或 in_progress |

## 常见工作流

### 场景 A：开始一个新功能

```
1. dijiang start feat-user-auth
   → 创建任务，状态为 in_progress

2. dijiang dispatch "实现用户登录模块"
   → 引擎分类为 dj-implement，分配 worktree

3. 在 worktree 中实现 → 测试 → dj-check 审查

4. dijiang finish-work --verification "测试通过" --docs-sync "CHANGELOG" --version-impact minor --commit
   → 提交、归档任务
```

### 场景 B：修复一个 Bug

```
1. dijiang dispatch "用户登录后页面空白"
   → 引擎分类为 dj-hunt（bug/regression）

2. dj-hunt 排查根因 → 找到问题 → 修复 → 验证

3. dijiang finish-work --verification "Bug 已修复，回归测试通过" --docs-sync "CHANGELOG" --version-impact patch --commit
```

### 场景 C：需求不明确

```
1. dijiang dispatch "优化用户体验"
   → 需求模糊 → 引擎分类为 dj-grill

2. dj-grill 追问 2-3 轮，明确具体范围
   → 输出对齐结论

3. dj-output 根据对齐结论创建 PRD

4. 确认 PRD 后 → dj-implement 实现
```

### 场景 D：跨 Session 交接

```
1. 当前 session 结束前：dj-handoff
   → 输出 session 摘要、未决问题、下一步建议

2. 新 session 开始：加载 dj-handoff 产物
   → 快速回到工作状态
```

### 场景 E：并行任务

```
1. channel spawn "安装依赖的 agent"
2. channel spawn "写测试的 agent"
3. channel execute-all --timeout 120
   → 两个 channel 并行执行
4. channel status 查看结果
```

## Worktree 工作流

DiJiang 使用 git worktree 隔离代码修改。了解该流程可避免常见问题。

```
主仓库（main checkout）   ← 始终保持干净
  │
  ├── dijiang dispatch ...
  │   → Git Gate 检测为主仓库
  │   → 自动创建 worktree ../dijiang-<task>
  │   → 切换到 worktree 执行
  │
  └── dijiang finish-work
      → 提交、集成
      → 询问是否删除 worktree
      → 是：自动删除 worktree 和分支
      → 否：保留 worktree，手动清理
```

### 常见误区和处理方法

| 问题 | 原因 | 处理方式 |
|------|------|----------|
| `dijiang dispatch` 找不到 `.dijiang/` | 在 worktree 内直接运行，但跨 worktree discovery 尚未全量覆盖 | 使用主仓库目录运行 dispatch，引擎会自动供应 worktree |
| 忘记在哪个 worktree 里 | 多个 worktree 正在使用 | `dijiang task current` 查看当前 task metadata；`git worktree list` 列出所有 worktree |
| finish-work cleanup gate 被阻断 | 破坏性操作需要显式批准 | 检查 prompt 中 gate 的提示，显式确认后再运行 |
| "当前不在正确的 worktree 里" | runtime 在主 checkout 或错误的 worktree 中 | 引擎会 block 并提供 nextAction |

## 门禁系统（Gates）

三层的安全约束确保你不会意外破坏仓库。以下是各门禁的触发条件：

| 门禁 | 触发场景 | 处理方式 |
|------|---------|----------|
| Route Gate | 对 `archived` task dispatch 实现类请求 | block → 提示重新 start |
| Route Gate | 对 `planning` task dispatch 实现类请求 | redirect → 走 dj-grill |
| Route Gate | 对 `paused` task dispatch 任何请求 | redirect → 走 dijiang-continue |
| Git Gate | 在主 checkout 直接 dispatch 代码路由 | provision → 自动创建 worktree |
| Git Gate | 在错误 worktree 中运行 | block → 提示切换到正确 worktree |
| Capability Gate | `finish-work --integrate` 未显式批准 | block → 需确认后才允许 |
| Capability Gate | `finish-work --push` 未显式批准 | block → 需确认后才允许 |
| Capability Gate | merge 后 worktree cleanup 未批准 | block → 需确认后才清理 |

## Pi Extension 集成

DiJiang 通过 Pi Extension（`.pi/extensions/dijiang/index.ts`）与 Pi 平台深度集成。扩展在 Pi 的 agent 生命周期事件中自动注入路由上下文，无需用户手动触发。

### 注册方式

`.pi/settings.json` 中配置：
```json
{
  "enable_skill_commands": true,
  "extensions": ["./extensions/dijiang/index.ts"],
  "skills": ["./skills"],
  "prompts": ["./prompts"],
  "agents": []
}
```

### 自动注入行为

扩展在以下场景自动工作：

1. **agent 启动或用户提交提示时**：自动运行 `dijiang workflow-state` 刷新状态信息，同时运行 `dijiang dispatch --json --hook-event` 分类提示。agent 提示顶部注入 `<dijiang-workflow-state>` 和 `<dijiang-route>` 路由上下文。
2. **bash 命令失败时**：自动注入 `<dijiang-route>` 路由到 `dj-hunt`，附带失败命令。agent 应停止当前实现流程，转向排查模式。
3. **验证/检查通过且有脏 diff 时**：自动注入 `<dijiang-route>` 路由到 `dj-output`。agent 应在完成验证后先同步文档再收尾。
4. **每次 bash 工具调用时**：自动注入 `DIJIANG_CONTEXT_ID` 环境变量，用于标识来源 session。
5. **session 开始/关闭时**：自动刷新状态栏和 widget。

路由注入有去重机制——同一 session 中同一命令不重复注入。

### 子 Agent 系统

DiJiang 定义三个 Pi 子 agent（`.pi/agents/`），分别处理不同职责：

| 子 agent | 职责 | 加载来源 |
|---------|------|----------|
| dijiang-check | 质量审查、审计、技术债、健康报告 | `.pi/agents/dijiang-check.md` |
| dijiang-implement | 特性实现、TDD、原型、脚本 | `.pi/agents/dijiang-implement.md` |
| dijiang-research | 技术调研、bug 排查、分类 | `.pi/agents/dijiang-research.md` |

子 agent 加载后的第一件事是读取 `dijiang workflow-state --json` 获取运行时路由上下文，然后根据 `<dijiang-target-skill>` 确定使用哪个 `dj-*` skill。

### UI 组件

扩展提供两处可视化信息：

- **状态栏**（Status Bar）：两个条目 `dijiang-task` 和 `dijiang-capsule`，显示 `{任务标题} [{capsule}]`，让你知道当前是否在正确的工作流阶段。
- **Widget**：显示详细信息行 `任务: {标题} | 状态: {status} | Capsule: {capsule} | Gate: {gate}`。

Pi 内置命令 `/dijiang` 可手动刷新并显示当前任务和 capsule 摘要。

### 与 CLI 分立的运行时上下文

扩展在每次 agent 启动时自动注入 `<dijiang-workflow-state>` 和 `<dijiang-route>` 上下文，包含最新工作流状态、Skill Manifests 和目标 skill 路由。这是 agent 消费 DiJiang 上下文的**主要渠道**——agent 不应主动频繁调用 `dijiang dispatch` 或 `dijiang workflow-state`，扩展会在适当时机自动注入。

### Prompt 模板

Pi 提供三个 `/dijiang-*` prompt 模板作为轻量检查清单：

- `/dijiang-start` — 加载当前任务上下文和 workflow.md，适合新 session 开始时手动调用。
- `/dijiang-finish-work` — 验证、检查、文档同步、版本决策、记忆记录、收尾执行的步骤清单，适合功能完成时手动调用。
- `/dijiang-reason` — `dj-reason` 的轻量入口；真正的推理增强流程由 `/skill:dj-reason` 承载。

## CLI 速查

### 项目初始化

```bash
dijiang init my-project --yes
dijiang init my-project --platforms pi,codex --yes
```

### 任务生命周期

```bash
dijiang start <name>
dijiang task list
dijiang task current
dijiang task status <name> <status>
dijiang task archive <name>
dijiang task prune --days 30
```

### Dispatch（核心入口）

```bash
dijiang dispatch "实现用户注册"
dijiang dispatch "修复登录 bug"
```

### 收尾

```bash
dijiang finish-work --verification "..." --docs-sync "CHANGELOG" --version-impact patch
dijiang finish-work --verification "..." --docs-sync "CHANGELOG" --version-impact minor --commit
dijiang finish-work --verification "..." --docs-sync "CHANGELOG" --version-impact major --commit --push
```

### 记忆

```bash
dijiang mem findings --finding "用户反馈登录后页面空白，排查发现 token 解析失败"
dijiang mem learn --lesson "token 过期时间需要同步到前端"
dijiang mem archive
dijiang mem list
dijiang mem sync
```

### Channel（并行）

```bash
dijiang channel spawn "安装依赖"    # agent 1
dijiang channel spawn "写测试"      # agent 2
dijiang channel execute-all         # 并行执行
dijiang channel list                # 查看状态
```

### 模板和更新

```bash
dijiang template list
dijiang skills --sync
dijiang update
```

## 排查指南

### Git Gate 相关问题

**现象**：dispatch 后显示 `blocked`，原因与 worktree 相关。

**检查步骤**：
1. `git worktree list` — 查看所有 worktree
2. `dijiang task current` — 确认 active task 的 metadata
3. 确认当前目录是否有 `.dijiang/`（`ls -la .dijiang/`）
4. 在主仓库目录重新运行 dispatch

### Route Gate 相关问题

**现象**：dispatch 后路由与期望不符（例如想实现功能却被重定向到 dj-grill）。

**检查步骤**：
1. `dijiang task current` — 查看当前任务状态
2. 如果任务状态为 `planning`，实现类请求会被重定向，属于正常行为
3. 如果任务状态为 `completed`，需要先 `dijiang start <task>` 重新激活
4. 如果任务状态为 `archived`，必须先 `dijiang start <task>`

### Memory 相关问题

**现象**：`dijiang mem findings` 不生效。

**检查步骤**：
1. `dijiang mem list` — 确认平台 session 是否正确加载
2. `dijiang status` — 确认 `.dijiang/` 存在且配置正确
3. 如果使用平台适配器（Pi、Codex 等），确认平台 API 可用

### Channel 相关问题

**现象**：channel 执行超时或失败。

**检查步骤**：
1. 增加超时时间 `channel execute-all --timeout 300`
2. `channel status <id>` 查看具体 agent 状态
3. 检查 agent 是否正确安装了所需工具和依赖

### 收尾相关问题

**现象**：`dijiang finish-work` 在 cleanup gate 被阻断。

**处理方式**：
- 如果需要清理 worktree：确认后，gate 会放行
- 如果需要保留 worktree 手动处理：选择不清理，后续手动 `git worktree remove` 和 `git branch -d`
