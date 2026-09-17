# ADR 001: dj-* 技能精简为指令聚焦模式

## 状态
`accepted`

## 上下文
dj-* 技能承担了过多角色：核心指令、输入/输出约定、失败处理、反例教育、模板参考。导致技能文件体积大（平均 7KB，最大 17KB），agent 读取时注意力被非核心内容稀释。同时，mattpocock/skills 项目展示了更有效的模式：每个 SKILL.md 只保留 agent 执行时必须逐条读的最小指令集，辅助内容放到独立参考文档。

## 决策
将所有 dj-* 技能的 SKILL.md 精简为核心指令（平均 1.5-2.5KB），移除：

1. 输入/输出规格表 → 移到 `.dijiang/spec/<skill>/io.md`
2. 失败处理表 → 移到 `.dijiang/spec/<skill>/failure-handling.md`
3. 深度模块原则、第一性原理检查、Conventional Commits 规范 → 移到 `spec/dj-implement/principles.md`
4. CHECKPOINT 模板 → 精简为单行引用
5. 反例表 → 从 7-8 条减到 4 条核心

保留：frontmatter、职责、工作流步骤、核心规则、精简反例。

## 影响
正面：
- 技能总大小从 149KB 降到 44KB（-71%）
- agent 读技能时注意力集中在执行指令上
- 辅助内容仍可通过 `.dijiang/spec/` 引用

负面：
- 需要熟悉 `.dijiang/spec/` 位置才能找到参考内容
- 部分测试依赖移除的标记（已修复）

## 关联
- 触发：mattpocock/skills 分析
- 涉及：全部 20 个 dj-* 技能
- 后续：Phase 2 补齐流水线；Phase 3 并行 review
