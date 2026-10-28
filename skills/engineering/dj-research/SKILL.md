---
name: dj-research
description: "技术调研与信息收集。在任务规划或实现期间查找外部信息、比较方案、验证技术可行性。 Use when investigating third-party libraries, comparing approaches, or exploring technical unknowns. 触发词：调研、研究、比较、对比、可行性、research、investigate、figure out。"
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 结构化的调研产出，记录在 `{TASK_DIR}/research/` |
| **Done when** | 调研问题已回答，发现已写入 research/，剩余不确定性已记录 |
| **Evidence** | research/ 下的文件 |
| **Output** | 每专题一个文件，记录发现、来源、权衡和未决项 |

## Research 流程

### 1. 定义调研问题

明确要回答的 1-3 个具体问题。一次调研回答一组紧密相关的问题，不一次调研整个任务。

### 2. 收集信息

使用可用工具查找信息：

- 代码库内搜索（grep、compose）——先用自己代码
- 外部搜索（web_search）——库文档、最佳实践、社区讨论
- 官方文档（fetch_content）——API 参考、示例
- 项目 history（git log）——历史决策和变更

### 3. 记录发现

每专题一个文件，写入 `{TASK_DIR}/research/<topic>.md`：

- **Context** — 为什么调研这个问题
- **Findings** — 按来源或方案组织的发现
- **Comparison** — 多方案时的对比表
- **Conclusion** — 推荐方案及理由
- **Open questions** — 调研后仍不确定的事项

### 4. 关联回任务

调研结论影响 `prd.md`、`design.md` 或 `implement.md` 时，用 `dj-output` 更新对应文档。调研本身不直接修改这些文档。

## 调研约定

- 一个调研专题一个文件，命名：`research/<topic-slug>.md`（英文）
- 外部引用标明来源 URL 和获取时间
- 记录库/工具的版本号和已知兼容性约束
- 多方案对比必须包含每个方案的取舍，不能只说最终选择

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 外部源不可达 | 换镜像/缓存/官方文档 | 标「源不可达」，用可得信息先答 |
| 多源结论冲突 | 列出分歧点与各自依据 | 标「结论冲突」，建议小规模验证 |
| 调研范围失控 | 收窄到 1-3 个具体问题 | 暂停，交用户确认范围 |

## 边界

- 调研不修改代码或 task artifacts（prd.md、design.md、implement.md）
- 不确定时标注「需要验证」，不把假设写入结论
- 调研期间发现必须修复的 bug → 关联回任务，不在调研中修复

## Hard Rules

1. 调研产出必须写入 `{TASK_DIR}/research/`，不能只留在对话
2. 每个调研主题独立文件，不合并到 prd.md 中
3. 外部信息标明来源 URL
4. 不确定的结论标注「待验证」
5. 调研期间发现 bug 只记录不修复
