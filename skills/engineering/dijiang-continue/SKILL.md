---
name: dijiang-continue
description: "恢复当前任务：找到活跃任务和阶段，加载产物，然后报告合适的 dj-* 路线。 触发词：继续、恢复任务、接着做、上次的任务、resume、continue。"
disable-model-invocation: true
---

# 继续会话

继续当前 DiJiang 任务。本 skill 重建上下文并报告下一条 workflow 路线；不直接实现或收尾工作。

## 步骤

### 1. 加载状态

```bash
test -f .dijiang/active_task.txt && cat .dijiang/active_task.txt
git status --short --branch
```

如果不存在活跃任务指针，报告 `follow-up: dj-ask`，让当前用户请求进入分类。

### 2. 读取当前任务产物

读取活跃任务指针指向的任务目录：

1. `task.json` — 必须存在，否则报告 `blocking: task state corrupt; follow-up: dj-hunt`
2. `prd.md` — 需求文档
3. `design.md` — 技术设计（如果存在）
4. `implement.md` — 执行计划（如果存在）
5. `check.md` 或 handoff 产物（如果存在）

### 3. 加载记忆上下文

如果存在 `.dijiang/memory/` 目录，可以调用 `dj-memory` skill 恢复项目上下文。记忆命令失败时从任务产物继续。

### 4. 选择下一个 Skill

根据 `task.json` 的 `status` 推断阶段并推荐路线：

| 状态 | Skill | 需加载上下文 |
|---|---|---|
| planning | `dj-grill` | task goal and open questions |
| planning (有 PRD) | `dj-output` 或 `dj-split` | PRD/design 目标 |
| in_progress | `dj-implement` 或 `dj-tdd` | implement plan and verification loop |
| in_progress (bug) | `dj-hunt` | symptom, reproduction, evidence |
| in_progress (review) | `dj-check` | diff, requirements, validation output |
| completed | `dijiang-finish-work` | verification summary and version decision |
| paused | 恢复后进入 planning 或 in_progress | 恢复后的任务上下文 |

**🔴 实现门禁（in_progress → 推荐实现类 skill 前）**：任务要进入实现（`dj-implement`/`dj-tdd`）时，先读 `{task_dir}/task.json` 的 `meta.grilling` 门禁证据。若 `meta.grilling` 缺失、或 `confirmedAt`/`confirmation` 未填 → **拷问门禁未过，推荐 `dj-grill` 补逐轮对齐，不得直接推实现**。有 `planning` 转 `in_progress` 的中间产物但缺门禁证据，标「门禁缺失」而非 `in_progress`。

阶段不明确时，输出 `follow-up: dj-grill` 进行对齐。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 产物文件与任务状态矛盾 | 以最新产物为准 | 标矛盾，请用户裁决 |
| 活跃任务指针指向不存在目录 | 报告，指针失效 | 问用户选哪个任务 |
| `in_progress` 但 `meta.grilling` 门禁证据缺失 | 报告门禁缺失，回 `dj-grill` 补对齐 | 用户明确确认已对齐后放行实现 |

## 状态报告

```text
当前任务: <name>
状态: <status>
已加载产物: <paths>
缺失产物: <paths or none>
脏改状态: <summary>
路线: <dj-* skill name>
下一动作: <one sentence>
```

如果 active task 状态损坏、路线选择不明确，或继续需要猜测先前意图，停止。

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要把缺失任务文件当成空需求 | 停止并报告 task state corrupt |
| 不要只凭记忆继续实现 | 先加载任务产物 |
| active task 存在时不要创建新任务 | 继续当前任务，或通过 dj-ask 路由 |
| 不要从 continue mode 收尾 | 对 completed 任务输出 `follow-up: dijiang-finish-work` |
