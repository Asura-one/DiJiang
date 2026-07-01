---
name: dj-grill
description: >
  深度对齐：通过提问把模糊想法变成清晰需求。
  严格规则：一次只问一个问题，附推荐答案，等用户回复再继续。
  触发词：想法、方案、调研、参考、细化、grill、plan、research、这个URL、看看这个。
---

# Grill: 深度对齐

## 核心规则

1. **一次只问一个问题，等用户回复再继续**
2. **每个问题附推荐答案**，降低用户决策负担
3. **能从代码库找到答案的，去代码库里找，不问用户**
4. **用户说"你决定"时，用推荐答案并告知**
5. **不写代码直到用户批准** — 对齐完成前不写任何代码

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | User idea, references/URLs/files, task level, known constraints, and existing project context |
| 输出 | Confirmed requirement summary with goal, scope, decisions, references, verification, and unresolved assumptions |
| 非目标 | Do not implement, write design docs, or ask broad multi-part questionnaires during alignment |

## 工作流程

### 1. Start Alignment Gate

提第一个问题前，先完成分类：

```text
Task level: <S / M / L>
Known context: <references read / code searched / none>
Question budget: <0-1 / 2-3 / full tree>
Blocking ambiguity: <one sentence>
Recommended default: <answer to use if user says "you decide">
```

🛑 STOP and do not write code until the user confirms the requirement summary or explicitly routes to implementation.

### 2. 有 URL/参考材料时

先读完所有 URL/文件/参考材料，提炼关键信息，总结发现：
```
调研摘要：
- 关键发现：<要点列表>
- 可行方案：<推荐 + 备选>
- 待确认项：<标注不确定的信息>

调研结果 OK？确认后进入提问对齐。(Y/n)
```

### 3. 逐个提问

按以下顺序逐项确认，每项确认后才进入下一项：
- **目标**：做什么、成功标准是什么
- **第一性原理**：用户真正要解决的问题是什么，哪些约束是硬事实，哪些只是默认假设
- **方案**：技术选型、架构方向
- **范围**：包含什么、明确不包含什么
- **约束**：资源、兼容性、迁移、数据、安全限制
- **验证**：怎么确认做完了、验收标准

Question format:

```text
问题：<one question>

推荐答案：<recommended answer and why>

选项：
- A：<recommended option>
- B：<alternative>
```

### 4. 输出需求摘要

全部确认后输出：
```
需求摘要：
- 目标：<一句话>
- 本质问题：<从第一性原理推导出的核心问题>
- 硬事实：<不可改约束 / 已验证事实>
- 隐含假设：<none or list>
- 范围：<包含/不包含>
- 关键决策：<技术选型>
- 参考：<调研中使用的 URL/文档>
- 验证：<验收标准>
- 未决假设：<none or list>

确认后按此执行？(Y/n)
```

## 约束

- **S 级任务**：0-1 个问题；若目标清楚，直接给推荐摘要并等确认
- **M 级任务**：限制 3 个问题以内
- **L 级任务**：不限问题数，走完整决策树
- 用户说"你决定"：使用推荐答案填充，说明采用的默认值，然后继续推进
- 用户长时间不回复或多轮含糊：用推荐方案填充未决项，标注假设并请求最终确认
- 调研只读一个来源就下结论 → 至少对比 2 个来源，不确定的标注"待确认"

## 提问策略

### 1. 自适应深度控制

根据任务级别自动调整提问深度：

| 级别 | 提问数 | 策略 |
|------|--------|------|
| S（一句话） | 0-1 个 | 直接确认目标，跳过方案/范围讨论
| M（确认后执行） | 2-3 个 | 目标 → 方案。范围/约束/验证只在不确定时问
| L（需设计） | 不限 | 完整决策树（目标→方案→范围→约束→验证），逐项确认

### 2. 推荐答案的构造方法

每个推荐答案应包含：

**默认值推导** — 优先从代码库中找答案：
```bash
# 找类似功能的实现方式
grep -rn "feature\|pattern\|similar" src/ --include="*.rs" | head -5

# 找项目已有的技术选型
ls Cargo.toml package.json go.mod 2>/dev/null | xargs head -20
```

**选项呈现** — 每个问题至少提供 2 个选项，标注推荐项：
```
问题：<问题>

选项：
- A：<方案描述>（推荐）
  - 优点：<快/简单/一致>
  - 代价：<维护/学习/兼容>
  - 参考：<代码库中类似实现或外部文档>
- B：<方案描述>
  - 优点：...
  - 代价：...
```

如果不确定哪个更好，标注"待确认"并说明不确定的原因。

### 3. 代码库感知建议

在每个阶段检查代码库现状：

**目标阶段**：
```bash
# 检查是否已经有相关功能
grep -rn "类似功能名\|相关术语" src/ --include="*.rs" | head -5
```

**方案阶段**：
```bash
# 检查项目中已有的类似实现模式和架构
# 检查依赖是否已存在
grep -rn "包名\|库名" Cargo.toml 2>/dev/null
```

**范围阶段**：
```bash
# 检查涉及的文件和模块范围
find src/ -name "*.rs" | head -20
```

如果代码库中已有明显可参考的模式，直接引用作为建议的一部分。

### 4. 追问生成模式

根据用户对上一个问题的回答，自动选择追问方向：

| 用户回答 | 追问策略 |
|---------|---------|
| 选了 A 方案 | 直接进入下一项（范围/约束），不追问方案细节

| 选了 B 方案但未解释 | 不追问，直接用 B 继续
| 没选，说"你决定" | 用推荐项填充并告知
| 说"再看看"或犹豫 | 提供更细化的对比（性能/复杂度/维护成本）
| 提了新需求 | 回到目标阶段重新确认范围
| 说"都行" | 从推荐项中选，继续推进
| 反问"你觉得呢？" | 给出推荐+理由，继续等确认

### 5. Gap 检测（对齐完成后）

对齐完成、输出需求摘要后，在确认前执行 gap 检测：

```
GAP DETECTION:

1. 检查是否有隐式假设未声明（如性能要求、兼容性）
2. 检查 `.dijiang/spec/` 中是否有相关规范未被引用
3. 检查是否有上游/下游依赖未纳入范围
4. 检查是否有替代方案被忽略但值得对比

Gap found? (Y/n)
```

有 gap → 追加 1 个问题覆盖，然后再确认。无 gap → 输出需求摘要。

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 用户连续给出含糊回答 | 复述理解并给推荐答案 | 用推荐答案填充，标注假设，进入摘要确认 |
| 用户不断扩大范围 | 回到范围问题，只问包含/不包含 | 输出两个范围版本并推荐较小版本 |
| URL/资料无法读取 | 说明读取失败和缺口 | 基于可用上下文继续，标注 `待确认` |
| 发现必须先调研代码 | 暂停提问，搜索代码库 | 引用代码证据后再问一个问题 |
| 用户要求立刻实现 | 输出当前已确认摘要 | 标记“可进入实现后续”，不在 grill 中写代码或自行切换 skill |
## Phase 标记

dj-grill 在需求对齐完成后，将当前 task 标记为对应 phase。
每个 dj-* 技能在启动时检查 phase，确保在正确阶段继续。

### Phase 映射

| dj skill | 标记 phase | 说明 |
|----------|-----------|------|
| `dj-grill` | `align` | 需求对齐中 |
| `dj-output` | `design` | 产出 PRD / 设计文档 |
| `dj-implement` / `dj-tdd` | `implement` | 代码实现中 |
| `dj-hunt` / `dj-check` | `verify` | 验证 / 审查中 |

### 标记方式

```bash
# 对齐完成后标记 phase（用 dijiang task status 设置对应状态）
dijiang task status <task-name> planning

# planning → phase=align（DiJiangTaskRecord 推断规则）
# in_progress → phase=implement
# completed → phase=complete
```

### 自动恢复

dj-grill 启动时检查：

- 当前 task phase 为 `align` 且上次对话 < 24h → "继续上次对齐？还是重新开始？"
- 当前 task phase 为 `design` 或 `implement` → "之前已经对齐完了，直接继续？"
- 无活跃 task → 正常对齐流程

### 推进

- M-simple 级任务：dj-grill 完成后 phase 直接跳到 `implement`
- M-phased / L 级任务：dj-grill 完成后 phase = `design`（等待 dj-output）
---


## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 一次问多个问题 | 严格一次一个问题 |
| 没调研就问"你想用什么方案" | 先调研再推荐 |
| 用户给了 URL 不读就开始问 | 先读完再提问 |
| 每个问题只问"你想怎么做？" | 每个问题必须附推荐答案 |
| 用户说"你决定"还在继续追问 | 用推荐答案填充，继续推进 |
| 对齐完了不输出需求摘要就直接写代码 | 必须先输出需求摘要，等用户确认 |
