---
name: dj-wayfinder
description: "规划一个大块工作（超过一个 agent session 能容纳的）为一张共享的决策票据地图，一次解决一个票据，直到通往目的地的路清晰。"
disable-model-invocation: true
---

一个模糊想法到来，太大，一个 agent session 装不下，被迷雾包裹。Wayfinding 就是找到那条路。本 skill 将路绘制为**共享地图**，然后逐张解决**决策票据**（决策而非构建切片的票据），直到路径清晰。

## Plan, don't do

Wayfinder 默认是**规划**：每张票据解决一个决策，地图在路清晰时完成。往往"直接去做"的信号就是到达地图边缘，该交接了。

## 地图（本地文件实现）

地图是一份 Markdown 文件：`.dijiang/plans/<name>/map.md`，是唯一权威产物。它的子票据是同目录下的子文件。

### 地图正文

```markdown
## Destination

<到达地图终点是什么样：要产出的 spec、决策或变更。一两行；每个 session 在选票据前先对齐。>

## Notes

<领域；每个 session 都应查阅的 skill；本次努力的首选项>

## Decisions so far

<!-- 索引：每张已关闭票据一行，足够判断相关性，链接详情 -->

- [<已关闭票据标题>](链接): <答案一句话>

## Not yet specified

<!-- 战争中迷雾：范围内但还不能票据化的雾；前沿推进时毕业 -->

## Out of scope

<!-- 明确排除在目的地之外的工作；关闭后不再毕业 -->
```

### 票据

每张票据是一个文件，正文是问题，大小不超过一个 session 能处理的量：

```markdown
## Question

<此票据要解决的决策或调查>
```

每张票据带 `wayfinder:<type>` 标签，类型如下。

**认领**：session 开工前先把票据标记为"进行中"（记录在票据文件顶部），避免并发 session 重复处理。

**前沿**：所有依赖票据已关闭的票据中，开放且未认领的票据——已知的边缘。

## 票据类型

- **Research（AFK）**: 阅读文档、第三方 API 或本地资源来呈现一个决策等待的事实。由调用 `dj-research` 的子 agent 解决。
- **Prototype（HITL）**: 通过做便宜、粗糙、具体的产物（草稿、占位、UI/逻辑代码）提升讨论保真度。调用 `dj-prototype`。用 "它应该长什么样" 或 "它应该怎么表现" 是关键问题时。
- **Grilling（HITL）**: 对话。默认。调用 `dj-grill` 和 `dj-domain-modeling`。
- **Task（HITL 或 AFK）**: 必须在决策前发生的动手工作。手动干，agent 不能代替。

## 迷雾

地图**刻意不完整**。活的票据之外是迷雾：你看到将来但还钉不下来的决策和调查，因为它们挂在仍开放的问答上。解析票据会清掉其前方的雾。

**雾或票据？** 测试标准是能否现在精确陈述问题，而不是能否现在回答。

- 问题已经精确 → 票据（即使阻塞不能行动）
- 还不能那么精确地表述 → Not yet specified

**Not yet specified** 排除已决定的（Decisions so far）、已活票据、和范围外（下节）。

## Out of scope

雾只朝**目的地**聚。目的地固定了范围，范围外的工作不是雾，不属于 **Not yet specified**。范围（而非锐度）把它放到 **Out of scope** 节。范围外的工作从不毕业；只有目的地重绘才回来，且作为全新努力，不是恢复。

## 调用

两种模式。无论如何，**每个 session 至多解决一张票据**（research 票据除外）。

### 绘制地图

用户用一个模糊想法调用。

1. **命名目的地**。调用 `dj-grill` 和 `dj-domain-modeling` 确认地图在找什么。目的地先定，因为它钉死范围。
2. **绘制前沿**。再 grill，这次**广度优先**。如果没有雾（路已清楚、工作小到能放进一个 session），你不需要地图。停下来问用户怎么继续。
3. **创建地图**。Destination 和 Notes 填好，Decisions-so-far 空，雾写到 **Not yet specified**。
4. **创建能确定的票据**。第二遍连依赖。
5. 停止：绘图是一个 session 的工作；它不手动作。

### 逐张解决

用户带着地图调用。

1. 加载**地图**（低清视图，不读每张票据正文）。
2. 选票据：用户命名就选它；否则按顺序取第一张前沿票据。**认领它**。
3. 解决它。**需要时放大**：按需取相关票据全文；调用 `## Notes` 命名的 skill。不确定就调用 `dj-grill` 和 `dj-domain-modeling`。
4. 记录解决：在票据上写**解决注释**、**关闭**票据、往 Decisions-so-far **追加**一行。
5. 添加新浮出的票据；把可确定的雾毕业后从 **Not yet specified** 清掉。如果答案显示某票据超出了目的地，**判它 out of scope** 而不是在路线上解决。

## 参考

- `dj-domain-modeling` 提供领域语言和 ADR 纪律
- `dj-split` 可将已清晰的工作拆为实施票据

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '667c1dc7-73c6-4f53-ac00-d780b999e2fa'
  PropagateID: '667c1dc7-73c6-4f53-ac00-d780b999e2fa'
  ReservedCode1: 'ad76d93e-e09c-4257-b774-318abbd81a22'
  ReservedCode2: 'ad76d93e-e09c-4257-b774-318abbd81a22'
