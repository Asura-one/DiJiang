---
name: dj-output
description: >
  创建和维护项目文档（PRD、设计文档等），并确保文档与代码双向对齐。
  Use when the user needs to create project docs, update docs after code changes,
  write PRD, design doc, or architecture decision record.
  触发词：写文档、PRD、设计文档、做文档、输出文档、记录决策。
dispatch_intent: >
  创建和维护项目文档（PRD、设计文档等），并确保文档与代码双向对齐。
when_to_use: 写文档、PRD、设计文档、做文档、输出文档、记录决策
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 创建/更新的项目文档（PRD、设计文档、ADR 等） |
| **Done when** | 文档创建或更新完成 + 与代码双向对齐检查完成 |
| **Evidence** | 文档文件 diff |
| **Output** | 新的/修改的文档文件 |

参考规范：`references/spec-derivation.md`（Spec 推导原则——不面试用户，只从已有对话综合）。

# Output: 创建和维护项目文档

## 职责

创建和维护 PRD、设计文档、决策记录。文档与代码双向对齐——代码改则文档改，文档改则代码改。

## 创建文档

### 1. PRD 模板

```markdown
## Problem Statement
<要解决的问题，用户视角>

## Solution
<方案概述>

## User Stories
- As a <角色>, I want <功能> so that <价值>

## Implementation Decisions
- <技术选型及其理由>

## Testing Decisions
- <测试策略>

## Out of Scope
- <明确不包含的内容>

## Further Notes
<补充信息>
```

### 2. 设计文档模板

```markdown
## 背景
<为什么需要这个设计>

## 方案
<做了什么设计决策、取舍理由>

## 影响范围
<影响的模块/接口/数据模型>
```

### 3. 对抗式审查

写完文档后，自己提问：
- 我有没有隐瞒任何不确定的地方？
- 读者能理解我为什么做了这个决策吗？
- 遗漏了什么边界场景吗？

### 4. 保存

- PRD 存到 `.dijiang/prd/` 下
- 设计文档存到 `.dijiang/design/` 下
- 命名：`<项目/功能>-<类型>-<日期>.md`

## 文档与代码对齐

- 代码改完 → 检查关联的文档是否需要更新
- 文档改完 → 标注代码中需要变更的部分
- 双方不一致时以代码为准，但必须更新文档标记已过时

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 文档写完美再给用户看 | 先写草稿，确认方向再完善 |
| 文档和代码互相矛盾 | 改了一方就改另一方 |
| 文档写得太技术 | 面向读者（PM、新人）写 |

## Hard Rules

1. 先写草稿再完善——不追求一次完美
2. 改了一方代码就改另一方的文档（或标明已过时）
3. 双方不一致时以代码为准，但必须标注文档已过时
4. 面向读者写——PM 看 PRD、开发者看设计文档、新人看 README

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 文档写完美再给用户看 | 浪费迭代时间 | 先写草稿确认方向 |
| 文档和代码互相矛盾 | 读者该信哪个 | 改了一方就改另一方 |
| 文档写得太技术 | PM/新人读不懂 | 面向目标读者写 |

参考规范：`references/doc-template.md`（文档模板标准框架）。
