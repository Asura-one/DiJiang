# 退役: dj-pattern（模式识别）

- **退役日期**：2026-11-05
- **来源**：全库技能健康审计（dj-audit health profile，三源证据：派发记录/引用入度/边界精读）后用户拍板
- **否决原因**：三个扫描维度逐条被现有技能覆盖——①重复代码（≥3 次）≈ dj-audit「过早抽象/冗余层」的镜像面 + dj-evolve 复现门槛同语义；②历史修复模式与 dj-audit 的 `references/git-archaeology.md`（bug 聚类 × 变更热点，2026-11-05 吸收自琅嬛）完全同域；③术语不一致 ≈ dj-domain-modeling 重载词挑战。零派发记录（task.json 分发史中 0 次）；非索引引用仅 4 处且均为「那是 dj-pattern 的职责」式免责行，无实质消费。
- **最强理由**：模式识别作为独立认知步骤有价值——「先确认再抽象、三次以下不抽、抽象必须评估迁移成本」是好纪律。但这些纪律已分散活在 dj-audit（证据要求）、dj-evolve（复现门槛 ≥2）、dj-ponytail（第二次调用者出现前不抽象）的承重条款里；独立维护一个 3066B 的技能来承载三者的交集，是纯熵。
- **复活条件**：出现「只找该抽未抽的重复、不管其他审计维度」的专项真实需求，或 ai-detector 等项目出现高频模式挖掘工作流时，作为 dj-audit 的 pattern profile 复活（骨架见 git log）。

## 退役时技能快照

- 功能：三维扫描（重复代码/历史修复/术语不一致）→ 模式报告（可复用抽象 + 反模式 + 成本估算），只报告不修改
- 最后状态：verify PASS；非索引引用 4 处已全部改指 dj-audit / dj-domain-modeling
- 本档案即复活时的最小记忆载体；完整历史见 git log skills/engineering/dj-pattern/
