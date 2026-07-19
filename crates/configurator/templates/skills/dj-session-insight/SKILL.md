---
name: dj-session-insight
description: >
  跨会话记忆检索。通过 `dijiang mem recall` 检索过去的对话历史、findings 和 learnings。
  当用户问"上次怎么解的"、"之前讨论过吗"、"决定是什么"、"我想想上次的对话"、"记得那条坑吗"，
  或需要跨会话补全上下文时使用。返回原始记忆片段；根据情景决定是否更新 spec、追加到 task notes、
  在回答中引用、或仅内化。
---

# dj-session-insight

本 skill 教会 AI **如何调用 `dijiang mem`** —— DiJiang 的跨会话记忆检索系统，以及**何时**使用它。

这是一个**能力型 skill，不是工作流**。无固定产出文件，无需强制写入步骤。`mem` 返回什么由你在对话中自行判断。

## `dijiang mem` 是什么

一个本地 CLI，检索当前项目的记忆存储（findings、learnings、patterns），以及跨平台的 AI 会话历史。
所有读取在本机完成，不上传任何数据。

## 何时使用

判断标准是"一个资深队友会问'我们不是已经讨论过这个了吗？'"时的那些时刻：

| 场景 | 触发线索 |
|------|---------|
| **脑暴重复风险** | 开始一个新 task 时涉及之前讨论过的领域，先检查是否有既得决定 |
| **熟悉 bug 调试** | 当前的 bug 模式感觉像用户之前报告/修复过的 |
| **跨会话续接** | 用户间隔了一段时间后说"继续"、"where were we" |
| **决策检索** | 用户提到"关于 X 的决定"但该决定在旧对话中，不在任何 prd.md/spec/ 里 |
| **finish-work 回顾** | 用户明确要求总结 task 期间的决定/痛点/意外 |
| **模式发现** | 用户问"我是不是一直在踩同一个坑" |

如果以上场景都不存在，不要调用 `mem`。它是个工具，不是仪式。

## 何时不用

- 当前上下文（已打开的文件、当前对话、task.json/prd.md）已有足够信息
- 用户问的是代码事实而非对话历史 —— `grep`/`git log`/读文件更快更权威
- 用户明确说"不要翻历史，直接回答"

## 怎么用它

```bash
# 通过关键词召回项目记忆（findings/learnings/patterns）
dijiang mem recall <关键词>

# 列出过往会话
dijiang mem list --project

# 列出可用 patterns
dijiang mem patterns

# 查看记忆统计
dijiang mem stats
```

## 返回后的处理

`dijiang mem recall` 返回的原始记忆，根据当前对话自行判断：

- **在回答中引用** —— 如果具体过往交换回答了当前问题，摘录并注明来源
- **更新 `<task>/prd.md` 或 `<task>/design.md`** —— 如果记忆暴露了一个应该被记录的承重决策
- **追加到 task notes** —— 如果发现属于当前 task 的记录但不适合 PRD
- **更新 `.dijiang/spec/`** —— 如果发现是项目级约定或 gotcha
- **仅内化** —— 用于一次性的回忆，不写入任何文件

Trellis 不预设单一目的地。把每次召回强制写入固定的文件会让文件变成噪音。

## 范围外

- `mem` 不编辑代码和文件。任何写入行为是你基于现场判断的决定。
- `mem` 是只读的。不会推送或同步到远程。
- 本 skill 不替代 `.dijiang/spec/` 更新或平台原生的 task/workflow 流程。
