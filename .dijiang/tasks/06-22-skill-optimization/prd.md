# Skill Optimization and Evaluation

## Goal

对当前项目下的所有 skill 进行评审和优化，提升 skill 质量。

## Requirements

- 使用 darwin-skill 的 9 维度评估 rubric 对每个 skill 进行评分
- 对每个 skill 设计测试 prompt 进行实测验证
- 优化低分 skill，确保改进后分数严格高于改进前
- 生成优化报告和成果卡片

## Acceptance Criteria

- [ ] 所有 skill 都经过评估并记录分数
- [ ] 低分 skill 经过优化并验证改进效果
- [ ] 生成优化报告和成果卡片
- [ ] 结果记录到 results.tsv

## Notes

- 优化范围：待确认（全部 16 个 skill 或指定 skill）
- 评估标准：darwin-skill 的 9 维度 rubric（结构维度 59 分 + 效果维度 35 分 + Meta-skill 维度 6 分）
- 优化策略：按分数从低到高排序，优先优化最低 5-10 个
- 验证方式：独立子 agent 评估，避免自评偏差

## 现有基线

根据 results.tsv 记录，16 个 skill 已有基线评估结果（round4）：
- 分数范围：80.9 - 87.4
- 最低分：check (80.9) → round4 提升至 86.6
- 最高分：grill (87.4) 和 script (87.4)
- 状态：全部为 keep（已保留）
- 评估模式：dry_run（干跑验证）

**关键发现**：
1. 所有 skill 都已在 round4 完成评估
2. 评估模式均为 dry_run，缺乏实测验证
3. 需要补充真实测试 prompt 进行 full_test 验证

## 优化建议

基于现有基线，建议：
1. **优先优化低分 skill**：dispatch (81.9), tdd (81.9), ponytail (81.9), prototype (81.9)
2. **补充实测验证**：为每个 skill 设计测试 prompt，进行 full_test
3. **关注 dry_run 比例**：当前 100% dry_run，需降至 30% 以下
