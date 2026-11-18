---
name: dj-regression-guard
name_cn: 回归守卫
description_cn: 改码三明治回归协议：改前基线→修改→专项验证→改后回归护送，防回归、区分本来就坏与我改坏的。
description: "改码三明治回归协议：对每一次代码修改任务执行『改前基线 → 修改 → 专项验证 → 改后回归』护送，防止改码引入回归、区分『本来就坏』与『我改坏的』。 触发词：回归保护、改前基线、三明治、回归守卫、regression guard、baseline、改码护送。"
---

参考：`skills/engineering/dj-implement/references/code-task-contract.md`、`skills/engineering/dj-ponytail/references/decision-ladder.md`、`skills/engineering/dj-check/references/intensity-levels.md`；协议各步深度细节 → `references/protocol-details.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 改码任务被三明治护送：基线、专项验证、回归、归因结论 |
| **Done when** | 基线快照 → 专项通过 → 回归无新增失败（或已按 S4 处置） |
| **Evidence** | `.temp/regression-baseline-<ts>.json`、专项结果、回归 diff |
| **Output** | 回归保护报告 |

# Regression Guard: 改码三明治回归协议

对每次代码修改任务执行**改前基线 → 修改 → 专项验证 → 改后回归**护送——新改动生效与否、旧功能是否被破坏，都有测试背书，不靠"没报错"直觉。

## Recursion Guard

你已身为 dj-regression-guard。直接执行协议步骤，不得再调用或派发另一个 dj-regression-guard。全量模式转交 dj-fullstack-testing 属边界转移不是递归。

本技能是**任务级轻量协议**；全量回归由 `dj-fullstack-testing` 承担，本技能只引用其机制、不复制。

## 模式路由

| 场景 | 判定 | 走向 |
|------|------|------|
| 改码任务需要改前后回归保护 | 三明治模式 | 本技能按 S0–S6 执行 |
| 全面/系统/全量回归测试 | 全量模式 | 转交 `dj-fullstack-testing` |
| 仅讨论测试方案，不涉改码 | 不适用 | 直接回答 |

拿不准选三明治——它便宜；全量成本由 `dj-fullstack-testing` 流程决定，不打折执行。

## 协议流程

### S0 · 快速层定义（每项目首次付一次成本）

"快速层"＝改前/改后双跑测试子集（<3 分钟），选法与超时降级 → `references/protocol-details.md`；首次接入建 `CONSTRAINTS.md` → `references/quality-contract.md`。

### S1 · 改前基线

记 commit-id → 快速层快照落盘 `.temp/regression-baseline-<ts>.json` → **允许基线红**（增量对比）→ 生命周期=本任务；细节 → `references/protocol-details.md`

**🔴 🛑 STOP（冒烟全红）**：快速层全红（连冒烟都不过）→ 停止协议报用户：先处理阻断问题，改码无意义。

### 改码执行（协议包住 TDD，不替代）

TDD 红测兼任专项测试；粒度=逻辑任务级（细节 → `references/protocol-details.md`）

### S2 · 专项验证（先行，便宜、归因清晰）

改码完成后**先跑专项再跑回归**：红测转绿=改动生效，专项失败回改码不跑 S3；两类失败不得混报 → `references/protocol-details.md`

**S2.5 · JiT 探针（条件触发）**：diff 触及无覆盖新行为 → 生成探针；测试锁 bug 用 `scripts/regression_check.py`（基线红+候选绿才算锁定）→ `references/jit-probes.md`

### S3 · 改后回归

校验 commit-id（过期重跑 S1）→ 逐项 diff（已知失败保持红不算回归，**新增失败 → S4**）→ 门禁自检 `scripts/floor_guard.py`（exit 2 ≠ 通过）→ `references/protocol-details.md`

### S4 · 新增失败处置（按序，失败才进下一步）

1. **基线 diff 滤噪**：不在已知失败清单即存量；stash 甄别与 flaky 判定 → `references/protocol-details.md`
2. **flaky 重试 ×1**：单独重跑，过则标记 flaky 放行；重跑只用于定性、不算通过
3. **自动修复 ≤ 2 轮**：定位修复，重跑 S3
4. **🛑 STOP · 回滚+报告**：回滚修改，保留失败现场，报告"改动导致回归已回滚"。**回滚后等用户裁决，不得自行重新改码**

**铁律：禁止改/删/跳失败的测试让回归变绿。** 测试自身问题（断言过时、环境变化）必须显式标记说明，交用户裁决。

### S5 · 全量层建议（🛑 决定权在用户）

波及面大时主动建议全量（P0 判据与话术 → `references/protocol-details.md`）。🛑 **用户未答复前不得自行启动**；同意 → 转交 `dj-fullstack-testing`。

### S6 · 测试债台账

欠账记 `docs/testing/test-debt.md`（格式与规则 → `references/protocol-details.md`）

## Hard Rules

1. 改码任务默认触发三明治，S0–S6 不可跳步（跳过须显式说明）
2. 改前必须记基线；无基线不得声称"回归无新增"
3. 禁止改/删/跳测试让回归变绿——显式标记交用户裁决
4. 专项未通过不进 S3；S3 新增失败必走 S4
5. S5 全量层：用户未确认绝不自行启动
6. 回滚后等用户裁决，不自行重新改码

## Gotchas

| Gotcha | 预防 |
|--------|------|
| 基线过期还跑回归 | S3 先校验 commit-id |
| 基线全红仍改码 | S1 冒烟全红先 STOP |
