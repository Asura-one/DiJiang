---
name: dijiang-start
description: "启动 DiJiang session：加载项目上下文、当前任务和状态，然后报告合适的 dj-* 路线。"
disable-model-invocation: true
---

# 启动会话

启动由 DiJiang 管理的开发会话。本 skill 只加载上下文并报告选中的路线；任务执行交给选中的 `dj-*` skill。

## 步骤

### 1. 加载项目状态

读取以下文件（如果存在）：

- `.dijiang/config.toml` — 项目配置（名称、开发者、平台）
- `.dijiang/active_task.txt` — 活跃任务指针（如果存在）
- `.dijiang/tasks/` — 列出任务目录

如果没有 `.dijiang/` 目录，停止并提示用户运行 `dj-setup` 初始化项目。

```bash
test -d .dijiang/ && echo "found" || echo "missing"
test -f .dijiang/config.toml && cat .dijiang/config.toml
test -f .dijiang/active_task.txt && cat .dijiang/active_task.txt
ls .dijiang/tasks/ 2>/dev/null
git status --short --branch
```

### 2. 读取当前任务

如果存在活跃任务指针，读取任务目录中的文件：

1. `task.json` — 任务元数据（id/title/status/branch/notes/meta）
2. `prd.md` — 需求文档
3. `design.md` — 技术设计（如果存在）
4. `implement.md` — 执行计划（如果存在）

根据 `task.json` 的 `status` 字段推断当前阶段：
- `planning` → 需求对齐阶段
- `in_progress` → 实现阶段
- `completed` → 收尾阶段
- `paused` → 恢复阶段
- `archived` → 已归档（只读）

### 3. 发现 Specs

```bash
test -d .dijiang/spec && find .dijiang/spec -maxdepth 2 -type f -name "*.md" | sort
```

读取与当前任务相关的 spec 文件（按任务 scope 或 package 匹配）。

### 4. 初始化会话记忆

如果用户已给出具体任务描述，可以调用 `dj-memory` skill 记录初始发现。不要因为记忆持久化阻塞会话启动。

### 5. 交接给 dj-dispatch

不要在 `dijiang-start` 内部分类请求。启动只加载上下文；任务分类和路由委托给 `dj-dispatch`。

Call the Skill tool with "dj-dispatch".

## 状态报告

离开该 skill 前报告：

```text
当前任务: <name or none>
状态: <planning|in_progress|completed|paused|archived|none>
已加载 specs: <paths or none>
路线: <dj-* skill name>
下一动作: <one sentence>
```

如果路线不明确、项目状态缺失，或继续需要猜测任务意图，停止。

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要在 startup 中实现代码 | 先输出路线 |
| 不要臆造任务状态 | 如果 .dijiang/ 缺失，提示运行 dj-setup |
| 不要忽略 git 脏改状态 | 报告它，并要求编辑前使用 worktree |
| 不要从猜测写入记忆 | 将不确定上下文保留在任务产物中 |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '49901892-17a0-452d-bcfd-6be3bc03a04a'
  PropagateID: '49901892-17a0-452d-bcfd-6be3bc03a04a'
  ReservedCode1: 'f0937a1e-6196-4b4a-bb2c-eaef0500bebc'
  ReservedCode2: 'f0937a1e-6196-4b4a-bb2c-eaef0500bebc'
