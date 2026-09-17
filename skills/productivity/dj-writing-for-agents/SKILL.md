---
name: dj-writing-for-agents
description: "为 agent 写文档的规范参考：skill、AGENTS.md/CLAUDE.md、或任何 agent 通过指针到达的文档。Use when creating or editing skills, or modifying AGENTS.md or CLAUDE.md."
---

参考文档：任何 agent 消费的文档的写作——skill、`AGENTS.md` / `CLAUDE.md`、通过指针到达的文档。包装不同；写作相同：同一套杠杆让每个可预测，因为 agent 每次运行走同一**过程**而不是产出相同输出。

写 skill 时，读 `references/SKILL-MECHANICS.md` 了解 frontmatter、调用选择和路由 skill。

## 上下文指针

**上下文指针**是 agent 上下文中持有的引用：命名某些上下文外的材料并编码到达它的条件。skill 的 description 是一个；AGENTS.md 里命名一个文档的一行是同一个对象。**指针的措辞**，而不是它的目标，决定 agent 何时到达材料、多可靠。一个高价值目标配上措辞弱的指针是变异 bug：先锐化措辞，只有锐化失败才内联材料。

## 信息层级

文档中每条信息按代理何时需要它分层：

1. **始终在上下文中**（指针本身、前置词）
2. **可按需取**（指针的目标）
3. **几乎不需要**（省略或深藏）

## 前置词

一个描述/指针的**前置词**（触发词）训练模型何时到达它。前置词应：
- 覆盖触发场景的多种表达方式
- 清晰区分什么时候用它 vs 不用它
- 用领域语言，不用空泛词

## 修剪

- 一刀砍一半：去除重复、填充、背景叙事
- 指令用祈使句（"run tests"）而非描述
- 例外和反模式放表格，不放散文
- 参考文件大，SKILL.md 保持窄

## 参考

- `references/SKILL-MECHANICS.md` — skill 的 frontmatter/调用/路由机制

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '0292ee55-bf69-485d-a7d2-5f0e78fb7309'
  PropagateID: '0292ee55-bf69-485d-a7d2-5f0e78fb7309'
  ReservedCode1: 'cd0b3481-8956-404a-b8b1-b0c50e944235'
  ReservedCode2: 'cd0b3481-8956-404a-b8b1-b0c50e944235'
