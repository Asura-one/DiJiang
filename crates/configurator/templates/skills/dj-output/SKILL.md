---
name: dj-output
description: >
  创建和维护项目文档（PRD、设计文档等），并确保文档与代码双向对齐。
  Use when the user needs to create project docs, update docs after code changes,
  or check if docs match the current code.
  触发词：PRD、设计文档、文档、文档对齐、文档更新、docs、documentation、
  代码改了文档没更新、文档和代码不一致、output。
---

# Output: 项目文档管理

## 职责

1. **创建文档** — PRD、设计文档、API 文档等结构化项目文档
2. **维护文档** — 代码变更后同步更新文档
3. **对齐检查** — 发现文档与代码不一致时，双向修复

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Confirmed alignment summary, target document type, source artifacts, target path, and code/spec evidence when updating docs |
| 输出 | Structured document or patch, evidence of code/doc alignment, unresolved assumptions, and save/update summary |
| 非目标 | Do not invent requirements, create new doc locations by default, or change code while producing docs |

## 创建文档

### 🔴 CHECKPOINT · 文档来源门禁

创建、重写或无明确输入时，先自动收集当前任务与已有文档证据，再报告：
```text
文档类型：<PRD / design / API / spec / report / sync>
事实来源：<active task / existing docs / code / user request / none>
目标路径：<已有 task/doc 路径或 proposed path>
已读证据：<files/URLs/code refs>
未决假设：<none or list>
是否修改代码：no
```

默认行为：如果用户使用当前文档技能且没有额外输入，执行文档同步模式：读取 active task、现有 task artifacts、相关 docs/spec 与当前 diff，更新或补齐当前任务相关文档。不要把“缺少显式输入”解释成 bug 排查或自动切换到其他 skill。

🛑 STOP：只有在没有任何来源材料，且用户要求创建会凭空发明产品需求的新文档时，停止并报告“需要需求对齐”；不要在 `dj-output` 内自行切换 skill。

### PRD 模板

```markdown
## Problem Statement
用户视角描述遇到的问题。不是技术问题，是用户的问题。

## Solution
用户视角描述期望的结果。

## User Stories
编号列表，每条格式：
1. As an <角色>, I want <功能>, so that <价值>

要求：尽可能详尽，覆盖功能的所有方面。

## Implementation Decisions
- 涉及哪些模块（新建/修改）
- 接口变更
- Schema 变更
- API 契约
- 架构决策
- 技术选型

不包含具体文件路径或代码片段（会很快过时）。

## Testing Decisions
- 测试切入点（seam）：在哪个层面测试
  - 优先用已有 seam，不需要新 seam
  - 如果需要新 seam，在最高层级提出
  - 理想情况只有一个 seam
- 什么算好测试（只测外部行为，不测实现）
- 哪些模块需要测试
- 已有测试的参考

## Out of Scope
明确列出不包含的内容。

## Further Notes
其他补充。
```

### 设计文档模板

```markdown
## 背景
为什么要做这件事。

## 方案
### 第一性原理推导
- 核心问题：这份设计真正解决什么用户或系统问题？
- 硬事实：哪些现有代码、接口、数据、约束不可改？
- 隐含假设：哪些判断来自推测而非证据？
- 推导结论：为什么当前方案能从硬事实推出？

### 选项对比
| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|

### 选定方案详述
具体设计、接口定义、数据流。

## 影响范围
改动了哪些模块、需要同步更新什么。

## 对抗式审查
从反方视角检查这份文档：
- 逻辑链是否断裂？
- 事实来源是否不足？
- 是否遗漏失败路径、边界条件或验收标准？
- 是否把愿望、推测或实现偏好写成了需求？

## 验证方式
怎么确认方案可行。
```

## 文档与代码对齐

### 主动对齐（代码改了 → 同步文档）

触发场景：实现完一个功能后、check 审查通过后。

流程：
1. 扫描相关文档（PRD、设计文档、README、API docs）
2. 对比代码实际行为与文档描述
3. 列出不一致项：
   ```
   文档 vs 代码不一致：
   - [文档路径:行号] 描述 X → 代码实际是 Y
   ```
4. 修复：
   - 文档过时 → 更新文档
   - 代码偏离设计 → 标记，询问用户该改哪个

### 被动对齐（发现不一致 → 修复）

触发场景：用户指出文档和代码对不上。

流程：
1. 读取文档和对应代码
2. 确定哪个是"真相"（通常以代码为准，除非是设计意图偏离）
3. 更新另一方
4. 如果发现是功能偏移（代码偏离了设计意图），标记并询问用户

## 文档存放

优先使用项目已有的文档结构：

- 当前任务有 `.dijiang/tasks/<task>/` → 放对应 task 下
- 已有 `docs/` 目录且该文档属于长期项目文档 → 放 docs/
- 已有 spec 目录且是规范更新 → 放 `.dijiang/spec/<layer>/`
- 都没有 → 先在对话中输出内容并提出路径，不主动创建新目录或根目录文档

写入前必须明确目标路径。生成任务 artifact 时使用当前 active task 目录。

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 文档和代码对不上，不知道哪个是真相 | 检查 git log 和当前代码行为 | 以代码为准，文档标注 `待确认` |
| PRD 写完用户不满意 | 对照 source gate 找缺失输入 | 标记为“需要需求对齐”，由 workflow 或用户决定是否进入 `dj-grill` |
| 项目没有 docs/ 目录 | 使用 active task 目录或先在对话中输出 | 询问目标路径，不主动创建 docs/ |
| 功能偏移发现后不知道改哪个 | 展示偏移详情，问用户意图 | 文档过时→改文档；设计意图偏离→标记并等用户决策 |
| 缺少代码证据 | 读取相关实现或测试 | 标注 `not verified against code` 和原因 |
| 用户要求边写文档边改代码 | 先完成文档输出并标记实现后续项 | 不在 `dj-output` 中改代码，后续实现由 workflow 或用户显式触发 |

## 🔴 CHECKPOINT · 文档确认

创建/大改文档后：
```
文档：<文档名>
目标路径：<path>
内容摘要：<关键章节>
来源证据：<alignment/code/docs refs>
与代码一致性：<已确认 / 需要对齐 / not verified>
未决假设：<none or list>

确认保存或更新？(Y/n)
```

## TemplateContext 模型

dj-output 使用 DocumentContext 模型分离"文档内容"和"模板渲染"：

### 文档-渲染分离

```
dj-grill 对齐结论
       ↓
   DocumentContext（文档内容模型）
       ↓
   模板渲染 → PRD.md / 设计.md / 合约.md
```

### DocumentContext 字段

| 字段 | 来源 | 用途 |
|------|------|------|
| `problem` | dj-grill 结论 | PRD → Problem Statement |
| `solution` | dj-grill 结论 | PRD → Solution |
| `userStories` | dj-grill 结论 | PRD → User Stories |
| `implementationDecisions` | dj-grill 结论 | PRD → Implementation Decisions |
| `testingDecisions` | 推断 | PRD → Testing Decisions |
| `outOfScope` | dj-grill 结论 | PRD → Out of Scope |
| `optionsComparison` | dj-grill 调研 | 设计文档 → 选项对比 |
| `selectedSolution` | dj-grill 决策 | 设计文档 → 选定方案 |

模板存放位置：`.pi/skills/dj-output/templates/`

---

## Spec 更新合约

当 dj-check 审查发现需更新 spec（或 dj-hunt break-loop 建议沉淀到 spec）时，
dj-output 按以下结构化合约格式输出：

### 触发条件（任一）

- 新增/变更了 API 签名
- 跨层请求/响应合约变更
- 数据库 schema 变更
- 基础设施集成变更
- dj-hunt break-loop 报告建议沉淀到 spec

### 强制章节（7 项）

```markdown
### 1. Scope / Trigger
<触发原因>

### 2. Signatures
<API 签名 / DB 变更>

### 3. Contracts
<Request/Response/Env 字段名: 类型: 约束>

### 4. Validation & Error Matrix
| 条件 | 期望结果 |
|------|---------|
| <条件1> | <期望行为1> |

### 5. Good/Base/Bad Cases
- Good: <正确行为>
- Base: <典型正常输入>
- Bad: <触发 bug 的输入>

### 6. Tests Required
- [ ] <测试描述> — 断言：<具体断言>

### 7. Wrong vs Correct
#### Wrong
<错误写法>
#### Correct
<正确写法>
```

### 写入目标

| 类型 | 路径 |
|------|------|
| 技术规范 | `.dijiang/spec/{layer}/`（backend / frontend / meta） |
| 通用经验 | `.dijiang/spec/guides/`（Thinking Checklist） |

---


## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 代码改了不管文档 | 每次实现后检查文档是否需要更新 |
| 文档和代码不一致时只改文档 | 先判断哪个是真相，再决定改哪个 |
| 没有来源材料就凭空写 PRD | 停止并报告需要需求对齐 |
| PRD 写成散文 | 用结构化模板，每节有明确用途 |
| 文档路径随便放 | 遵循项目已有结构，写入前确认目标路径 |
| 项目没有 docs/ 就主动创建 | 先输出内容或使用 active task 目录 |
| 功能偏移不标记 | 发现偏离设计意图时必须告知用户 |
| 写文档时顺便改代码 | 文档完成后只给实现后续项，不自行切换 skill |
