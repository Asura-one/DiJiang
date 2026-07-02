---
name: dijiang-start
description: "启动 DiJiang session：加载项目上下文、当前任务和 workflow 状态，然后报告合适的 dj-* 路线。"
triggers:
  - session:start
---

# 启动会话

启动由 DiJiang 管理的开发会话。本 skill 只加载上下文并报告选中的路线；任务执行交给选中的 `dj-*` skill。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | 用户请求、当前仓库、DiJiang 项目状态 |
| 输出 | 当前任务上下文、已加载 workflow/specs，以及选中的 `dj-*` 路线 |
| 非目标 | 不在本 skill 中实现、编辑代码、提交或收尾任务 |

## 步骤

### 1. 加载项目状态

```bash
dijiang status
dijiang task current
git status --short --branch
```

输出：当前任务路径、当前阶段、git 分支、脏改状态，以及可用时的开发者身份。

如果 `dijiang status` 失败，运行 `pwd` 并检查 `.dijiang/` 是否存在。若缺少 `.dijiang/`，停止并输出 `follow-up: project initialization`，不要臆造任务状态。

### 2. 读取 Workflow

```bash
test -f .dijiang/workflow.md && cat .dijiang/workflow.md
dijiang workflow-state --json
```

输出：规范阶段映射和当前 workflow 状态。

如果缺少 `.dijiang/workflow.md`，读取 `AGENTS.md`；只有其中包含 DiJiang block 时才继续。两个文件都不存在时，用 `missing DiJiang workflow context` 停止。

### 3. 发现 Specs

```bash
test -d .dijiang/spec && find .dijiang/spec -maxdepth 2 -type f | sort
test -f .dijiang/spec/index.md && cat .dijiang/spec/index.md
```

输出：路由或实现前需要加载的适用 spec 文件。

`.trellis/` 可作为迁移项目的 legacy fallback 存在，但新会话必须先加载 `.dijiang/`。

### 4. 初始化会话记忆

仅当用户已经给出具体任务描述时，才记录初始发现。Durable memory 必须包含 source 和 scope；未经确认的猜测留在任务上下文中。

```bash
dijiang mem findings --finding "<initial task description; source=user; scope=current task>"
```

如果 memory 命令失败，不写入 memory 并继续，在 handoff 中说明失败。不要因为 memory 持久化阻塞会话启动。

### 5. 交接给 dj-dispatch

不要在 `dijiang-start` 内部分类请求。启动只加载上下文；任务分类和级别选择必须委托给唯一路由权威 `dj-dispatch`。

```bash
dijiang dispatch "<user request>" --json --hook-event session:start
```

已有 active task 时，先读取它的 `task.json` 和 workflow-state 输出，再判断当前请求是继续任务还是冲突的新请求。该判断使用 `dj-dispatch` 的 session-start 规则。

只有项目状态不安全解释时，才直接路由到 `dj-hunt`。不安全状态仅限：active task 指针缺失/陈旧、`task.json` 缺失或不可读、任务状态无效，或 workflow-state 与 `task.json` 对 active task 或状态存在直接冲突。缺少 `prd.md`、`design.md`、`implement.md`、check notes 等可选产物不是 hunt 触发条件；标记为缺失，并继续通过 `dj-dispatch` 或基于状态的路线。不要把每个 `in_progress` 任务都当成 hunt 任务。

## 失败处理

| 触发条件 | 第一动作 | 回退 |
|---|---|---|
| `dijiang status` 失败 | 确认仓库根目录和 `.dijiang/` 是否存在 | 停止，并要求初始化项目或修正工作目录 |
| 没有 active task | 将用户请求交给 `dj-dispatch` 路由 | 只通过 DiJiang dispatch/start 流程创建任务 |
| 当前任务存在但产物缺失 | 读取 `task.json` 并列出已存在文件 | 只有 `task.json` 缺失/不可读或状态无效时路由到 `dj-hunt`；可选文档可以缺失 |
| Workflow state 与任务状态冲突 | 以 `.dijiang/tasks/<task>/task.json` 作为事实源 | 记录冲突；若继续执行不安全，输出 `follow-up: dj-hunt` |
| 开始前 git tree 已脏 | 报告脏改文件，并将其排除在新任务范围外 | 实现前使用专用 worktree |

## 🔴 CHECKPOINT · 启动完成

离开该 skill 前先报告：

```text
当前任务: <name or none>
Workflow 阶段: <planning|in_progress|completed|paused|none>
已加载 specs: <paths or none>
路线: <dj-skill>
下一动作: <one sentence>
```

🛑 如果路线不明确、项目状态缺失，或继续需要猜测任务意图，停止。

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要在 startup 中实现代码 | 先输出 `dj-dispatch` 返回的路线 |
| 不要在 DiJiang 报告没有 active task 时臆造一个任务 | 使用 `dj-dispatch` 创建或选择任务 |
| 不要忽略 git 脏改状态 | 报告它，并要求编辑前使用任务 worktree |
| 不要从猜测写入 durable memory | 将不确定上下文保留在任务产物中 |
| 不要在 startup 中复制 `dj-dispatch` 分类表 | 将模糊请求和新任务路由委托给 `dj-dispatch` |
