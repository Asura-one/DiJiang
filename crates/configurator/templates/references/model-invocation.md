# 技能调用分类（Model Invocation）

每个 `dj-*` skill 在 `disable_model_invocation` 字段声明其调用方式。

## 调用类型

| 类型 | disable_model_invocation | 说明 |
|------|--------------------------|------|
| **用户调用** | `true` | 只有用户明确要求时才执行。模型不会自动触发。 |
| **模型调用** | `false` | 模型可以在推理过程中根据上下文自动选择此 skill。 |

## 设计原则

- **路由 skills**（如 dj-dispatch、dj-grill）应为模型调用——它们是模型自动进入流程的入口
- **执行 skills**（如 dj-implement、dj-tdd）应为模型调用——模型在 routing 后自动进入
- **分析/报告 skills**（如 dj-audit、dj-debt、dj-health）应为用户调用——只在用户要求时执行，减少不必要的上下文消耗
- **写作/润色 skills**（如 dj-write）应为用户调用——改写原文需要用户确认
- **检查 skills**（如 dj-check、dj-review）应为模型调用——它们是流程内置的质量门

## 当前映射

| Skill | 调用类型 | 理由 |
|-------|----------|------|
| dj-dispatch | 模型调用 | 新请求入口路由 |
| dj-grill | 模型调用 | 需求模糊时自动对齐 |
| dj-implement | 模型调用 | 执行流程核心 |
| dj-tdd | 模型调用 | 执行流程核心 |
| dj-hunt | 模型调用 | bug 修复核心 |
| dj-check | 模型调用 | 质量门禁 |
| dj-review | 模型调用 | 代码审查 |
| dj-ponytail | 模型调用 | 可叠加编码模式 |
| dj-script | 用户调用 | 只在用户要求时创建脚本 |
| dj-prototype | 用户调用 | 只在用户要求时做原型 |
| dj-design | 用户调用 | 只在 UI 需求时执行 |
| dj-audit | 用户调用 | 全仓扫描，只在要求时执行 |
| dj-debt | 用户调用 | 技术债检查，定期执行 |
| dj-health | 用户调用 | 配置检查，只在要求时执行 |
| dj-pattern | 模型调用 | 代码审查时可自动触发 |
| dj-reason | 用户调用 | 只在需要深度分析时执行 |
| dj-write | 用户调用 | 润色需要用户确认 |
| dj-handoff | 用户调用 | session 结束时的显式交接 |
| dj-output | 用户调用 | 只在要求文档时执行 |
| dj-karpathy | 模型调用 | 编码行为准则，始终有效 |
