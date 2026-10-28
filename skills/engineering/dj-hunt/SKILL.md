---
name: dj-hunt
description: "系统化排查 bug：先定位根因，再修复。尤其擅长回归和\"以前好现在坏\"的情况。 Use when the user reports errors, crashes, regressions, failing tests, or unexpected behavior changes — anything that needs root cause investigation. 触发词：修 bug、出错了、报错、crash、不工作、坏了、hunt、调查、排查。"
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '5a268706-89b2-40a5-b89f-5677fd9b56bf'
  PropagateID: '5a268706-89b2-40a5-b89f-5677fd9b56bf'
  ReservedCode1: 'eacf21da-a01e-4dba-a560-ce5b9beb9520'
  ReservedCode2: 'eacf21da-a01e-4dba-a560-ce5b9beb9520'
---

参考：`docs/references/decision-ladder.md`、`docs/references/code-task-contract.md`。深度细节：反馈环 10 方法 → `references/diagnose-feedback-loop.md`；置信度门禁与实战长案例（重叠原因/静默降级/生成未消费/防护路径）→ `references/hypothesis-and-cases.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 定位根因并修复，无新 regression |
| **Done when** | 稳定复现 → 根因定位 → 修复验证 → regression 全部通过 |
| **Evidence** | 复现步骤、根因描述（特定函数/变量/逻辑行）、fix diff、验证日志 |
| **Output** | 根因分析报告 + 修复代码 diff |

# Hunt: 系统化排查 bug

先定位根因，再修复。不跳步骤、不猜原因。

## Recursion Guard

你已身为 dj-hunt。直接执行排查与修复，不得再调用或派发另一个 dj-hunt。修复的回归护送按 dj-regression-guard 协议文本执行（协议引用，非派发）。

参考 `diagnosing-bugs`（Matt Pocock）6 阶段诊断法，按序执行，跳过须显式说明理由。

## 铁律

1. **先复现，再分析** — 不稳定复现的优先找复现条件，不猜原因
2. **先定位，再修复** — 找到根因（特定函数/变量/逻辑行）再修，不"顺便修周边"
3. **一次改一个变量** — 不绿就回退换方向，不堆叠补丁
4. **修复后跑 regression** — 护送走 `dj-regression-guard` 三明治协议；本 skill 定位根因+写专项，基线与回归由它承接

## 工作流

### Phase 1 — 建反馈环（占 >40% 时间，最关键）

有稳定 pass/fail 信号，bug 就 90% 定位了。紧环是调试超能力——2 秒确定性循环强过 30 秒 flaky 循环。按序尝试 10 法建环（failing test / curl / CLI+diff / headless / 重放 trace / 临时 harness / fuzz / bisection / 差分 / HITL bash，详见 references/diagnose-feedback-loop.md）。

**收紧**：更快？信号更清晰（断言精确症状）？更确定（固定时间、seed RNG）？无法建环时停止汇报，列已试方法，向用户要复现环境/HAR/日志/埋点权限。**完成标准**：可命名命令、已跑过且变红。没建环就读代码猜原因 → **停下来**。

### Phase 2 — 复现 + 最小化

跑反馈环复现用户**精确**错误场景（多次复现，flaky 确认复现率），捕获精确错误信息。**最小化**到最小 still-red：每次减一个输入/调用者/配置/步骤，每步重跑——缩小 Phase 3 假设空间，成为 Phase 5 的 regression 测试。剩余元素都是 load-bearing。

### Phase 3 — 假设（3-5 个）

列 **3-5 个有排名的可证伪假设**（"如果 <X> 是根因，<改 Y> 后 bug 会消失"），无法陈述预测的丢弃。**排名展示给用户**——领域知识能立刻重排（"刚部署了 #3"）。多候选按置信度门禁行动（≥90% 直接修附 fallback / 50-70% 先探针 / <50% 继续收证据），细则见 `references/hypothesis-and-cases.md`。

### Phase 4 — 探针

每个探针映射 Phase 3 一个特定预测，一次只改一个变量。工具偏好：调试器/REPL（一断点胜十 log）> 定向日志（加在区分假设的边界）> **永不"log 一切再 grep"**。Perf 分支：性能 regression 改用 baseline→bisect，先测量再修。

### Phase 5 — 修复 + regression 测试

任务仍 `planning` 时用原始用户请求重新走 dj-dispatch，不绕过 worktree check（见 dj-git-guardrails）。

回归测试写在修复**之前**——仅当存在正确 seam；没有 seam → 本身就是发现：架构阻碍 bug 锁定。有 seam：最小化 repro → failing test → RED → 修 → GREEN → 重跑反馈环确认原始场景不复现。遵守 **Code Task TDD Contract**（RED/Repro evidence、GREEN command、Regression scope、Exception）。

### Phase 6 — 清理 + 复盘

完成前检查：场景不复现（重跑反馈环）；regression 过（或无 seam 已记录）；探针全清（grep 唯一前缀）；一次性原型删除；正确假设进 commit message。再问"什么能预防这个 bug"——涉及架构变更（无好 seam、耦合、隐藏依赖）的，修复**后**转交相应 skill。同一 bug 连续 3 次修复失败 → 停止方向做 Break-Loop 报告（当前假设与验证/未查路径/是否换工具）。

## Hard Rules

1. 先建稳定的复现反馈环，再动代码
2. 每条假设必须可证伪
3. 修复前先写 regression 测试（有正确 seam 时）
4. 一次只改一个可能的原因，改完验证
5. 3 次连续修复失败 → 停止汇报
6. 每个调试探针带唯一前缀 `[DEBUG-xxx]`，复盘时清理

## Gotchas

| Gotcha | 预防 |
|---|---|
| 没确认复现就改 | 先建反馈环 |
| 一次改太多可能原因 | 一次一个改动 |
| 修完不验证 | 跑全量测试或冒烟 |
| 不确认 root cause 就改 | 先确认再改 |
| 存量失败未甄别 | 干净基线对比区分存量/新引入 |
| 多原因只修一个 | 重跑反馈环，列候选清单逐个验证 |
| 静默降级无标记 | fallback 显式标记，不注入 preferred 专属参数 |
| 生成字段未消费 | 审查输出字段提取代码；token 超耗优先排查 |
| 防护只在降级路径 | 预算/截断/校验全路径执行 |

后四条的实测案例详情见 `references/hypothesis-and-cases.md`。

**下一步**：修复完成且红测转绿后，完成后调用 `dj-check` 验证交付质量（回归护送已由 dj-regression-guard 三明治承接）。