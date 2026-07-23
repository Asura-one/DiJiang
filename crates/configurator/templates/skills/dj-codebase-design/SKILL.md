---
name: dj-codebase-design
description: >
  代码结构设计：决定代码放在哪、模块如何划分、接口如何定义。"先设计两次"——设计一个方案，扔掉，再设计一个更好的。
  Use when planning a new module, refactoring existing code, or deciding where to put new functionality.
  触发词：结构设计、放在哪、模块划分、架构设计、codebase design、design twice。
summary: 代码结构设计：决定模块划分、接口定义
phases: [align]
risk: low
  代码结构设计：决定代码放在哪、模块如何划分、接口如何定义。"先设计两次"——设计一个方案，扔掉，再设计一个更好的。
  Use when planning a new module, refactoring existing code, or deciding where to put new functionality.
  触发词：结构设计、放在哪、模块划分、架构设计、codebase design、design twice。
---

# Codebase Design: 代码结构设计

决定代码放在哪、模块如何划分、接口如何定义。核心原则：**先设计两次**。

## 工作流

### 1. 理解上下文

- 要解决的问题/功能是什么？
- 现有的代码结构是怎样的？
- 哪些模块/文件会受影响？

### 2. 设计两次（Design It Twice）

先设计第一个方案——不管多粗糙。然后**扔掉它**，从零开始设计第二个。

第一个方案的价值不是实现，是让你发现隐藏的假设和约束。第二个方案才是你要用的。

### 3. 评估方案

```
方案 A（第一次设计）：
- 核心思路
- 为什么放弃
- 发现了什么约束

方案 B（最终方案）：
- 核心思路
- 模块划分
- 接口定义
- 文件放置
- 与现有结构的关系
```

### 4. 输出

```markdown
## 设计方案

### 结构
<新增/改动的模块和文件>

### 接口
<对外暴露的接口签名>

### 数据流
<数据如何流转>

### 影响范围
<影响的现有模块和迁移路径>
```

## 配套

参考文件 `references/deepening.md` 查看设计深化实践。
参考文件 `references/DESIGN-IT-TWICE.md` 查看"设计两次"详细指南。

## 边界

- 不写实现代码（那是 dj-implement 的事）
- 不做性能基准测试（那是 dj-prototype 的事）
- 设计文档存到 `.dijiang/design/`
