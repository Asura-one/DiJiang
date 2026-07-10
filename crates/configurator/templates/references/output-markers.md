# 输出标记（Output Markers）

为 skill 产出添加统一前缀标记，使其在对话中可快速识别。

## 标记规则

每个 `dj-*` skill 在输出其主要交付物时，以 🥷 开头：

```
🥷 <skill-name>: <deliverable summary>
```

## 示例

| Skill | 输出标记 |
|-------|----------|
| dj-dispatch | `🥷 dispatch: 路由到 dj-implement` |
| dj-grill | `🥷 grill: 需求已对齐，6 个维度确认清晰` |
| dj-implement | `🥷 implement: 功能 X 已完成，通过验证` |
| dj-hunt | `🥷 hunt: 定位到空指针异常在 src/x.rs:42` |
| dj-check | `🥷 check: 质量门禁通过` |
| dj-review | `🥷 review: 审查完成，2 个发现` |
| dj-audit | `🥷 audit: 全仓扫描完成，3 个可删除项` |
| dj-handoff | `🥷 handoff: session 交接文档已保存` |

## 什么时候不用

- 用户明确要求不用 emoji 时
- 辅助性检查输出（如 grep 结果、日志片段）
- 对话中嵌入的代码片段
