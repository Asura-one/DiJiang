---
name: dj-evolve
name_cn: 技能自进化
description_cn: "任务收尾时回顾本 session 的发现/经验/纠正，按升格判据筛选可复用经验，生成技能更新候选并验证后落库。打通'记录→规则'闭环。"
description: "任务收尾时回顾本 session 的发现、经验与纠正，按升格判据筛选可复用经验，生成技能更新候选，差分验证后经用户确认落库。 Use when a task session produced reusable lessons, or the user asks to consolidate experience into skills. 触发词：总结经验、沉淀技能、自进化、evolve、升格技能、经验进技能。"
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 经判据筛选、差分验证、用户确认的技能更新；或明确的"不升格"结论 |
| **Done when** | 每个候选都有落库或拒绝的留痕决策 |
| **Evidence** | 候选清单、判据核对记录、差分对比结果、拒绝留档 |
| **Output** | 技能更新候选（每次 ≤3 个）或拒绝留档 |

# Evolve: 技能自进化

使用过程中总结经验并更新 skill——记忆系统只存不用会变成坟场，本 skill 是"记录→规则"的出口。升格判据以 `SCOPE.md` 为权威，本 skill 只定义执行流程。

## Recursion Guard

你已身为 dj-evolve。直接执行总结与筛选，不得再调用或派发另一个 dj-evolve。候选落库的实施走 `dj-implement` 纪律（worktree），收尾走 `dijiang-finish-work`。

## 工作流

### 1. 回顾

扫描本 session 的任务产物与记忆写入（findings / learnings / corrections），列出全部候选经验。先读 `.dijiang/evolution-rejections.md` 拒绝史（若存在），已被拒绝过的不再重复提案。

### 2. 判据筛选（SCOPE.md 升格五问）

- 它未来还会跨任务重复出现吗？
- 带明确的做事方法吗（不只是观察）？
- 写进 skill 比留在 memory 更省 token 和错误率吗？
- 有验证基础吗（未验证的先暂存 dj-memory 标待验证）？

四类挖掘信号：重复人工纠正 / 稳定成功的长流程 / 高频工具错误 / 反复查询的外部知识。未过判据 → 记忆层暂存；过 → 进入候选。

### 3. 候选分类

| 性质 | 落点 |
|---|---|
| 单技能流程增强 | 对应 SKILL.md 增量段落 |
| 跨技能通用纪律 | `docs/references/` 反模式新条目或参考文档 |
| 路由不命中 | description 触发词补充 |
| 全新能力 | 走 `SCOPE.md` 门槛提案——可组合则不建新技能 |

### 4. 差分验证

- 候选触及共享文档（anti-patterns 等 46 技能共读文件）时：对照 `tests/behavior-cases.json` 全量案例评估不伤既有行为——**比基线看差值，不听候选自述**
- 结构合规跑 `python scripts/verify_skills.py --strict`

### 5. 🛑 确认与留痕

展示候选（≤3 个/次）：判据核对 + 差分结果 + 落点。用户裁决：

- **确认** → 按 `dj-implement` 纪律实施，worktree 开发、`dijiang-finish-work` 收尾
- **拒绝** → 追加 `.dijiang/evolution-rejections.md` 一行（日期 / 候选摘要 / 拒绝理由），下轮先读拒绝史

## 边界

- 不改平台自进化机制（darwin review 是平台能力，本 skill 是库内主动流程）
- 升格判据与 `SCOPE.md` 冲突时以 SCOPE 为准
- 每次最多 3 个候选——防批量洗库
- 新建技能候选必须先过 `SCOPE.md` 双门槛 + `dj-meta` 体量约束

## 反模式

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 一次提案 10 个候选批量改库 | 每次 ≤3 个，改完验证再下一批 |
| 未验证的经验直接进 skill | 暂存 dj-memory 标待验证，验证后提候选 |
| 听候选自述"改进了 X" | 差分判据：比基线看差值 |
| 拒绝后不留痕 | 写拒绝档案，防下轮重复提案 |

## Hard Rules

1. 每个候选必须有判据核对与差分验证，缺一不提案
2. 落库需用户确认，不静默改技能文件
3. 拒绝必留档
4. 触及共享文档必跑 behavior-cases 对比