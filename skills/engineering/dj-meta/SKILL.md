---
name: dj-meta
description: "Skill 写作指南：记录 dj-* 的设计原则和写作规范，用于创建和审查技能；也为 agent 写文档（AGENTS.md/CLAUDE.md 及指针文档）提供写作规范。 Use when creating a new dj-* skill, reviewing an existing one, onboarding a new developer, or writing any doc an agent will read. 触发词：写 skill、创建技能、skill 规范、技能指南、agent 文档、AGENTS.md、写作规范、meta。"
disable-model-invocation: true
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
- 跨流程行为约束属于 canonical workflow，不应伪装成可单独路由的 skill

### 4. 输入来自上游，输出流向下游

每个技能都应明确：
- 上游产出是什么（dialogue / PRD / issue / code diff）
- 下游谁接手（`dj-output` / `dj-split` / `dj-implement` / `dj-check` / `dijiang-finish-work`）

### 5. 只写增量纠偏，规则必带 why

写指令前先判断：模型默认会做好这件事吗？会的不写——每条指令都是对模型默认行为的**纠偏**，不是百科全书。全大写 ALWAYS/NEVER 串是坏信号。承重级禁令必须附一句理由：理由在场时模型能判断边界情形；缺 why 的规则是技能自进化时代第一批被"聪明"放宽的承重墙。

### 6. 引用带条件，具体度匹配任务脆弱性

- **条件细节引用**：references/ 引用带触发条件（"If 任务涉及 X：读 `references/<主题>.md`"）；引用深度 ≤1——references 内再引用 references 是反模式
- **自由度校准**：脆弱操作（git 红线、格式破坏风险）写死步骤——窄桥要护栏；灵活任务（排查、设计）给目标不给清单——开阔地允许多条路
- **具体判据**：要求具体到数字、来源、代码、步骤；空泛描述视为未完成

## 为 agent 写文档（含 AGENTS.md/CLAUDE.md）

skill、`AGENTS.md`/`CLAUDE.md`、任何 agent 通过指针到达的文档——包装不同，写作相同：同一套杠杆让每个可预测，因为 agent 每次运行走同一**过程**而非产出相同输出。

- **上下文指针**：agent 上下文中持有的引用。**指针的措辞**而非目标决定 agent 何时到达材料、多可靠——高价值目标配弱措辞是变异 bug：先锐化措辞，锐化失败才内联
- **信息层级**：始终在上下文（指针本身）/ 可按需取（指针目标）/ 几乎不需要（省略或深藏）
- **前置词**（触发词）：覆盖触发场景多种表达、清晰区分用与不用、用领域语言不用空泛词
- **修剪**：一刀砍一半；指令用祈使句；例外和反模式进表格不进散文；参考文件大则 SKILL.md 保持窄

skill 特化机制（invocation 选择/拆分/路由技能）→ `references/agent-doc-mechanics.md`。

## 治理门槛（先于模板）

创建新技能前先过 `SCOPE.md` 双门槛（真实失败 + 契合哲学）；**可由现有技能组合的行为不建新技能**，落点优先是现有技能的增量段落或 `references/` 文档。被否决的概念登记到 `.out-of-scope/`，防止重复提案。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 新技能概念与现有技能重叠 | 并入现有技能的增量段落或 references | 过 SCOPE 双门槛后再定 |
| SKILL.md 超 3000B | 规格/模板/失败表移到 `.dijiang/spec/` | 拆分技能，只留可路由的最小指令 |
| 规则缺 why 被后续放宽 | 补一句理由说明边界 | 承重级禁令升级为 SCOPE 双门槛保护 |

## 创建新技能的模板

完整骨架复制 `references/skill-template.md`（frontmatter/description/职责/工作流/边界/反例六段）。触发词写 description 内供 dj-dispatch 路由；写完回 `SCOPE.md` 双门槛核验。


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
