---
name: dj-karpathy
description: >
summary: LLM 编码行为准则：减少常见错误，避免过度工程
phases: [align, implement]
risk: low
  LLM 编码行为准则：减少常见错误，避免过度工程，暴露假设，定义可验证的成功标准。
  可叠加到任何其他 skill 上使用。
  Use when writing, reviewing, or refactoring code — especially for non-trivial tasks.
  Activates automatically when writing 20+ lines or touching more than 3 files.
  触发词：karpathy、规范、纪律、行为准则、不要乱写。
---

参考规范：`.dijiang/references/decision-ladder.md`（编码前的决策阶梯）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 编码行为改进 + 减少 LLM 常见错误 |
| **Done when** | 编码/审查/重构完成，行为准则各条得到遵守 |
| **Evidence** | 代码 diff + 行为自评检查 |
| **Output** | 代码变更 + 行为准则遵循说明 |

# Karpathy: LLM 编码行为准则

可叠加到任何 skill 上。在写代码/审查/重构时激活。

## Execution Protocol

### 1. 写代码前先想清楚

- 读一遍要改的文件，理解上下文
- 在脑海里过一遍：输入是什么？处理逻辑？输出是什么？边界情况？
- 有不确定性先写注释描述意图，再写代码

### 2. Simplicity First

- 不要为未来做设计。未来不会按你预想的发生。
- 能用数组不用对象，能用函数不用类，能同步不用异步
- 怀疑任何超过 50 行的函数、超过 3 层缩进、超过 2 层嵌套的条件

### 3. Surgical Changes

- 改什么文件、改多少行、为什么改——每一行都要能回答
- 不改没读过的代码
- 不改不相关代码（即使它需要重构）

### 4. Goal-Driven Execution

- 当前任务是什么？这行代码直接服务于它吗？
- 这一轮改动完成后，用户能看到/验证什么变化？

## 自我检查

实现或修改完代码后，快速问自己：
- "这段代码能再简单一点吗？"
- "如果别人第一次看这段，能马上理解吗？"
- "有对当前任务不必要的抽象吗？"

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不改文件直接写 | 先读再改 |
| 加不必要的抽象层 | 简单直接，按需抽象 |
| 顺手改无关代码 | 只做任务指定的事 |
| 不设成功标准就写代码 | 先定义"怎样算完成" |

## Hard Rules

1. 写代码前先确定成功标准——"怎样算完成"
2. 不改文件直接写——先读再改
3. 不加不必要的抽象层
4. 不改未读过的文件

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 不读文件就改 | 改错了上下文 | 先读再改 |
| 加不必要的抽象层 | 过度工程 | 简单直接，按需抽象 |
| 顺手改无关代码 | reviewer 看不懂 | 只做任务指定的事 |
