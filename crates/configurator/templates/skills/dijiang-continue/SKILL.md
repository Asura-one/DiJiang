---
name: dijiang-continue
description: "Resume work on the current task: find active task and phase, load artifacts, then report the appropriate dj-* route."
triggers:
  - session:start
---

# 继续会话

继续当前 DiJiang 任务。本 skill 重建上下文并报告下一条 workflow 路线；不直接实现或收尾工作。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | 已有 DiJiang 任务状态、先前任务产物、workspace journal 和当前用户请求 |
| 输出 | 恢复后的上下文摘要、当前阶段、已加载产物，以及一个适合当前阶段的 `dj-*` 路线 |
| 非目标 | 不要在本 skill 中编辑代码、创建新计划或关闭任务 |

## 步骤

### 1. 加载状态

```bash
dijiang status
dijiang task current
dijiang workflow-state --json
git status --short --branch
```

输出：当前任务名称、任务状态、workflow 阶段、仓库分支和脏改状态。

如果 `dijiang task current` 报告没有 active task，不要臆造先前上下文。报告 `follow-up: dj-dispatch`，让当前用户请求进入分类。

### 2. 加载记忆上下文

存在 DiJiang workspace memory 时，从中恢复项目上下文。

```bash
dijiang mem list
```

如果 memory 命令失败，从任务产物继续，并在恢复摘要中说明 `memory unavailable`。

### 3. 读取当前任务产物

按以下顺序读取 active task 目录：

1. `task.json`
2. `prd.md` when present
3. `design.md` when present
4. `implement.md` when present
5. `check.md` or handoff artifacts when present

输出：任务目标、状态、已知决策、验证状态和未解决 blocker。

如果缺少 `task.json`，停止并输出 `blocking: task state corrupt; follow-up: dj-hunt`。继续会制造错误上下文。

### 4. 读取 Journal 上下文

只读取 `.dijiang/workspace/{{developer}}/` 中引用当前任务的近期条目。

如果 journal 与 `task.json` 冲突，以 `task.json` 为准，并在上下文摘要中记录冲突。

### 5. 选择下一个 Skill

| 阶段 / 状态 | Skill | 必需加载上下文 |
|---|---|---|
| requirements alignment | `dj-grill` | task goal and open questions |
| document creation | `dj-output` | PRD/design/doc target |
| implementation | `dj-implement` or `dj-tdd` | implement plan and verification loop |
| investigation / debugging | `dj-hunt` | symptom, reproduction, evidence |
| review / verification | `dj-check` | diff, requirements, validation output |
| completed | `dijiang-finish-work` | verification summary and version decision |

阶段不明确时，输出 `follow-up: dj-grill` 进行对齐；除非存在具体失败信号，具体失败输出 `follow-up: dj-hunt`。

## 失败处理

| 触发条件 | 第一动作 | 回退 |
|---|---|---|
| 没有 active task | 输出 `follow-up: dj-dispatch` | 只有 dispatch 无法推断意图时才询问任务选择 |
| 缺少 `task.json` | 停止正常继续流程 | 输出 `blocking: task state corrupt; follow-up: dj-hunt` |
| 引用的产物不存在 | 在摘要中标记缺失 | 只有下一个 skill 可以在没有它的情况下工作时才继续 |
| Journal 与任务状态冲突 | 以 `task.json` 为准 | 记录冲突；不安全时输出 `follow-up: dj-hunt` |
| Git 脏改早于当前 session | 列出脏改路径 | 实现前要求 worktree/范围决策 |
| Workflow state 命令失败 | 回退到 `task.json.status` | 在摘要中标记 workflow state degraded |

## 🔴 CHECKPOINT · Context Restored

离开该 skill 前先报告：

```text
当前任务: <name>
状态 / 阶段: <status> / <phase>
已加载产物: <paths>
缺失产物: <paths or none>
脏改状态: <summary>
路线: <dj-meta>
下一动作: <one sentence>
```

🛑 如果 active task 状态损坏、路线选择不明确，或继续需要猜测先前意图，停止。

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要把缺失任务文件当成空需求 | 停止并输出 `blocking: task state corrupt; follow-up: dj-hunt` |
| 不要只凭 memory 继续实现 | 先加载任务产物 |
| 不要因为 journal 里有旧状态就覆盖当前阶段 | 以 `task.json` 为准并记录冲突 |
| active task 存在时不要创建新任务 | 继续当前任务，或明确通过 dispatch 路由 |
| 不要从 continue mode 收尾 | 对 completed 任务输出 `follow-up: dijiang-finish-work` |
