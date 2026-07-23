---
name: dj-meta
description: >
  Skill 写作指南：记录 dj-* 的设计原则和写作规范，用于创建和审查技能。
  Use when creating a new dj-* skill, reviewing an existing one, or onboarding a new developer to the skill system.
  触发词：写 skill、创建技能、skill 规范、技能指南、meta。
summary: DiJiang 架构自省、技能创建指南和系统理解
phases: [align, implement, check, finish]
risk: low
  Skill 写作指南：记录 dj-* 的设计原则和写作规范，用于创建和审查技能。
  Use when creating a new dj-* skill, reviewing an existing one, or onboarding a new developer to the skill system.
  触发词：写 skill、创建技能、skill 规范、技能指南、meta。
---

# Meta: Skill 写作指南

创建和审查 `dj-*` 技能的设计规范。

## 核心原则

### 1. SKILL.md 只保留执行指令

SKILL.md 是 **agent 执行时必须逐条读的最小指令集**。不是参考文档、不是培训材料、不是项目管理文件。

| ✅ 保留 | ❌ 移到 `.dijiang/spec/` |
|---|---|
| 职责（一句话） | 输入/输出规格表 |
| 工作流步骤（编号列表） | 失败处理表 |
| 核心规则（3-5 条） | CHECKPOINT 完整模板 |
| 精简短例（3-4 条） | 深度设计原则 |
| 验证命令 | 模板化内容 |

### 2. 技能体量控制

```
目标：800-2500B（约 40-100 行）
硬上限：3000B（超过说明塞了非指令内容）
```

### 3. 聚焦、可组合

- 每个 skill 只做一件事，做好
- 用 pipeline 串联多个技能，不在一个技能里做所有事
- 如果技能需要"叠加"使用（如 `dj-ponytail`），在描述中说明

### 4. 输入来自上游，输出流向下游

每个技能都应明确：
- 上游产出是什么（dialogue / PRD / issue / code diff）
- 下游谁接手（`dj-output` / `dj-split` / `dj-implement` / `dj-check` / `dijiang-finish-work`）

## 创建新技能的模板

```markdown
---
name: dj-<name>
description: >
  <一句话说清楚做什么>
  Use when <触发条件，agent 判断依据>
  触发词：<用户说的词，帮助 dj-dispatch 路由>
---

# <Title>: <中文描述>

## 职责

<一段话说明这个 skill 负责什么、不负责什么>

## 工作流

### 1. <步骤 1>

<指令>

### 2. <步骤 2>

<指令>

### 3. <步骤 3>

<指令>

## 边界

- <不做的事>

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| <反例> | <正例> |
```

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| SKILL.md 超过 3000B | 精简到 800-2500B |
| 把培训材料写进技能 | 移到 `.dijiang/spec/` |
| 一个技能做多件事 | 拆成多个技能串联 |
| 技能之间内容重复 | 引用已有的技能 |

## 参考

- `references/skill-fusion-pattern.md` — 跨仓库技能分析与融合方法论，用于从外部 skill 项目中提取模式并融合到本地技能库
- `.dijiang/spec/` — 各技能的辅助参考文档（输入/输出、失败处理、原则等）
