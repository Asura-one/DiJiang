---
name: dj-regression-guard
name_cn: 回归守卫
description_cn: 改码三明治回归协议：改前基线→修改→专项验证→改后回归护送，防回归、区分本来就坏与我改坏的。
description: "改码三明治回归协议：对每一次代码修改任务执行『改前基线 → 修改 → 专项验证 → 改后回归』护送，防止改码引入回归、区分『本来就坏』与『我改坏的』。 触发词：回归保护、改前基线、三明治、回归守卫、regression guard、baseline、改码护送。"
---

参考：`skills/engineering/dj-implement/references/code-task-contract.md`、`skills/engineering/dj-ponytail/references/decision-ladder.md`、`skills/engineering/dj-check/references/intensity-levels.md`。
深度细节：零基建退路、S5 全量触发条件、技能边界表、任务报告要素、反例全表 → `references/protocol-details.md`。

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

"快速层" = 改前/改后双跑的测试子集，目标 < 3 分钟。按序判定：项目有冒烟标记（`@pytest.mark.smoke` 等）→ 跑标记子集；无标记 → 按 `dj-fullstack-testing` 0.1a 映射表圈受影响模块测试 + 3a 冒烟最小集；无测试基建 → 走零基建退路（见 references）。选法持久化到 `docs/project-understanding.md`"快速层定义"节，基础设施文件变更时重判。首次接入的项目同时建 `CONSTRAINTS.md` 质量契约（四列表，见 references/quality-contract.md）。

**超时降级**：S1 执行时快速层超 3 分钟 → 收窄为"冒烟最小集 + 直接触及模块测试"，记入基线快照（S3 跑同一子集保可比）；仍超时则保留并标注建议拆分，不阻塞。

### S1 · 改前基线

1. `git log -1 --format=%H` 记 commit-id
2. 跑快速层，快照落盘 `.temp/regression-baseline-<时间戳>.json`（commit-id、清单、逐项 PASS/FAIL、耗时）
3. **允许基线红**：已有失败记入"已知基线失败清单"，不阻断改码；比对用**增量对比**——只关心改后新增失败
4. 快照生命周期 = 本任务，结束即删

**🔴 🛑 STOP（冒烟全红）**：快速层全红（连冒烟都不过）→ 停止协议报用户：先处理阻断问题，改码无意义。

### 改码执行（协议包住 TDD，不替代）

改码入口照常走 `dj-implement` / `dj-tdd`，本技能不接管编码。**TDD 红测兼任专项测试**：bug 修复先写失败测试，功能新增先写验收测试。粒度 = 逻辑任务级：任务内多次编辑不重复跑回归，任务结束执行一次 S3。

### S2 · 专项验证（先行，便宜、归因清晰）

改码完成后**先跑专项再跑回归**：红测转绿/验收通过 = 改动生效；专项失败 = 改动没生效，回改码，此时**不要**跑 S3。专项挂 = 改错了；回归挂 = 改多了波及别处——两类失败归因分离，不得混报。

**S2.5 · JiT 探针（条件触发）**：diff 触及**无测试覆盖的新行为**时，基线绿 + 回归绿 = 假阴性。按 `references/jit-probes.md` 生成探针补验证；diff 在既有覆盖内则跳过。**测试须真能锁 bug**：用 `scripts/regression_check.py --base-ref <pre-fix> --test-cmd <测试命令> [--symptom <正则>]`把候选测试冻结到基线版本上跑——基线红（命中 symptom）+ 候选绿才算锁定；基线也绿 = 测试没抓住 bug（exit 1）。

### S3 · 改后回归

1. 校验 commit-id 与基线一致：中途有其他提交（基线过期）→ 重跑 S1
2. 重跑快速层，与基线逐项 diff：已知失败保持失败不算回归；**新增失败 → S4**；已知失败转绿 → 顺带修复记录（不放行条件）
3. **门禁自检**：`scripts/floor_guard.py --base <分支点>` 查五类「最便宜变绿」与契约缺命令行；exit 2 不得读作通过（详 `references/quality-contract.md`）

### S4 · 新增失败处置（按序，失败才进下一步）

1. **基线 diff 滤噪**：确认不在已知失败清单；基线缺失/过期时用 `git stash` 重跑**同一条命令、同一组测试**甄别存量（单跑/合跑不可比）。**只 stash 自己改动的文件**（`git stash push -- <path>`）——工作区有他人未提交改动时，全量 stash 回干净 HEAD 会把别人的改动一起抽走，「本来就坏」的现场消失、存量被误判成自己的回归。stash 循环被中断会把修改埋进 stash、后续 pop 按 LIFO 弹错条目**静默丢失修改**——中断后先 `git stash list` + `git status` 对账，pop 后 grep 标志行确认修改还在
2. **flaky 重试 ×1**：单独重跑，过则标记 flaky 放行（连续两次任务同 flaky 项建议隔离修复）。**重跑只用于定性、不算通过**：flaky 项不得作为"回归已绿"证据，须记入测试债台账跨轮追踪；发布卡点的 flaky 不得静默放过（语义细节见 `references/protocol-details.md` §6）
3. **自动修复 ≤ 2 轮**：定位修复，重跑 S3
4. **🛑 STOP · 回滚+报告**：回滚修改，保留失败现场，报告"改动导致回归已回滚"。**回滚后等用户裁决，不得自行重新改码**

**铁律：禁止改/删/跳失败的测试让回归变绿。** 测试自身问题（断言过时、环境变化）必须显式标记说明，交用户裁决。

### S5 · 全量层建议（🛑 决定权在用户）

任务改动波及面大时（diff 触及路由/依赖清单/公共工具库/DB schema 的 P0 范围，详情见 references）主动建议跑全量，话术含波及面证据。**用户未答复前不得自行启动**；同意 → 转交 `dj-fullstack-testing`；不同意 → 记录后止。

### S6 · 测试债台账

走过零基建退路或补全不完整的任务，欠账记入 `docs/testing/test-debt.md`（模块 | 欠账 | 补全方式 | 日期）。台账非阻塞；任务触及欠账模块时顺带偿还；汇报时附测试债概览。

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
| 快照存 docs/ | 生命周期=本任务 |
| 混跑专项与回归 | 专项先行，回归后行 |
| stash 甄别存量被中断埋掉修改 | `git stash list` + `git status` 对账，pop 后验证标志行还在 |
| 脏工作区全量 stash 甄别（抽走他人未提交改动） | 只 stash 自己的文件：`git stash push -- <path>` |
