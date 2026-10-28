# Out of Scope: 拒绝档案

每个文件记录一个被否决的概念提案，防止重复发明——相似想法再次出现时，先读否决原因，再决定是否重新提案。

## 条目格式

每概念一个文件 `<概念名>.md`：

```markdown
# <概念名>

- **否决日期**：YYYY-MM-DD
- **提案来源**：<会话 / issue / 参考项目>
- **否决原因**：<一段话，说清为什么不进 DiJiang>
- **最强理由**：<先替被拒概念辩护到最强，再说明为何仍不采纳——防止下个会话只看到否定忘了它的价值>
- **重新打开条件**：<什么实质变化发生时值得重评>
```

## 档案索引

| 概念 | 一句话否决原因 |
|---|---|
| workflow-state-machine | ADR-005 已放弃 CLI 路线，无运行时状态机 |
| skill-retirement-dj-spec-bootstrap | 零使用孤儿：spec 目录为空、零入度、消费链断 |
| runtime-lifecycle-hooks | 纯 skill 架构无生命周期事件 |
| changeset-release | 无 npm/插件发布链路 |
| version-consistency-guard | 无多 manifest 版本声明场景 |
| multi-platform-marketplace | 私有技能库，无对外发布需求 |
| i18n-multilang-docs | DiJiang 全中文，无多语言需求 |
