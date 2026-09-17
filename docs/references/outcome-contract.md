# Outcome Contract 模式

每个 `dj-*` skill 必须明确定义其交付物和完成条件。这是 skill 合约的头部块，放在 skill 描述之后、具体流程之前。

## 结构

```markdown
## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 这个 skill 做完后交付的最终成果 |
| **Done when** | 判断完成的验收条件 |
| **Evidence** | 证明完成的产生物（文件、报告、日志等） |
| **Output** | 对外暴露的产出格式 |
```

## 设计原则

- **Outcome** 是用户视角的"我得到了什么"，不是"我做了什么步骤"
- **Done when** 是可验证的，不是模糊的"做完了"
- **Evidence** 是具体的文件路径、日志条目、测试结果
- **Output** 是接下来可能会作为输入的格式说明
