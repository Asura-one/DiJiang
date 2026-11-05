---
review_before: 2028-10-27
---

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

## 分层披露（L0/L1/L2）

拆分解决宽度（何时拆文件），分层解决深度（读多少再判断相关性）：

| 层 | 载体 | 作用 |
|----|------|------|
| L0 | frontmatter `name` + `description` | 路由可见，判断「要不要进这个技能」 |
| L1 | 文件标题下一行的 `> 摘要：` 引用块（≤100 token） | 不读全文判断「这篇 reference 相关吗」 |
| L2 | 正文全文 | 按需精读 |

纪律：**新建 references 必须带 L1 摘要头**；改写存量 references 时顺手补。摘要回答「这篇文档解决什么问题」，不是内容缩写。

## 好处

- 主文件保持 100-150 行，可快速浏览
- 深度内容按主题分散，读者选择性阅读
- 多个 skill 可共享同一份 reference
