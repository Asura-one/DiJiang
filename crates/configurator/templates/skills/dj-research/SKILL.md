---
name: dj-research
description: >
  系统性调研技术方案、库、竞品或代码库。阅读文档、分析代码、对比选项，产出结构化报告。
  Use when the user asks to research a topic, compare technologies, investigate a codebase,
  understand a domain, or explore options before making a decision.
  触发词：调研、研究、对比、技术选型、了解一下、research、investigate、compare。
---

# Research: 系统性调研

调研技术方案、库、竞品或代码库。产出结构化报告。

## 工作流

### 1. 明确调研目标

确认要回答的问题：
```
核心问题：<一个可回答的问题>
范围：<要覆盖的维度>
产出：<一份报告 / 对比表 / 建议>
```

### 2. 收集信息

按顺序：

- **官方文档** — 先读官网/README，建立基本认知
- **代码** — 看 API 签名、示例代码、核心实现
- **社区** — issue、讨论、博客（验证踩坑经验）
- **竞品对比** — 如果有多个选项，列对比表

### 3. 分析

- 每个选项的优势和劣势
- 与当前项目/技术栈的匹配度
- 已知的坑和局限性
- 迁移成本（如果涉及替换）

### 4. 产出

```markdown
## 调研结论

### 核心问题
<原始问题>

### 选项
| 选项 | 优势 | 劣势 | 匹配度 |
|---|---|---|---|

### 推荐
<推荐方案 + 理由>

### 风险和注意事项
<已知问题和缓解方案>
```

## 边界

- 不写代码（那是 dj-prototype 的事）
- 不替用户决策——呈现信息，给出建议
- 不确定的地方标注"需进一步确认"
