---
name: dj-karpathy
description: >
  LLM 编码行为准则：减少常见错误，避免过度工程，暴露假设，定义可验证的成功标准。
  可叠加到任何其他 skill 上使用。
  Use when writing, reviewing, or refactoring code to avoid overcomplication,
  make surgical changes, surface assumptions, and define verifiable success criteria.
  触发词：karpathy、编码准则、行为规范、LLM编码、避免过度工程、surgical。
license: MIT
---

# Karpathy Guidelines

减少 LLM 编码常见错误的行为准则，源自 [Andrej Karpathy 的观察](https://x.com/karpathy/status/2015883857489522876)。

**权衡**：这些准则偏向谨慎而非速度。简单任务也必须保留可验证成功标准。

## Execution Protocol

### 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | User request, relevant code, existing tests, project style, and known constraints |
| 输出 | Stated assumptions, minimal plan, verified change, and concise delivery summary |
| 非目标 | Do not broaden scope, rewrite unrelated code, or replace the active workflow skill |

### 工作流

1. **State assumptions** → Output every assumption that affects scope, behavior, or data safety.
2. **Define success** → Convert the request into pass/fail checks: test, typecheck, lint, CLI fixture, or manual checklist.
3. **Choose the smallest design** → Prefer deletion, stdlib, existing dependency, and direct code before adding abstraction.
4. **Make surgical changes** → Edit only lines traceable to the request; clean only orphan code created by this edit.
5. **Run verification** → Start with the narrowest relevant check, then widen only when needed.
6. **Report outcome** → Say what changed, why it is enough, what was verified, and what remains unverified.

实现前使用这个格式：

```text
假设：<影响范围的假设；没有则写 none>
成功检查：<命令或人工检查>
计划：
1. <步骤> → 验证：<检查项>
2. <步骤> → 验证：<检查项>
非目标：<明确跳过的工作>
```

### 第一性原理审查

如果假设会改变产品行为，停止并提问。如果假设只影响可逆实现细节，说明后继续。

需要审查设计或重构方案时，按这个顺序拆解：

```text
1. Fundamental problem：这段代码真正解决的问题是什么？
2. Basic facts：已有代码、数据、用户约束中哪些是硬事实？
3. Hidden assumptions：实现依赖了哪些未验证假设？
4. Derived solution：从硬事实出发，当前方案是否能推导出来？
5. Simpler approach：是否存在更小、更直接、更少抽象的方案？
6. Trade-offs：保留当前方案要付出什么复杂度、风险或维护成本？
```

输出发现时必须说明：
- 哪个假设被代码依赖；
- 为什么它可能不成立；
- 第一性原理下的替代方案是什么。

## 1. 写代码前先想清楚

**不假设。不掩盖困惑。主动暴露权衡。**

开始实现前：
- 明确说出你的假设。不确定时，先问。
- 存在多种解读时，呈现这些解读——不要默默选一个。
- 有更简单的方案时，说出来，必要时推回去。
- 遇到不清楚的地方，停下来，说清楚哪里有歧义，然后问。

## 2. Simplicity First

**最少代码解决问题。不做投机性设计。**

- 不做未请求的功能
- 不为单次使用代码创建抽象
- 不为"灵活性"或"可配置性"加设计
- 不为不可能场景加错误处理
- 200 行能缩到 50 行？重写。

问自己："资深工程师会觉得这过度复杂吗？"是 → 简化。

### 何时不该简化

| 场景 | 为什么不能简化 | 示例 |
|------|----------------|------|
| 外部 API 调用 | 网络不可靠，必须处理超时/重试/错误码 | `fetch` 需要 try-catch + 状态码检查 |
| 用户输入 | 永远不可信，必须验证 | 表单需要格式校验 + 长度限制 + XSS 防护 |
| 金融/安全相关 | 错误代价高 | 支付需要幂等性 + 对账 + 审计日志 |
| 并发/异步 | 竞态条件难调试 | 需要锁/队列/状态机 |

判断标准：**如果简化后丢失的错误信息会导致用户困惑或数据丢失，就不要简化。**

## 3. Surgical Changes

**只改必须改的。只清理自己制造的垃圾。**

编辑已有代码时：
- 不"顺手改进"相邻代码、注释或格式
- 不重构没坏的东西
- 匹配已有风格，即使你会做得不同
- 发现无关死代码，提一句——但不要删

你的改动制造了孤儿（未使用的 import/变量/函数）→ 清理。
改动前就存在的死代码 → 除非被要求，否则不动。

测试：每一行变更都应能直接追溯到用户的请求。

## 4. Goal-Driven Execution

**定义成功标准。循环直到验证通过。**

把任务转化为可验证的目标：
- "加验证" → "写无效输入的测试，然后让它们通过"
- "修 bug" → "写一个复现它的测试，然后让它通过"
- "重构 X" → "确保重构前后测试都通过"

多步任务，先陈述简短计划：
```
1. [步骤] → 验证：[检查项]
2. [步骤] → 验证：[检查项]
```

明确的验证标准让你能独立循环执行；模糊标准（"让它跑起来"）则需要反复确认。

## 🔴 CHECKPOINT · 准则确认

开始实现前：
```
遵循准则：
- 写代码前先想清楚：假设已明确？[是/否]
- 优先简单方案：方案是最简？[是/否]
- 外科式修改：只改必要代码？[是/否]
- 目标驱动：成功标准已定义？[是/否]

确认开始实现？(Y/n)
```

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 不确定该简化还是保留完整 | 按场景表判断（外部API/用户输入/安全 → 不简化） | 选更保守的方案（保留完整） |
| 发现代码过度复杂但不确定怎么改 | 问自己"200行能缩到50行吗" | 标记 `ponytail:` 后继续，不阻塞 |
| 多种解读无法选择 | 列出所有解读，标注推荐 | 用推荐方案，标注假设 |
| 验证标准定义不了 | 回到需求，提取用户故事 | 从 happy path 开始，逐步加边界 |
| 匹配已有风格和自己偏好冲突 | 匹配已有风格 | 标注风格差异，不阻塞实现 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 假设用户意图就开始写 | 明确说出假设，不确定就问 |
| 200行能50行搞定却不管 | 重写到最简 |
| 顺手重构相邻代码 | 只改用户要求的部分 |
| 为"可能的扩展"预留抽象层 | 等真正需要时再提取 |
| 成功标准写"让它跑起来" | 定义具体可验证的断言 |
| 发现死代码顺手删了 | 提一句，不动它 |
