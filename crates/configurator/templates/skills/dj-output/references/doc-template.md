# 文档模板规范

技能文档页（用于 `.dijiang/spec/` 或外部文档）使用固定的 5 段框架。

## 模板结构

```
# <Title>: <Subtitle>

## Quickstart
一句最简使用说明。

## 它做什么
一至两段描述这个文档/技能的行为。不展开实现细节。

## 什么时候用它
- 触发条件列表（用户场景）
- 前置条件（什么需要先准备好）

## 它适合哪里
- 与周边组件的关系
- 能力边界（什么不做）
- 与类似功能的区别

## 扩展阅读
- 相关文档/规范链接
```

## 约束

- 每段不超过 4 句
- 先写草稿再完善——不追求一次完美
- 面向目标读者（PM、开发者、贡献者）写，不同读者不同深度
- 不翻译术语（API、hook、skill、workflow 保持原文）

## 示例：规范页面

```markdown
# Spec 推导原则: 不面试用户

## Quickstart
写 PRD 或 design doc 时，从已有对话综合出需求，不追问用户。

## 它做什么
将 grill 阶段的讨论内容、issue 描述、用户初始请求综合为结构化 spec。
不做需求发现——那是 dj-grill 的职责。

## 什么时候用它
- 从 dj-grill 转向实现时
- 写 PRD / design doc 时
- 拆票（to-tickets）前

## 它适合哪里
- 配合 dj-grill（需求对齐）→ 本规范（spec 综合）→ dj-implement（实现）
- 不适合：用户明确要求"帮我写 PRD，我告诉你内容"

## 扩展阅读
- `.dijiang/spec/guides/spec-derivation.md`
- `dj-output/SKILL.md`
```
