---
name: dj-setup
description: "初始化 DiJiang 项目状态或从旧版本迁移。创建 .dijiang/ 目录结构、config.toml 和 CONTEXT.md。每个项目运行一次。"
disable-model-invocation: true
---

# 初始化 DiJiang 项目

为当前项目初始化 DiJiang 工作流状态。如果检测到旧版 `.dijiang/` 目录（含 agents/、references/、scripts/ 等子目录），执行迁移。

## 步骤

### 1. 检测现有状态

```bash
test -d .dijiang/ && echo "exists" || echo "missing"
ls .dijiang/ 2>/dev/null
```

### 2a. 新项目初始化

如果 `.dijiang/` 不存在，创建以下结构：

```
.dijiang/
├── tasks/          # 任务存储
├── memory/         # 记忆 JSONL 文件
├── spec/           # 编码规范
├── config.toml     # 项目配置
└── active_task.txt # 活跃任务指针（空文件）
```

创建 `.dijiang/config.toml`：

```toml
[project]
name = "<从 git remote 或目录名推断>"
developer = "<从 git config user.name 推断>"
```

创建空的 `CONTEXT.md`（如果不存在）。

### 2b. 旧版迁移

如果 `.dijiang/` 已存在且包含旧版子目录（agents/、references/、scripts/、workflow-templates/、benchmarks/、channels/、prd/、.runtime/、workspace/、queue.toml、.template-hashes.json 等），执行迁移：

1. **保留** `tasks/`、`memory/`、`spec/`、`config.toml`、`active_task.txt`（如果存在）
2. **移除** 旧版子目录：`agents/`、`references/`、`scripts/`、`workflow-templates/`、`benchmarks/`、`channels/`、`prd/`、`.runtime/`、`workspace/`、`queue.toml`、`.template-hashes.json`、`workflow.md`、`glossary.md`
3. **迁移 task.json 格式**：将每个 `tasks/*/task.json` 的 24 字段 Trellis 格式转换为简化格式：

```json
{
  "id": "<原 id>",
  "title": "<原 title>",
  "status": "<原 status，保持 5 态>",
  "createdAt": "<原 createdAt>",
  "completedAt": "<原 completedAt 或 null>",
  "branch": "<原 branch>",
  "notes": "<原 notes>",
  "meta": "<原 meta>"
}
```

4. **迁移 glossary**：如果存在 `.dijiang/glossary.md`，将其内容合并到 `CONTEXT.md`
5. **迁移 ADR**：如果存在 `.dijiang/decisions/`，将 ADR 文件移到 `docs/adr/`

### 3. 验证

确认以下结构完整：

```bash
test -d .dijiang/tasks && echo "tasks: ok"
test -d .dijiang/memory && echo "memory: ok"
test -d .dijiang/spec && echo "spec: ok"
test -f .dijiang/config.toml && echo "config: ok"
test -f CONTEXT.md && echo "context: ok"
```

## 配置项

`.dijiang/config.toml` 支持以下可选配置：

```toml
[project]
name = "项目名"
developer = "开发者名"

[workflow]
grill_mode = "grill-me"  # grill-me | adaptive | grill-with-doc

[tracker]
type = "local"  # local | github | linear
```

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要删除已有任务数据 | 保留 tasks/ 并迁移格式 |
| 不要删除已有记忆数据 | 保留 memory/ 中的 JSONL 文件 |
| 不要覆盖已存在的 config.toml | 读取后合并新字段 |
| 不要在迁移中丢失 spec 内容 | 保留 spec/ 目录原位 |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '7d796554-4239-45e4-a921-80bae38b608f'
  PropagateID: '7d796554-4239-45e4-a921-80bae38b608f'
  ReservedCode1: 'f91af608-ecd1-4211-bdac-540102ec74a1'
  ReservedCode2: 'f91af608-ecd1-4211-bdac-540102ec74a1'
