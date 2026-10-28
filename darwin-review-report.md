# DiJiang 全部 47 Skill Darwin 9 维评审报告

**评审日期**：2026-10-28
**评审方法**：darwin-skill v2.1（SkillLens 9 维 rubric × SkillOpt validation-gated）
**范围**：`E:\Project\DiJiang\skills\` 下全部 47 个 skill（engineering 42 + productivity 5）
**模式**：仅评估不改（Phase 0.5 测试 prompt + Phase 1 基线），未进入优化循环
**实测**：dim8 用 5 个独立子 agent 全量对比（with_skill vs baseline），47/47 full_test（0 dry_run）

---

## 一、总分卡（结构 + 实测加权）

评分公式：结构分(dim1-7,9) + dim8×23/10，满分 100。

| 排名 | Skill | 结构分 | dim8实测 | 总分 | 评级 |
|---|---|---|---|---|---|
| 1 | dj-regression-guard | 67.9 | 9 | **88.6** | 🟢 |
| 2 | dj-fullstack-testing | 67.7 | 9 | **88.4** | 🟢 |
| 3 | dj-gov | 66.9 | 8 | **85.3** | 🟢 |
| 4 | dijiang-finish-work | 63.5 | 9 | **84.2** | 🟢 |
| 5 | dj-hunt | 65.5 | 8 | **83.9** | 🟢 |
| 6 | dj-remix | 61.7 | 9 | **82.4** | 🟢 |
| 7 | dj-triage | 61.7 | 8 | **80.1** | 🟢 |
| 8 | dj-evolve | 61.3 | 8 | **79.7** | 🟡 |
| 9 | dj-read | 60.4 | 8 | **78.8** | 🟡 |
| 10 | dijiang-continue | 60.4 | 8 | **78.8** | 🟡 |
| 11 | dj-grill | 60.1 | 8 | **78.5** | 🟡 |
| 12 | dj-absorb | 60.1 | 8 | **78.5** | 🟡 |
| 13 | dj-memory | 60.0 | 8 | **78.4** | 🟡 |
| 14 | dj-ponytail | 59.9 | 8 | **78.3** | 🟡 |
| 15 | dj-wayfinder | 60.9 | 7 | **77.0** | 🟡 |
| 16 | dj-split | 58.0 | 8 | **76.4** | 🟡 |
| 17 | dj-check | 55.6 | 9 | **76.3** | 🟡 |
| 18 | dj-implement | 62.3 | 6 | **76.1** | 🟡 |
| 19 | dj-dispatch | 59.9 | 7 | **76.0** | 🟡 |
| 20 | dj-script | 57.4 | 8 | **75.8** | 🟡 |
| 21 | dj-prototype | 56.0 | 8 | **74.4** | 🟠 |
| 22 | dj-pattern | 58.0 | 7 | **74.1** | 🟠 |
| 23 | dijiang-start | 58.0 | 7 | **74.1** | 🟠 |
| 24 | dj-design | 55.6 | 8 | **74.0** | 🟠 |
| 25 | dj-tdd | 57.6 | 7 | **73.7** | 🟠 |
| 26 | dj-audit | 57.0 | 7 | **73.1** | 🟠 |
| 27 | dj-setup | 56.8 | 7 | **72.9** | 🟠 |
| 28 | dj-review | 56.2 | 7 | **72.3** | 🟠 |
| 29 | dj-research | 56.0 | 7 | **72.1** | 🟠 |
| 30 | dj-write | 56.0 | 7 | **72.1** | 🟠 |
| 31 | dj-output | 53.4 | 8 | **71.8** | 🟠 |
| 32 | dj-teach | 53.8 | 7 | **69.9** | 🟡→🟠 |
| 33 | dj-ask | 53.2 | 7 | **69.3** | 🟠 |
| 34 | dj-meta | 52.8 | 7 | **68.9** | 🟠 |
| 35 | dj-handoff | 52.6 | 7 | **68.7** | 🟠 |
| 36 | dj-reason | 52.6 | 7 | **68.7** | 🟠 |
| 37 | dj-spec-bootstrap | 53.2 | 6 | **67.0** | 🟠 |
| 38 | dj-writing-for-agents | 48.6 | 8 | **67.0** | 🟠 |
| 39 | dj-merge-conflict | 50.7 | 7 | **66.8** | 🟠 |
| 40 | dj-wizard | 52.4 | 6 | **66.2** | 🟠 |
| 41 | dj-domain-modeling | 50.0 | 7 | **66.1** | 🟠 |
| 42 | dj-codebase-design | 49.7 | 7 | **65.8** | 🟠 |
| 43 | dj-questionnaire | 49.2 | 7 | **65.3** | 🟠 |
| 44 | dj-git-guardrails | 51.1 | 6 | **64.9** | 🟠 |
| 45 | dj-channel | 47.1 | 6 | **60.9** | 🟠 |
| 46 | dj-session-insight | 45.9 | 6 | **59.7** | 🔴 |
| 47 | dj-wait-what | 41.3 | 5 | **52.8** | 🔴 |

> 评级：🟢≥80 优秀 · 🟡70-79 良好 · 🟠60-69 合格 · 🔴<60 需提升

**统计**：平均 73.5；≥80 共 7 个；70-79 共 13 个；60-69 共 23 个；<60 共 2 个。

---

## 二、各维度系统分析

### 2.1 结构维度（dim1-7, 9）——平均 56.5/100

**普遍偏弱是结构性、非个别**：47 个全部落在 41.3-67.9 之间，无一个 ≥70。这说明**整库在结构上有共性短板**，而非个别 skill 质量问题。

#### 全库共性短板（按加权 gap 排序）

| 维度 | 权重 | 平均分(10) | 加权贡献 | gap | 说明 |
|---|---|---|---|---|---|
| **dim5 可执行具体性** | 18 | 7.4 | 13.3 | 4.7 | 多数已较具体，但部分 skill 含「建议/可以考虑」软化措辞 |
| **dim3 失败模式编码** | 12 | 6.0 | 7.2 | 4.8 | **全库最大短板**：普遍用 Hard Rules/Gotchas 表格兜底，缺「如果 X 失败 → Y」显式 if-then 分支 |
| **dim2 工作流清晰度** | 12 | 7.7 | 9.2 | 2.8 | 大部良好，短 skill（wait-what/spec-bootstrap）步骤过简 |
| **dim4 检查点设计** | 6 | 6.0 | 3.6 | 2.4 | **普遍缺显性 🔴/🛑 STOP 标记**，多靠文字「停止」 |
| **dim9 反例黑名单** | 6 | 6.7 | 4.0 | 2.0 | 多数有反例表，但部分（channel/session-insight）缺 |
| **dim1 frontmatter** | 7 | 8.2 | 5.7 | 1.3 | 良好，几乎全部达标 |
| **dim6 资源整合** | 4 | 6.2 | 2.5 | 1.5 | 25/47 有 references；少数引用路径需核实 |
| **dim7 整体架构** | 12 | 7.6 | 9.1 | 2.9 | 良好，极少数有冗余 |

**结构性最弱的 10 个**：dj-wait-what(41.3)、dj-session-insight(45.9)、dj-channel(47.1)、dj-writing-for-agents(48.6)、dj-questionnaire(49.2)、dj-codebase-design(49.7)、dj-domain-modeling(50.0)、dj-merge-conflict(50.7)、dj-git-guardrails(51.1)、dj-meta(52.8)。

### 2.2 效果维度（dim8 实测）——平均 7.2/10

- 强（8-9，baseline 难以替代）：**regression-guard、fullstack-testing、finish-work、hunt、remix、triage、read、memory、ponytail、check** 等——这些 skill 把模型默认行为扭转成有明确状态机/基线/防错护栏的可执行流程，增益真实显著。
- 中（7）：多数工程 skill，提供流程与纠偏，模型有部分默认能力，增益温和但真实。
- 弱（5-6）：**dj-wait-what(5)**（baseline 对"没懂"本就会带上下文重讲）、**dj-channel(6)**（强依赖外部 CLI 工具）、**dj-session-insight(6)**（纯桥接型，增益全依赖下游）、**dj-wizard(6)**（模板未内嵌）、**dj-spec-bootstrap(6)**（产出仅模板 index）、**dj-implement(6)**（纪律齐全但写代码本底尚可）、**dj-git-guardrails(6)**（多为有经验 agent 常识）。

**共性扣分点**：多个 skill 依赖 `references/` 外部文件未内嵌，实测落地一致性打折（wizard/teach 等）。

### 2.3 runtime 适配性（gate 项）

红灯扫描命中仅 `Call the Skill tool`（dijiang-start、dj-channel、dj-script、dj-session-insight、dj-tdd），为**本平台 TeleAgent 固有调用方式**，属允许的「花叔生态内部引用」例外，判为 `false_positive`，**无真红灯**（无 Claude Code/Cursor/Codex 单 runtime 钉死措辞）。**全库 runtime 中立性通过**。

---

## 三、发现的具体问题

### 3.1 需修复（非评审免责）

| # | skill | 问题 | 优先级 |
|---|---|---|---|
| 1 | **dj-meta** | 「治理门槛」章节**重复 2 次**（L59-64 与 L62-67）| P1 文案冗余 |
| 2 | **dj-session-insight** | 文末 AIGC 水印块重复 2 次（L73-90）；大量「Call the Skill tool」重复文本 | P1 冗余 |
| 3 | **dj-script** | 「删除文件直接 rm」反模式行重复 2 次 | P1 冗余 |
| 4 | **dijiang-start** | 与 dijiang-continue 结构高度同质（加载状态→读产物→推荐路线），体量冗余 | P2 合并/差异化 |

### 3.2 全库维度短板（优化候选）

1. **dim3 失败模式编码**：普遍缺 if-then 三段式 fallback。建议为 15 个 60 分以上 skill 补充「如果 X 失败 → Y 一线 → 仍失败兜底」结构（HL-2）。
2. **dim4 显性检查点**：多靠文字「停止」非显性标记。建议加 `🔴 CHECKPOINT`/`🛑 STOP`（HL-1，4 行改动撬动 +3 分）。
3. **可执行具体性**：少量 skill 含「建议/考虑」软化词，按 rubric 需消解。

---

## 四、结论与建议

**总评**：47 个 skill 整体质量**中等偏上（平均 73.5）**，实测（dim8）增益全部为正、无一为负，说明**所有 skill 都有真实存在价值**。结构上 **强 skill 集中在「安全护栏类」（regression-guard/fullstack-testing/gov/finish-work/hunt）**，这些是常识最守不住、一旦遵守收益最大的纪律型 skill。

**建议的优化优先级**（进入 Phase 2 时按总分从低到高，先改最弱）：

1. **P0 清理 3 处冗余**（dj-meta 重复段、dj-session-insight 重复水印+冗余、dj-script 重复行）——零风险快速收益
2. **P1 结构补强最弱 5 个**：dj-wait-what(52.8)、dj-session-insight(59.7)、dj-channel(60.9)、dj-git-guardrails(64.9)、dj-questionnaire(65.3)—— 补 dim3 if-then + dim4 检查点 + dim9 反例
3. **P2 全库 dim3/dim4 增补**：用 HL-1/HL-2 手法给 60-70 分档的 23 个 skill 加失败分支与显性 STOP

**不建议动的**：dj-regression-guard(88.9)、dj-fullstack-testing(88.4)、dj-gov(85.3) 已达优秀，避免过度工程（HL-3/HL-4 触顶即停）。

---

## 五、Phase 2 优化循环结果（darwin ratchet 棘轮）

**执行日期**：2026-10-28 · **分支**：`auto-optimize/20261028-0920`

### 5.1 执行总览

| 指标 | 值 |
|---|---|
| 处理 skill | **40/47**（另 7 个 ≥80 保持不动，符合「见好就收」）|
| 净改进保留 | **38 个** |
| revert | 1（dj-implement）+ 1 撤表（dj-absorb）|
| commit | 53 个（每 skill 独立 commit，可单独 revert）|
| paired judge 评估 | ~160 次独立子 agent 三方比较 |
| 精修补正 | 13 次 round2/round3（剔除复读行后全部转 better）|

### 5.2 棘轮判定统计

- **round1**：40 个 skill 全部经 3-judge paired——多数 `better` keep；dj-spec-bootstrap 3-0 `tie`、dj-write 2-1、dijiang-start 2-1、dj-wayfinder 1-2、dj-ponytail 2-1、dj-absorb 1-1-1 触发精修
- **revert**：dj-implement 失败模式表被 3 judge 判「复读正文、接近 tie」→ 主动 revert（未改进比假装改进诚实）
- **精修**：所有被点名复读的 skill 经 round2/3 剔除凑分行，全部转 3-0 `better`；dj-write round2 误删「语义漂移」真增量被 2/3 判 `worse` → revert → round3 精准修正——**paired 多数决两次拦住误伤**
- **dj-absorb**：三行失败模式 100% 复读既有 Gotchas → 整表撤除（judge 判 clear better）

### 5.3 主要改进维度

1. **dim3 失败模式**（38 个 skill）：补齐「触发条件 → 一线修复 → 仍失败兜底」if-then 三段式，全库共性最大短板被系统性填补
2. **dim7 冗余清理**：dj-meta「治理门槛」重复 2 次与「每技能明确」重复、dj-script 反例表重复行、dj-output/dj-research 与反例重复的 Gotchas、dj-session-insight 反例复读——存量缺陷一并清除
3. **dim9 反例**：低分 skill（wait-what/channel/questionnaire 等）补齐反例黑名单

### 5.4 judge 揭示的过程教训（对后续 skill 演化的价值）

- **模板化批量补表会混入复读行**（批 3/6 最明显）——失败模式必须从真实执行失败中挖掘，不是从正文既有规则改写
- **「兜底列」是失败模式表的灵魂**——正文已有的预防性规则不算失败模式，只有「触发后怎么办」的三段升级路径才算
- **去冗余也可能误伤**：删除前必须验证被删内容在正文有替代载体（grep 验证），否则如 dj-write 丢掉「语义存疑请作者确认」

### 5.5 未动清单（≥80 优秀，避免过度工程）

dj-regression-guard(88.6) · dj-fullstack-testing(88.4) · dj-gov(85.3) · dijiang-finish-work(84.2) · dj-hunt(83.9) · dj-remix(82.4) · dj-triage(80.1)

### 5.6 后续候选处理记录

| 候选 | 状态 |
|---|---|
| dj-reason「边界」↔ Hard Rules 逐字重复 | ✅ **已处理**（dim7 专项 `22879c5`：删「边界」节，3 条全有 Hard Rules 逐字载体，零信息损失，3-0 better keep）|
| dj-handoff 四重结构复读 | ✅ **已处理**（dim7 专项 `b2b31fd`：删「规则」节+Gotchas 表，四结构收敛为 失败模式/反例/Hard Rules 三结构，体量 -33%，3-0 better keep）|

两处均按新落反模式 **#26 去冗余不验载体**执行：删除前 grep 验证每条替代载体，`verify_skills.py --strict` PASS 146 / FAIL 0。

---

## 六、评审留档

- 测试 prompt 集：`.temp/darwin-review/test-prompts.json`（47 skill × 2 prompt）
- 结构评分明细：`.temp/darwin-review/structural-scores.json`
- 历史记录：`.temp/darwin-review/results.tsv`（47 结构基线 + 47 综合 full_test 行）

*本报告由 darwin-skill 2.1 9 维 rubric 生成，dim8 为独立子 agent 实测，keep/revert 需在 Phase 2 走 paired 多数决。*