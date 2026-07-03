<!-- DIJIANG:START -->
# DiJiang Project Instructions

This project uses DiJiang for task management and workflow.

## Project Structure

- `.dijiang/` — DiJiang project state and configuration
- `.dijiang/tasks/` — active and archived tasks
- `.dijiang/spec/` — coding guidelines
- `.dijiang/workspace/` — developer journals
- `.dijiang/workflow.md` — canonical workflow projection
- `.pi/` — Pi platform configuration

## Layer Boundaries

| Layer | Responsibility |
|-------|----------------|
| `dijiang` CLI | Project state, task lifecycle, memory persistence, templates, platform config, agent channels; `dijiang finish-work` is the only layer that mutates task archive/journal/commit/push/integration state |
| `dj-*` skills | 原子工作能力，例如对齐、实现、调查、检查、文档和报告 |
| `dijiang-*` skills | start、continue、finish-work 的 session 包装器；`/skill:dijiang-finish-work` 加载 finish-work skill，agent 调用 CLI 前必须遵守其调用契约 |
| `/dijiang-*` prompts | 轻量 Pi prompt checklist；只注入指导，不执行 CLI 状态转换 |
| `AGENTS.md` | agent 的最小路由索引；不是第二份 workflow 定义 |

## CLI 命令

| 命令 | 说明 |
|---------|-------------|
| `dijiang init [name]` | 初始化 DiJiang 项目状态和平台配置 |
| `dijiang status` | 显示项目状态 |
| `dijiang status --compat` | 显示兼容性诊断 |
| `dijiang start <name>` | 创建并激活工作 session |
| `dijiang dispatch <prompt>` | 从自然语言请求创建或复用 active task，并输出路由上下文 |
| `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | 通过验证、docs/spec 证据、版本决策、可选 commit/集成、journal 和归档完成当前工作 |
| `dijiang task list` | 列出 active tasks |
| `dijiang task current` | 显示 active task |
| `dijiang task start <name>` | 用底层任务语义创建或激活任务记录 |
| `dijiang task status <name> <status>` | 更新任务状态 |
| `dijiang task archive <name>` | 归档任务 |
| `dijiang task prune --days N` | 清理旧的已归档任务 |
| `dijiang mem list` | 列出平台 sessions |
| `dijiang mem sync` | 同步平台 sessions |
| `dijiang mem findings --finding "..."` | 追加项目 finding |
| `dijiang mem learn --lesson "..."` | 记录项目 lesson |
| `dijiang mem correction --correction "..." --lesson "..." --actionability "..."` | 记录带记忆质量元数据的用户纠正 |
| `dijiang mem archive` | 归档当前 memory session |
| `dijiang mem tactic --name N --description D` | 添加全局 tactic |
| `dijiang mem tactics --select N` | 列出或选择使用 Thompson sampling 的 tactics |
| `dijiang mem record --tactic T --outcome success --context C` | 记录 tactic 结果 |
| `dijiang mem pattern --name N --description D` | 添加项目 pattern 或标准操作流程 |
| `dijiang mem patterns` | 列出项目 patterns |
| `dijiang mem stats` | 显示 memory 统计 |
| `dijiang mem backup` | 将项目 memory 备份到全局存储 |
| `dijiang mem evolve` | 分析 session memory 并提取 tactics |
| `dijiang mem finetune` | 运行较慢的 memory fine-tuning 循环 |
| `dijiang template list` | 列出可用模板 |
| `dijiang template pull <source>` | 拉取模板 |
| `dijiang template validate <path>` | 验证模板 |
| `dijiang skills` | 列出可用 `dj-*` skills |
| `dijiang skills --sync` | 同步项目 `dj-*` skills |
| `dijiang workflow-state --json` | 输出供 hooks/agents 使用的 workflow 状态 |
| `dijiang migrate` | 将 legacy `.trellis/` 状态迁移到 `.dijiang/` |
| `dijiang channel spawn <agent>` | 创建 agent channel |
| `dijiang channel list` | 列出 active channels |
| `dijiang channel send <id> <message>` | 向 channel 发送消息 |
| `dijiang channel execute <id>` | 执行 channel 中的 agent |
| `dijiang channel execute-all` | 并行执行所有 active channels |
| `dijiang channel status <id>` | 检查 channel 状态 |
| `dijiang channel stop <id>` | 停止 channel |
| `dijiang update` | 更新 DiJiang 管理的 skills、agents、prompts、hooks 和 workflow 投影 |
| `dijiang update --from-github` | 更新项目之前，先从 GitHub 刷新全局 skills |

## Skill 路由

| 类别 | 使用 |
|----------|-----|
| 新任务 / 不清楚的请求 | `dj-dispatch` |
| 需求对齐；模糊的功能/优化或 bug/fix 请求 | `dj-grill` |
| 有明确对象/范围的具体功能实现 | `dj-implement` 或 `dj-tdd` |
| Bug / regression | `dj-hunt` |
| Code review / 质量门禁 | `dj-check` |
| 轻量只读 review | `dj-review` |
| 全仓审计 | `dj-audit` |
| 技术债评估 | `dj-debt` |
| 代码库健康报告 | `dj-health` |
| Pattern 研究 | `dj-pattern` |
| 最小聚焦改动 | `dj-ponytail` |
| Prototype | `dj-prototype` |
| UI design | `dj-design` |
| Script / tool | `dj-script` |
| 写作润色 | `dj-write` |
| 长篇代码讨论 | `dj-karpathy` |
| Session handoff | `dj-handoff` |
| Session findings / lessons | `dijiang mem findings` / `dijiang mem learn` |

## Workflow 路由

1. Session 开始时读取本文件和 `.dijiang/workflow.md`。
2. 用 `dijiang task current` 检查 active task。
3. 存在时读取 task artifacts：`task.json`、`prd.md`、`design.md`、`implement.md`。
4. 读取 `.dijiang/spec/` 中相关 spec 文件。
5. 按规范任务状态路由：
   - none → `dijiang start <name>` 或 `dj-dispatch`
   - `planning` → `dj-grill`，可选 `dj-output`
   - `in_progress` → implementation skill，然后 `dj-check`
   - `completed` → `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>`
   - `archived` → 只读，除非用 `dijiang start <task>` 重启
   - `paused` → `dijiang-continue`，然后回到 `planning` 或 `in_progress`

`review` 不是规范任务状态。质量验证使用 `dj-check`。

## 范围纪律

- **不多管闲事**。不添加未要求的功能，不重构，不做"顺带改进"。修复一个 bug 不需要清理周边代码。如果发现不相关的死代码，提一句——但不要动它。
- **不过度设计**。不为不可能发生的场景添加错误处理、回退或校验。只在系统边界（用户输入、外部 API）校验，信任内部代码和框架保证。
- **不提前抽象**。不为一次性操作创建 helper、utility 或抽象层。不为假想的未来需求设计。三行重复代码也好过一个过早出现的抽象。写了 200 行但 50 行能解决的，重写。
- **不乱建文件**。非绝对必要不创建新文件，始终优先修改现有文件。**绝不主动创建文档文件（`*.md`）或 README**。
- **不改没读过的代码**。必须先阅读和理解，再提议修改。

改动测试：每一行变更都应能直接追溯到用户的请求。你自己的改动带出的孤儿（未使用的 import、变量、函数）要清理；改动前就存在的死代码，除非被要求，否则不动。

Managed by DiJiang. Edits outside this block are preserved.
<!-- DIJIANG:END -->
