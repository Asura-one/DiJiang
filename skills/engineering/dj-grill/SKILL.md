---
name: dj-grill
description: "不留情的深度拷问：走遍决策树的每个分支，直到达成共享理解。一次一个问题，追到没有模糊为止。 触发词：拷问、grill、对齐需求、逐轮确认、需求不清、想法细化。"
disable-model-invocation: true
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 需求共享理解，关键决策已逐轮确认 |
| **Done when** | 访谈已开始，至少一题含推荐答案与用户回答，且用户明确确认共享理解 |
| **Evidence** | `task.json` 的 `meta.grilling` 含 `startedAt`、至少一条 `questions[{prompt,recommendation,answer}]`、`confirmedAt` 与 `confirmation`；它是可检查的声明性记录，不是 Pi UI 签发或不可伪造的确认凭证 |
| **Output** | 对齐后的需求描述、访谈门禁证据与已捕获的领域术语 |

## Convergence Mode

从 `.dijiang/config.toml` 的 `[workflow].grill_mode` 读取模式。没有配置或取值无效时使用 `grill-me`。

无论模式为何，非琐碎任务进入实现前都必须完成至少一次逐轮拷问，并取得用户明确确认共享理解。完整 PRD 只可减少问题数量，不能豁免该门禁。

| 模式 | 适用条件 | 行为 |
|---|---|---|
| `adaptive` | 已有材料较充分 | 先调查材料与代码；提出一个最高信息量、会影响实现的决策题。 |
| `grill-me` | 默认；需要共同探索方案 | 按决策依赖逐题推进，每题附推荐答案。 |
| `grill-with-doc` | 已有 PRD、issue、设计稿或需要记录决策 | 先阅读材料；至少提出一个最高信息量决策题，并持续写入任务文档。 |

高模糊/绿地任务先走**发散前段**（先铺 unknown-unknowns 再收敛）→ `references/diverge-first.md`——否则方向不清时的质询是在高效坐实错误前提。

每次只问一个问题，等待用户回答后再继续。问题应附带推荐答案。六维（目标/范围/方案/约束/验收/风险）的典型拷问角度与话术 → `references/question-bank.md`。可由代码库、文件或工具确认的事实必须先自行调查，不能反问用户。答案会改变范围、方案或验收时，沿决策树继续下一题。

已有实质 PRD 时，先从材料提取事实、假设与冲突，再提出一个最高信息量的决策题。六个维度是检查清单，不是固定问卷。

只有用户明确确认共享理解后，才停止访谈并允许实现。用户说“开始做”不等同于确认，除非其明确表示已达成共享理解。

结束时写入 `{task_dir}/task.json` 的最小门禁证据（`meta.grilling` JSON 结构示例）→ `references/evidence-artifacts.md`。
参考规范：`skills/engineering/dj-meta/references/anti-patterns.md`（跨技能行为约束）。

## Artifact Rules

任务文档维护（prd/design/implement）、`grill-with-doc` 文档契约与 PRD Convergence Pass 的展开 → `references/evidence-artifacts.md`。
- 文档格式参考 `dj-domain-modeling/references/context-format.md` 与 `dj-domain-modeling/references/adr-format.md`。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 用户答非所问 | 复述并追问 | 标「待澄清」 |
| 用户回答与已有 PRD 冲突 | 标注冲突，回问哪边为准 | 停下对齐 |

## Hard Rules

1. 一次只问一个会改变实现决策的问题，不能堆问题。
2. 每题必须给出推荐答案；事实能由环境确认时先调查。推荐答案附带把握度（0-100%），低于 70% 时明写「还缺什么」。
   - 「都行 / 你定 / whatever you think」是委托不是决策：换两个具体选项重问，不替用户拍板。
   - 听到「可扩展 / 最佳实践 / 行业惯例」这类精致话术时，用一句探针回到真实需求：「如果不用向任何人交代，你实际想要什么？」
3. 已有文档不能豁免至少一题最高信息量的决策拷问。
4. 非琐碎任务在用户明确确认共享理解前，不得进入实现。
5. 每次获得新决策后更新 `{task_dir}/prd.md`，保留假设和待确认项。
6. `grill-with-doc` 模式下，立即记录确认术语；仅为难逆、存在真实权衡的决策创建 ADR。
7. 结束前写入结构化 `meta.grilling` 记录：`startedAt`、至少一条含问题/推荐答案/用户回答的 `questions`，以及 `confirmedAt` 和用户确认原话。
8. 结束前执行 PRD Convergence Pass，确认任务文档足以支撑下一阶段。
9. 停止判据可检验：能预测用户对接下来三个问题的反应，且没有待确认项残留；否则继续问。

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 一次问 3 个问题 | 用户只回答最后一个 | 一次一个问题 |
| 用户回答了就默认懂了 | 漏了关键细节 | 每个答案追问确认 |
| 不问验收条件 | 实现完了对不上 | 验收是必须问的维度 |
| 不捕捉术语 | 后面代码用词不统一 | 听到新术语就记下来 |

如果问题涉及设计决策而非需求对齐，参考 `skills/engineering/dj-check/references/design-modes.md` 选择分析模式。
**下一步**：需求对齐确认后，建议下一步调用 `dj-output` 把共识沉淀为 PRD（含 TBD 项与验收标准），供 `dj-split` 拆分。
