# ADR 003: dj-review 并行审查 + dj-meta + ADR 机制

## 状态
`accepted`

## 上下文
Phase 1 精简了技能体量，Phase 2 补齐了流水线。Phase 3 要进一步提升质量和可维护性：

1. 代码审查的效率可以通过并行子 agent 提升（mattpocock 的 code-review 使用了类似模式）
2. 技能本身的质量需要有元规范和写作指南来保障
3. 架构决策需要有持久化记录机制

## 决策

### 1. dj-review 并行化
使用 `delegate_task` 并行执行两个独立子任务：spec 匹配度审查和代码质量审查。两个子 agent 独立运行后汇总结果。

### 2. dj-meta 技能写作指南
创建元技能记录 dj-* 的设计原则、体量控制、创建模板。新技能创建和现有技能审查以此为参考。

### 3. ADR 机制
- 创建 `.dijiang/decisions/` 目录
- 采用 Michael Nygard 的 ADR 模板
- 编号作为文件名前缀（NNN-title.md）
- 已接受的 ADR 不再修改

## 影响
正面：
- 代码审查通过并行子 agent 提升效率
- 技能质量有规范保障
- 架构决策持久化

负面：
- 并行审查依赖 delegate_task 可用性
- 新增技能需要维护

## 关联
- 前驱：ADR 001、ADR 002
- 涉及：dj-review、dj-meta、dj-dispatch、AGENTS.md
