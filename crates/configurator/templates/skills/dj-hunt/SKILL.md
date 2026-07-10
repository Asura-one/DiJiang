---
name: dj-hunt
description: >
  系统化排查 bug：先定位根因，再修复。尤其擅长回归和"以前好现在坏"的情况。
  Use when the user reports errors, crashes, regressions, failing tests,
  or unexpected behavior changes — anything that needs root cause investigation.
  触发词：修 bug、出错了、报错、crash、不工作、坏了、hunt、调查、排查。
---

参考规范：`.dijiang/references/decision-ladder.md`（编码前的决策阶梯）、`.dijiang/references/code-task-contract.md`（代码任务合约）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 定位根因并修复，无新 regression |
| **Done when** | 稳定复现 → 根因定位 → 修复验证 → regression 全部通过 |
| **Evidence** | 复现步骤、根因描述（特定函数/变量/逻辑行）、fix diff、验证日志 |
| **Output** | 根因分析报告 + 修复代码 diff |

# Hunt: 系统化排查 bug

先定位根因，再修复。不跳步骤、不猜原因。

参考 `diagnosing-bugs`（Matt Pocock）的 6 阶段 Bug 诊断方法。
以下阶段按顺序执行，跳过时必须显式说明理由。

> **Phase 1 是最关键的技能**——建好反馈环，bug 就 90% 定位了。

## 铁律

1. **先复现，再分析** — 不能稳定复现的问题，优先找复现条件，不猜原因。
2. **先定位，再修复** — 必须找到根因（特定函数/变量/逻辑行）再修。不"顺便修周边"。
3. **一次改一个变量** — 修复后不绿就回退，换方向，不堆叠补丁。
4. **每次修复后跑 regression** — 确认没引入新问题。

## 工作流

### Phase 1 — 建反馈环（占 >40% 时间）

**这是整场 bug hunt 最关键的技能。** 有稳定的 pass/fail 信号，bug 就 90% 定位了。

按顺序尝试以下方法建反馈环：

> **紧环**是调试超能力——2 秒确定性循环比 30 秒 flaky 循环强 10 倍。

| # | 方法 | 场景 |
|---|------|------|
| 1 | **Failing test**（单元/集成/e2e） | 有测试框架的项目 |
| 2 | **Curl / HTTP 脚本** | 服务端 bug |
| 3 | **CLI 调用 + diff 快照** | CLI 工具 |
| 4 | **Headless 浏览器脚本** | UI 渲染/交互 bug |
| 5 | **重放捕获的 trace** | 有真实请求/事件日志 |
| 6 | **临时 harness** | 最小子系统隔离验证 |
| 7 | **Property / fuzz loop** | "有时错"的间歇性 bug |
| 8 | **Bisection harness** | 回归场景，已知引入时间窗口 |
| 9 | **差分循环** | 新旧版本/两个配置对比 |
| 10 | **HITL bash 脚本** | 最后手段：人肉介入 |

**收紧反馈环**：
- 跑得更快？（缓存、跳过无关 init、缩小 scope）
- 信号更清晰？（断言精确症状，不是"没崩"）
- 更确定？（固定时间、seed RNG、隔离文件系统）

当 **无法建立任何反馈环** 时，停止并汇报。列出已尝试的方法，请求用户提供：复现环境访问、HAR/日志/核心转储、或生产临时埋点权限。

**Phase 1 完成标准**：你有一条可以命名命令的、已经跑过且变红的反馈环。

> 如果你发现自己还没建环就开始读代码猜原因——**停下来**。跳到假设是 feedback loop 技能要预防的精确失败模式。

### Phase 2 — 复现 + 最小化

- 跑反馈环，确认复现用户的精确错误场景——不是附近的另一个错误
- 确认多次复现（flaky bug：确认复现率足够高）
- 捕获精确错误信息（错误消息/输出/diff）

**最小化**：缩到最小的 still-red 场景。
- 每次减少一个输入/调用者/配置/步骤
- 每步减完重跑——只保留对失败必要的内容
- 最小化缩小后续 Phase 3 的假设空间，并成为 Phase 5 的 regression 测试

> 每个剩余元素都是 load-bearing 的——去掉任何一个就变绿了。

### Phase 3 — 假设（3-5 个）

列出 **3-5 个有排名的假设**，再逐个测试。单假设易锚定第一个想法。

每个假设必须 **可证伪**：陈述预测。

```text
假设 1：如果 <X> 是根因，那么 <改 Y> 后 bug 会消失 / <改 Z> 后 bug 会更糟
假设 2：...
```

如果无法陈述预测，这个假设是"感觉"——丢弃或打磨它。

**将排名列表展示给用户。** 用户往往有领域知识能立刻重排（"我们刚部署了 #3 的改动"），或知道哪些假设已经排除了。

### Phase 4 — 探针

每个探针映射到 Phase 3 的一个特定预测。一次只改一个变量。

工具偏好：
1. **调试器 / REPL** — 一个断点胜过十条 log
2. **定向日志** — 在区分假设的边界加 log
3. **永不"log 一切再 grep"**

**Perf 分支**：性能 regression 用日志通常是错的。改测 baseline → bisect。先测量再修。

### Phase 5 — 修复 + regression 测试

回归测试写在修复 **之前**——仅当存在正确的 seam。

- 正确的 seam：测试能复现 bug 在调用点的真实模式
- 没有正确的 seam → 本身就是一个发现：代码架构阻碍了 bug 锁定
- 如果 seam 存在：最小化 repro → 写 failing test → 看它 RED → 修代码 → 看它 GREEN → 跑 Phase 1 反馈环确认原始场景不复现

### Phase 6 — 清理 + 复盘

**完成前必须检查：**

- [ ] 原始场景不再复现（重跑 Phase 1 反馈环）
- [ ] Regression 测试通过（或无正确 seam 已记录）
- [ ] 所有调试探针已移除（grep 唯一前缀清理）
- [ ] 一次性原型已删除或移到标识的 debug 位置
- [ ] 正确的假设记录在 commit message

**然后问：什么能预防这个 bug？** 如果答案涉及架构变更（没有好的测试 seam、耦合、隐藏依赖），在修复后（不是之前）转交到对应 skill。

**修复失败计数器**：同一 bug 连续 3 次修复失败 → 停止当前方向，做 Break-Loop 报告：
- 当前假设和验证结果
- 还有哪些路径没查过
- 是否需要换工具

### 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不复现就猜原因 | 先稳定复现再分析 |
| 一次性改多个地方 | 一次改一个变量，验证再继续 |
| 修复完不跑回归 | 确认相关行为没坏 |
| 同一方向死磕 3 次以上 | 换思路，做 Break-Loop 报告 |

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）、`references/diagnose-feedback-loop.md`（Bug 诊断反馈环）、`.dijiang/references/durable-context-preflight.md`（记忆预检）。

## Hard Rules

1. 先建稳定的复现反馈环，再动代码
2. 不能说一句话根因就不改代码——每条假设必须可证伪
3. 修复前先写 regression 测试（有正确 seam 时）
4. 一次只改一个可能的原因，改完验证
5. 3 次连续修复失败 → 停止汇报
6. 每个调试探针带唯一前缀 `[DEBUG-xxx]`，复盘时清理

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 没确认复现就改 | 可能根本没找到 bug | 先建反馈环 |
| 一次改太多可能原因 | 不知道哪个修复生效 | 一次一个改动 |
| 修完不验证 | regression 遗漏 | 跑全量测试或冒烟 |
| 改代码不确认是否 root cause | 症状修了根因还在 | 先确认 root cause 再改 |

参考规范：`.dijiang/references/output-markers.md`（输出标记）。
