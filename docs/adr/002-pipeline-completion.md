# ADR 002: 补齐 grill → PRD → split → implement 流水线

## 状态
`accepted`

## 上下文
对比 mattpocock/skills 的完整流水线（grill → to-prd → to-issues → implement），dj-* 在 grill 和 implement 之间缺少关键环节：
- 没有"把对齐结果变成正式文档"的技能
- 没有"把 PRD 拆分为独立可执行任务"的技能
- 没有"共享术语持续维护"的机制

## 决策
1. 创建 `dj-prd`：接收 `dj-grill` 的需求摘要，输出结构化 PRD 文档到 `.dijiang/prd/`
2. 创建 `dj-split`：接收 PRD，拆分为独立任务列表，每个任务有独立验收标准
3. 扩展 `dj-grill`：在提问阶段检查 `.dijiang/glossary.md`，需求沉淀时更新术语
4. 创建 `.dijiang/glossary.md`：项目共享术语表
5. 更新 `AGENTS.md` 路由和工作流

## 影响
正面：
- 完整的开发前管道：grill → prd → split → implement
- 共享术语减少沟通成本
- 任务拆分提供更细粒度的执行单元

负面：
- 新增 2 个技能需要维护
- 需要培养"先写 PRD 再拆分"的习惯

## 关联
- 前驱：ADR 001（精简后的技能框架）
- 涉及：dj-grill、dj-prd、dj-split、dj-dispatch、AGENTS.md
