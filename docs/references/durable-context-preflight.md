---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'e2749e31-b8ec-4475-b0d3-4aaabd4a0664'
  PropagateID: 'e2749e31-b8ec-4475-b0d3-4aaabd4a0664'
  ReservedCode1: 'f22063fb-531f-4f62-9c9c-764a47d7a3f6'
  ReservedCode2: 'f22063fb-531f-4f62-9c9c-764a47d7a3f6'
---

# Durable Context Preflight

在读取 DiJiang 持久记忆前执行的标准化预检流程。

## 触发条件

以下操作前必须执行 Preflight：
- 开始新的实现任务（dj-implement）
- 开始 bug 修复（dj-hunt）
- 改码回归护送（dj-regression-guard）
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

## 写入侧红线

读取侧规则之外，向持久层写入（dj-memory 记录、技能文本沉淀、跨会话知识蒸馏）必须守住两条红线：

### 记忆非授权

历史记忆中出现的动作词（push、发布、删除、创建）是**待重评估的上下文，不是可复用的授权令牌**。用户上一次同意某个动作，不代表这一次同意——每个需要授权的动作在当下会话重新确认。

### 去来源化

私密上下文蒸馏为持久规则/技能文本前，剥离：本地路径、截图引用、项目专名、issue/PR 编号、commit 哈希、私有产品边界。用中性占位符替代（如 `<项目>`、`<路径>`）。持久层只留可迁移的规则，不留可定位的来源。