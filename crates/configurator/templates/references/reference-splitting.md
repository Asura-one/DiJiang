# SKILL.md 引用拆分策略

保持主 SKILL.md 文件专注和可维护，将深度内容拆分到独立引用文件。

## 什么时候拆分

主 SKILL.md 超过以下阈值时，应考虑拆分：

| 指标 | 阈值 |
|------|------|
| 行数 | >200 行 |
| 主题数 | 超过 3 个独立主题 |
| 参考深度 | 存在表格/列表/示例超过 20 行 |

## 拆分模式

```
skills/<name>/
  SKILL.md           # 核心：Outcome Contract + 流程 + Hard Rules + Gotchas
  references/        # 深度参考
    <topic-1>.md     # 主题参考 1
    <topic-2>.md     # 主题参考 2
  agents/            # 子 agent 定义（可选）
    <agent-1>.md     # 子 agent 定义
  scripts/           # 确定性验证脚本（可选）
    <check>.sh       # 验证脚本
```

## 引用方式

在主 SKILL.md 中使用相对路径引用：

```markdown
参考深度分析：`references/security-patterns.md`
```

## 好处

- 主文件保持 100-150 行，可快速浏览
- 深度内容按主题分散，读者选择性阅读
- 多个 skill 可共享同一份 reference
