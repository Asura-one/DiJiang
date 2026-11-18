---
name: dj-absorb
name_cn: 选择性吸收融合
description_cn: 以自有项目为主体，有选择地提取外部目标的设计模式、交互方式、功能模块或视觉语言，增量融入已有代码库，保持风格与架构一致。
description: "有选择地吸收外部项目的设计模式、交互方式、功能模块或视觉语言， 融合到自有项目中。输入是\"自有项目 + 一个或多个外部目标\"， 输出是\"外部元素被有选择地融入自有项目后的增量改动\"。 触发词：吸收、融合、借鉴、参考、汲取、养分、借鉴模式、吸收融合、借鉴设计、融入。"
disable-model-invocation: true
---

参考规范：`skills/engineering/dj-meta/references/anti-patterns.md`、`skills/engineering/dj-check/references/intensity-levels.md`、`skills/engineering/dj-check/references/output-markers.md`、`skills/engineering/dj-check/references/design-modes.md`。
深度细节：卡片/融合方案/快照与范围确认模板、Phase 4 验证清单、四态重验与多目标并行调研细则 → `references/fusion-templates.md`；拆解维度 → `references/absorption-checklist.md`；pstack 融合评估与落地进度存档 → `references/pstack-fusion-evaluation.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 外部精选元素增量融入自有项目，风格/架构一致，无回归 |
| **Done when** | Phase 4 清单全部通过，用户确认交付 |
| **Evidence** | 筛选清单、融合方案、增量 diff、验证记录 |
| **Output** | 吸收元素卡片 + 增量代码改动 + 融合说明 |

# Absorb: 选择性吸收融合

## 要解决的问题

吸收 vs 复刻边界、五大主要风险详释 → `references/fusion-templates.md`

五步流程概览 → `references/fusion-templates.md`

## 强度等级

三档定义（lite/full/ultra）→ `references/fusion-templates.md`

## Phase 0：摸底

摸底细则 → `references/fusion-templates.md`

**🔴 借鉴前先查许可证（商业化红线）**：外部目标是开源代码、且自有项目要商业化/闭源交付时，**先查外部目标的开源许可证**再谈吸收深度——这是最高优先级的前置决策，决定整个吸收的参考方式。三档风险：**机制级借鉴**（读懂原理自行实现，如"800ms 去抖 + CAS 状态机"思想）风险最低；**结构级参照**（同类 schema/表/包命名）次之；**代码级平移**（逐段翻译/搬运）风险最直接。若是 AGPL 类传染性许可证，代码级平移会把自有商业化产品拖入开源义务。查证动作：读外部仓库 LICENSE 全文 + 仓库元数据（标注 URL 与访问日期），并核验衍生项目自身如何标注来源。**同时查依赖树**：参考项目的核心能力可能来自宽松许可的独立依赖（实证：ARTEX 的 agent 循环来自 Apache-2.0 的 norma），那是唯一可代码级复用的通道——"自研 vs 外挂第三方 CLI"之外还有第三条路：自研业务层 + 复用宽松许可的底层库。**取证基线**：上游删库或镜像不可信时（实证：原仓库 404、流行镜像被 CERT 确认投毒），以本地带完整 git 历史的快照为唯一可信基线，绝不用网上镜像作代码源。产出"可搬/必须改写/不能碰"三档红线清单，写进吸收方案供所有后续步骤引用。**参考外部代码做架构级吸收时，许可证票应作为第一张决策票优先解决**（实证：Go 重写参考闭源前开源版 ARTEX，许可证票决定机制/结构/代码三档借鉴深度，先于所有下游设计）。

## Phase 1：拆解与筛选

拆解维度 → `references/absorption-checklist.md`；候选元素卡片 → `references/fusion-templates.md`

调研报告四态重验与多目标并行调研细则 → `references/fusion-templates.md`

## Phase 2：融合方案

方案模板与适配/冲突/实现策略要素 → `references/fusion-templates.md`

融合形式前置判定（新增 vs 内嵌）与 dj-ask 三处路由 → `references/pstack-fusion-evaluation.md`

**🔴 🛑 STOP：展示所有融合方案给用户确认**后再进 Phase 3；用户可能调整范围、修改方案或放弃某些元素。

## Phase 3：落地实现

逐条落地纪律 → `references/fusion-templates.md`

## Phase 4：最终验证

六项验证清单 → `references/fusion-templates.md`

## Hard Rules

1. 5 个 Phase 走完，不跳 Phase 0（摸底）和 Phase 2（融合方案设计）
2. 拆分清单和融合方案书面化，不能只在脑子里
3. Phase 2 方案经用户确认才进 Phase 3
4. 技术限制或不兼容明确告知原因和替代方案，不悄悄简化
5. 不以"借鉴"为名整页复刻——那是 `dj-remix` 范畴
6. 不破坏自有风格/架构一致性——每个外来元素确认与现有部分匹配
7. 多目标吸收自有项目是主体，外来适配自有风格，反之不行
8. 每实现一个元素验证一个，不堆到 Phase 4 一次性检查
9. 吸收范围与融合方案的确认走 `dj-grill` 逐轮拷问（一次一题、附推荐答案、取得明确共享理解后才进 Phase 3）；批量多选 question 不算拷问

## Gotchas

| Gotcha | 预防 |
|---|---|
| 商业化产品直接搬外部开源代码 | **先查许可证再谈吸收深度**：机制级/结构级/代码级三档风险不同，AGPL 类传染性许可证下代码级平移会拖入开源义务；许可证票作为第一张决策票优先解决 |

上述余下短条目与全部长条目教训实证 → `references/fusion-templates.md`、`references/pstack-fusion-evaluation.md`
