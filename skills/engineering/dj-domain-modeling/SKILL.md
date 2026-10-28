---
name: dj-domain-modeling
description: "统一语言（Ubiquitous Language）：检查代码中的术语一致性，标记工程师术语 vs 业务术语的差异，更新共享术语表。 Use when terminology feels inconsistent, when engineers and domain experts use different words for the same concept, or during a new feature that touches core domain concepts. 触发词：术语、统一语言、领域建模、ubiquitous language、术语不一致。"
---

# Domain Modeling: 统一语言

检查代码中的术语一致性，标记工程师术语 vs 业务术语的差异，更新共享术语表。

## 工作流

### 1. 收集术语

从这些来源提取领域术语：
- 代码中的类型/类/函数名
- PRD / issue 描述
- 对话中用户使用的词汇
- `CONTEXT.md` 已有条目

### 2. 检查不一致

标记这些模式：

| 模式 | 示例 | 风险 |
|---|---|---|
| 同义异构 | Order 和 Purchase 混用 | 新人困惑 |
| 工程师术语 | processOrderItem | 业务说的是"下单" |
| 过时术语 | old 代码中的 legacy 命名 | 新人不明白 |
| 模糊术语 | status 太笼统 | 不知道有哪些值 |

### 3. 输出

```markdown
## 术语审计报告

### 发现的差异
| 术语 | 代码中 | 业务语言 | 建议 |
|---|---|---|---|

### 更新 Glossary
- 新增/更新条目
```

### 4. 更新 glossary

将确认的术语更新到 `CONTEXT.md`。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 术语冲突无法裁决 | 列双方定义与用例，交用户/领域专家定 | 标注「待裁决」，不擅自择一 |
| 业务方缺席、术语只存在于代码 | 以代码命名暂记，标记待业务确认 | 归入「待确认」，不进权威 Glossary |
| 术语随版本变化 | 记录变更时间与新旧对应 | 旧术语标 deprecated，不直接删除 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 未确认就自动重命名代码 | 只报告差异，人工确认后再改 |
| 只收集术语不产出差异报告 | 输出审计表 + Glossary 更新 |
| 把实现细节混入术语定义 | 术语表只记规范术语与定义 |
| 模糊术语（如 status）不澄清取值 | 列出可能取值再定义 |

## 配套

参考文件 `references/adr-format.md` 查看 ADR 格式。
参考文件 `references/context-format.md` 查看 CONTEXT 格式。

## 边界

- 不改代码——只报告术语问题
- 不自动重命名——需要人工确认后再改
