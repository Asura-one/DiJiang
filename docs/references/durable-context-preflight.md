# Durable Context Preflight

在读取 DiJiang 持久记忆前执行的标准化预检流程。

## 触发条件

以下操作前必须执行 Preflight：
- 开始新的实现任务（dj-implement）
- 开始 bug 修复（dj-hunt）
- 设计或架构决策（dj-design、dj-reason）
- Session 交接（dj-handoff）
- 模式识别（dj-pattern）

不需要 Preflight 的场合：
- 纯路由（dj-dispatch）
- 需求对齐（dj-grill）
- 即时代码审查（dj-review、dj-check）
- 写作润色（dj-write）

## 预检流程

```
1. 判断任务是否需要历史记忆
   ├─ 新功能/新模块 → 不需要（或仅限项目级配置）
   ├─ bug 修复 → 需要（查找类似 bug 的历史 pattern 和 fix）
   ├─ 架构变更 → 需要（查找相关 ADR 和设计决策）
   └─ 交接 → 必须（读取完整 session 上下文）

2. 读取优先级
   ├─ 第 1 优先: active task artifacts（prd.md、design.md、implement.md）
   ├─ 第 2 优先: findings（任务期间的关键发现）
   ├─ 第 3 优先: lessons（项目级学习记录）
   ├─ 第 4 优先: ADR（架构决策记录）
   └─ 第 5 优先: handoff（session 交接文档）

3. 记忆定界
   ├─ 只读取与当前任务 DIRECTLY 相关的记忆
   ├─ 记忆超过 3 个月的标为"可能过期"
   └─ 冲突记忆需要用户确认
```

## 记忆类型与使用方式

| 类型 | 怎么用 |
|------|--------|
| decision / preference / principle | 作为约束条件，不询问直接遵守 |
| pattern / learning | 作为可复用检查项，运行验证 |
| fact | 需要自己验证，不直接取用 |
| task_artifact | 作为当前任务的已知上下文 |
