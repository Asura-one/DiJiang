---
name: dj-memory
description: "项目管理记忆：存储发现、经验、纠正和模式。其他 skill 通过 Call the Skill tool 调用。管理 .dijiang/memory/ 下的 JSONL 文件。"
---

# 项目记忆管理

管理 `.dijiang/memory/` 下的持久记忆文件。其他 skill 完成工作后调用本 skill 记录可复用的项目知识。

## 记忆类型

| 文件 | 用途 | 格式 |
|------|------|------|
| `findings.jsonl` | 项目发现（验证过的事实） | 每行一个 JSON 对象 |
| `learnings.jsonl` | 项目经验（做法/教训） | 每行一个 JSON 对象 |
| `corrections.jsonl` | 用户纠正（改变未来行为） | 每行一个 JSON 对象 |
| `patterns.jsonl` | 工作流模式/标准操作流程 | 每行一个 JSON 对象 |
| `sessions.jsonl` | session 闭环记录 | 每行一个 JSON 对象 |

## 操作

### 存储发现 (store-finding)

```json
{
  "type": "finding",
  "content": "<发现内容>",
  "source": "<来源：task|user|investigation>",
  "scope": "<范围：project|task|module>",
  "confidence": "verified|probable|speculative",
  "createdAt": "<ISO 8601 时间>"
}
```

写入 `.dijiang/memory/findings.jsonl`（追加一行）。

### 存储经验 (store-learning)

```json
{
  "type": "learning",
  "content": "<经验内容>",
  "context": "<适用上下文>",
  "actionability": "<可执行描述>",
  "createdAt": "<ISO 8601 时间>"
}
```

写入 `.dijiang/memory/learnings.jsonl`（追加一行）。

### 存储纠正 (store-correction)

```json
{
  "type": "correction",
  "content": "<纠正内容>",
  "lesson": "<教训>",
  "actionability": "<未来行为调整>",
  "scope": "<适用范围>",
  "confidence": "high|medium|low",
  "createdAt": "<ISO 8601 时间>"
}
```

写入 `.dijiang/memory/corrections.jsonl`（追加一行）。

### 回忆 (recall)

读取 `.dijiang/memory/` 下的 JSONL 文件，按关键词或类型检索相关记忆。返回匹配的记忆条目。

### 存储 session 闭环 (store-session)

```json
{
  "type": "session",
  "task": "<任务名>",
  "verification": "<验证摘要>",
  "versionImpact": "<major|minor|patch|none>",
  "status": "completed|archived",
  "createdAt": "<ISO 8601 时间>"
}
```

写入 `.dijiang/memory/sessions.jsonl`（追加一行）。

## 质量门禁

写入前检查：
- **content** 不为空，不模糊（"fixed bug" 不合格）
- **source** 必须明确（不能是 "guess"）
- **confidence** 必须诚实标注
- 未通过门禁的记忆保留在任务产物中，不写入持久记忆

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要写 "fixed bug" | 写具体发现：什么 bug、什么原因、什么修复 |
| 不要写猜测 | 标注 confidence 为 speculative 或不写入 |
| 不要写入不可执行的经验 | 经验必须含 actionability |
| 不要让记忆阻塞工作 | 记忆失败时继续工作，在 handoff 中说明 |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'e93f7bc8-1c6a-4587-b470-53a9b1a9759e'
  PropagateID: 'e93f7bc8-1c6a-4587-b470-53a9b1a9759e'
  ReservedCode1: 'a31b7aaf-41c4-49e4-ba01-88aafbeabf06'
  ReservedCode2: 'a31b7aaf-41c4-49e4-ba01-88aafbeabf06'
