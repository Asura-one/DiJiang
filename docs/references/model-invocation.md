---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'b96e337d-a0aa-4d9d-bd62-3f9bfded1a5e'
  PropagateID: 'b96e337d-a0aa-4d9d-bd62-3f9bfded1a5e'
  ReservedCode1: 'f1fbdde4-2706-4d46-9d12-2135bb4bf003'
  ReservedCode2: 'f1fbdde4-2706-4d46-9d12-2135bb4bf003'
---

# 技能调用分类（Model Invocation）

每个 `dj-*` skill 在 `disable_model_invocation` 字段声明其调用方式。

## 调用类型

| 类型 | disable_model_invocation | 说明 |
|------|--------------------------|------|
| **用户调用** | `true` | 只有用户明确要求时才执行。模型不会自动触发。 |
| **模型调用** | `false` | 模型可以在推理过程中根据上下文自动选择此 skill。 |

## 设计原则

- **路由 skills**（如 dj-dispatch、dj-ask、dj-grill）应为模型调用或用户调用——dispatch 是逐请求战术路由，dj-ask 是全局战略路由图，两者都是用户主动触发
- **执行 skills**（如 dj-implement、dj-tdd）应为模型调用——模型在 routing 后自动进入
- **分析/报告 skills**（如 dj-audit）应为用户调用——只在用户要求时执行，减少不必要的上下文消耗
- **写作/润色 skills**（如 dj-write）应为用户调用——改写原文需要用户确认
- **检查 skills**（如 dj-check、dj-review）应为模型调用——它们是流程内置的质量门

## 当前映射

| Skill | 调用类型 | 理由 |
|-------|----------|------|
| dj-dispatch | 模型调用 | 新请求入口路由 |
| dj-ask | 用户调用 | 全局 flow 路由图（用户主动问有哪些流程） |
| dj-grill | 模型调用 | 需求模糊时自动对齐 |
| dj-implement | 模型调用 | 执行流程核心 |
| dj-tdd | 模型调用 | 执行流程核心 |
| dj-hunt | 模型调用 | bug 修复核心 |
| dj-regression-guard | 模型调用 | 改码三明治回归协议（任务级前置纪律） |
| dj-fullstack-testing | 模型调用 | 全量回归引擎（全量模式时触发） |
| dj-check | 模型调用 | 质量门禁 |
| dj-review | 模型调用 | 代码审查 |
| dj-script | 用户调用 | 只在用户要求时创建脚本 |
| dj-prototype | 用户调用 | 只在用户要求时做原型 |
| dj-design | 用户调用 | 只在 UI 需求时执行 |
| dj-audit | 用户调用 | 使用 default、debt 或 health profile 的全仓扫描 |
| dj-pattern | 模型调用 | 代码审查时可自动触发 |
| dj-reason | 用户调用 | 只在需要深度分析时执行 |
| dj-write | 用户调用 | 润色需要用户确认 |
| dj-handoff | 用户调用 | session 结束时的显式交接 |
| dj-output | 用户调用 | 只在要求文档时执行 |