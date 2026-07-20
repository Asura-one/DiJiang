---
name: dj-tdd
description: >
  测试驱动开发：红绿重构循环，一次一个垂直切片。
  Use when the user wants to build features or fix bugs test-first,
  mentions TDD, test-driven, red-green-refactor, or vertical slice.
  触发词：TDD、测试驱动、红绿重构、先写测试。
summary: 测试驱动实现与行为回归保护
dispatch_intent: >
  测试驱动开发：红绿重构循环，一次一个垂直切片。
when_to_use: TDD、测试驱动、红绿重构、先写测试
phases: [implement]
risk: medium
---

参考规范：`.dijiang/references/decision-ladder.md`（编码前的决策阶梯）、`.dijiang/references/code-task-contract.md`（代码任务合约）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 通过测试验证的代码增量（一次一个垂直切片） |
| **Done when** | RED → GREEN → REFACTOR 循环完成，所有测试通过 |
| **Evidence** | 测试结果、代码覆盖率报告 |
| **Output** | 新增/修改的测试 + 实现代码 diff |

# TDD: 测试驱动开发

完成测试-代码-重构循环，一次一个垂直切片。不一次性实现整个功能，每轮只解决一个行为。

## 工作流

### 1. 切片（确定本轮 RED）

选择一个最小的、可独立验证的行为增量。从用户视角出发：这个切片是否有独立价值？

```text
切片：<本轮要新增或修复的行为>
验收：<这个切片通过时，什么命令或断言会绿>
```

### 2. RED — 写一个失败的测试

- 只写测试接口描述的行为，不涉及内部实现。
- 运行测试，确认它失败（且失败原因是你预期的）。
- 失败信息应清晰表明"这个行为还没实现"。

### 3. GREEN — 写可通过的最小代码

- 只写让测试变绿的代码，不多写一行。
- 可以写硬编码返回值、可以写临时实现——GREEN 阶段不追求优雅。
- 跑测试确认全绿。

### 4. REFACTOR — 重构，保持 GREEN

- 消除上一步的临时实现和重复。
- 不新增行为——测试全绿是重构的前提。
- 抽离共享逻辑，但不要提前抽象。

### 5. 进入下一个切片

回到步骤 1，直到所有行为完成。

参考 `references/mocking.md`（Mock 策略）和 `references/testing-patterns.md`（测试组织）。

## 好测试 vs 坏测试

- **好测试**：测行为不测实现；失败时有明确含义；与代码变化同寿命
- **坏测试**：测私有方法；测框架交互；过于脆弱；测试之间互相依赖

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| RED 阶段不跑测试就写实现 | 先确认测试真的失败（且原因对） |
| 一轮实现太多行为 | 一个切片只解决一个行为 |
| GREEN 后不重构 | 重构是 TDD 的三分之一 |
| 测内部实现 | 测公共接口的行为 |

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. RED 阶段必须先确认测试真的失败，且失败原因正确
2. 一轮循环只解决一个行为（一个垂直切片）
3. GREEN 后必须重构——重构是 TDD 的三分之一
4. 不测私有方法，不测框架交互
5. 切片选择：从用户可感知的价值出发

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| RED 阶段不跑测试就写实现 | 测试可能永远不会红 | 先确认测试失败（且原因对） |
| 一轮实现太多行为 | 一个失败分不清哪里错 | 一个切片只解决一个行为 |
| GREEN 后不重构 | 技术债堆积 | 重构是 TDD 的三分之一 |
| 测内部实现 | 重构时测试全部要重写 | 测公共接口的行为 |
