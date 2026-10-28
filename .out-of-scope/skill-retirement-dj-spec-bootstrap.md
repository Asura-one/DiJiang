# 退役: dj-spec-bootstrap（Spec 初始化技能）

- **退役日期**：2026-10-28
- **来源**：DJ-014（fusion-chain 任务，四轮诊断后首例技能退役）
- **否决原因**：产品从未被消费——`.dijiang/spec/` 目录实际为空（技能自身产出物从未落盘）；全库零入度（无任何技能正文引用它）；`.dijiang/spec/` 的消费链已断（dj-meta 已改指 `.dijiang/spec/` 为辅助参考目录说明，无运行时依赖）；三个月使用记录为零（37 份记忆文件零命中）。SCOPE 退役政策三触发条件（被更高层覆盖/零引用孤儿/与治理文档冲突）中"零引用孤儿"成立。
- **最强理由**：模块探测表（Cargo.toml→crates、package.json→src、go.mod→顶层包）是个好想法——新项目冷启动时自动生成 spec 骨架能省手工。但它解决的是"spec 覆盖缺口"，而 DiJiang 的 spec 一直由 dj-split/dj-output 的任务文档链承担，缺口假设从未被真实失败验证过（SCOPE 双门槛第一条不满足）。若未来多模块大仓真出现"无 spec 覆盖"的真实失败，这文档就是重启点。
- **复活条件**：项目出现"新模块持续多周无 spec 覆盖且造成实际返工"的真实失败案例，且任务文档链（prd.md/design.md）不足以承载时重评。

## 退役时技能快照

- 功能：扫描项目主要源码目录，为每个模块生成 `.dijiang/spec/<module>/index.md`
- 最后状态：darwin 两轮优化过（7975eec/1e90a1f），verify PASS
- 本档案即复活时的最小记忆载体；完整历史见 git log skills/engineering/dj-spec-bootstrap/
