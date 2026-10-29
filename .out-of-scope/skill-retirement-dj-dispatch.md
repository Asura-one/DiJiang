# 退役: dj-dispatch（任务分类器）

- **退役日期**：2026-10-29
- **来源**：用户提案（dj-ask 与 dj-dispatch 职责重叠，且高频使用 dj-ask；合并方案经多选确认）
- **否决原因**：双路由的职责重叠风险（design.md 原 §4 权衡中预判）被使用验证为现实成本——routing-table 与 ask 技能目录维护同一份 skill 映射，skill 变更须同步两处；触发面模糊（"用哪个 skill" vs "新任务"）抬高用户选择成本；逐请求路由的即时分流在现代 harness 中已被平台对 skill description 的自动匹配机制部分替代。dispatch 的独有增量（歧义消解 4 条规则、显式输出路由决策、`planning` re-entry 锚点）是几行规则而非独立工作流，已随合并平移入 dj-ask。
- **最强理由**：单请求级调度需要快速确定性、全局路由需要解释 flow 全貌——两种粒度的认知负载确实不同，分离有理论依据。但实践中 dispatcher 的"一进一出"决策已被两层自然接管：平台对 description 的自动匹配承担即时分流，dj-ask 的 flow 图承担路径规划；剩余的歧义消解与显式输出并入 ask 只增十几行，两个技能共享一套 skill 事实，单源维护反而更不易漂移。
- **复活条件**：skill 数量增长到 flow 图与关键词表在同一技能内互相干扰（SKILL.md 拆 references 也无法承载），或出现"路由决策需要独立执行历史/状态"的真实失败时重评。

## 退役时技能快照

- 功能：识别请求类型 → 路由到对应 dj-* skill；混合任务主类型驱动串联；低置信度转 dj-grill；作为任务状态机 re-entry 锚点（planning 重新路由）
- 最后状态：darwin 优化后 76.0 分，verify PASS；核心路由规则与 routing-table.md 已并入 dj-ask，re-entry 锚点引用全部改指 dj-ask
- 本档案即复活时的最小记忆载体；完整历史见 git log skills/engineering/dj-dispatch/
