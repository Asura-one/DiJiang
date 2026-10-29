# 新技能模板（创建时复制此骨架）

> 从 dj-meta SKILL.md 外溢（DJ-012，meta 自身超 3000B 身先士卒）。步骤见 dj-meta 正文「治理门槛」；填完回 SCOPE.md 双门槛核验。

## 创建新技能的模板

```markdown
---
name: dj-<name>
description: >
  <一句话说清楚做什么>
  Use when <触发条件，agent 判断依据>
  触发词：<用户说的词，帮助 dj-ask 路由>
---

# <Title>: <中文描述>

## 职责

<一段话说明这个 skill 负责什么、不负责什么>

## 工作流

### 1. <步骤 1>

<指令>

### 2. <步骤 2>

<指令>

### 3. <步骤 3>

<指令>

## 边界

- <不做的事>

## 使用提醒

- 触发词写在 description 内，供 dj-ask 路由——两两 Jaccard >= 0.5 会被 verify FAIL
- 面向 agent 写（上下文指针/前置词纪律见 dj-meta「为 agent 写文档」节）
- 引用带触发条件，深度 <= 1
