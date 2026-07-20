---
name: dj-domain-modeling
description: >
summary: 统一语言：检查术语一致性，更新共享术语表
phases: [align]
risk: low
  统一语言（Ubiquitous Language）：检查代码中的术语一致性，标记工程师术语 vs 业务术语的差异，更新共享术语表。
  Use when terminology feels inconsistent, when engineers and domain experts use different words for the same concept,
  or during a new feature that touches core domain concepts.
  触发词：术语、统一语言、领域建模、ubiquitous language、术语不一致。
---

# Domain Modeling: 统一语言

检查代码中的术语一致性，标记工程师术语 vs 业务术语的差异，更新共享术语表。

## 工作流

### 1. 收集术语

从这些来源提取领域术语：
- 代码中的类型/类/函数名
- PRD / issue 描述
- 对话中用户使用的词汇
- `.dijiang/glossary.md` 已有条目

### 2. 检查不一致

标记这些模式：

| 模式 | 示例 | 风险 |
|---|---|---|
| 同义异构 | Order 和 Purchase 混用 | 新人困惑 |
| 工程师术语 | processOrderItem | 业务说的是"下单" |
| 过时术语 | old 代码中的 legacy 命名 | 新人不明白 |
| 模糊术语 | status 太笼统 | 不知道有哪些值 |

### 3. 输出

```markdown
## 术语审计报告

### 发现的差异
| 术语 | 代码中 | 业务语言 | 建议 |
|---|---|---|---|

### 更新 Glossary
- 新增/更新条目
```

### 4. 更新 glossary

将确认的术语更新到 `.dijiang/glossary.md`。

## 配套

参考文件 `references/adr-format.md` 查看 ADR 格式。
参考文件 `references/context-format.md` 查看 CONTEXT 格式。

## 边界

- 不改代码——只报告术语问题
- 不自动重命名——需要人工确认后再改
