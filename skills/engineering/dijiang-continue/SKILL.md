---
name: dijiang-continue
description: "恢复当前任务：找到活跃任务和阶段，加载产物，然后报告合适的 dj-* 路线。"
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

如果不存在活跃任务指针，报告 `follow-up: dj-dispatch`，让当前用户请求进入分类。

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

阶段不明确时，输出 `follow-up: dj-grill` 进行对齐。

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
| active task 存在时不要创建新任务 | 继续当前任务，或通过 dj-dispatch 路由 |
| 不要从 continue mode 收尾 | 对 completed 任务输出 `follow-up: dijiang-finish-work` |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '42d95e2a-b138-445d-bd4f-9e3f119231b2'
  PropagateID: '42d95e2a-b138-445d-bd4f-9e3f119231b2'
  ReservedCode1: '6cd43ed8-285d-42ba-b5d0-770c4c6f114f'
  ReservedCode2: '6cd43ed8-285d-42ba-b5d0-770c4c6f114f'
