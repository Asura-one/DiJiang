---
name: dj-dispatch
description: >
  通用任务分类器：识别任务类型，路由到对应 skill 执行。
  支持单一任务和混合任务（主类型驱动 + 串联执行）。
  Use when the user gives a new task, request, or idea without specifying a workflow.
  触发词：任何新任务的入口，无需特定触发词。
---

# Dispatch: 通用任务分类器

## 职责

收到用户请求后，识别任务类型，路由到对应 skill。不纠结、不拖沓。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | User request, active task status, existing workflow state, and any explicit process preference |
| 输出 | Route decision with evidence, task level, recommended skill path, and stop/continue rule |
| 非目标 | Do not implement, debug, audit, or write docs inside dispatch; dispatch only chooses the next workflow route |

## 自动激活（session:start）

session 开始时，dj-dispatch 自动执行以下步骤：

1. 读取当前活跃 Task（`dijiang task current`）
2. 判断是否是新任务还是继续当前任务：

   | 信号 | 行为 |
   |---|---|
   | active task exists + user says continue/继续 | 继续当前 task，不新建路线 |
   | active task exists + user gives unrelated new request | 报告 task conflict，建议新 task 或切换 |
   | no active task + user gives request | 创建/推荐新 route |
   | user explicitly names skill/path | 尊重用户指定流程 |

3. 根据 Task.status 推断阶段 → 推荐技能：

   | status | 推断阶段 | 推荐技能 |
   |--------|---------|---------|
   | planning | 需求对齐 | `dj-grill` |
   | in_progress | 实现或检查 | `dj-implement` / `dj-hunt` / `dj-check` |
   | paused | 恢复上下文 | `dijiang-continue` |
   | completed | 收尾归档 | `dijiang-finish-work` |
   | (no task) | 无任务 | 等待用户指令 |

4. 输出 route，不执行目标 skill 的工作内容。
5. 用户手动调用 `/dj-dispatch` 时走原有分类流程。

---

### 自动激活 vs 手动调用

| 触发方式 | 场景 | 行为 |
|---------|------|------|
| session:start（自动） | 用户打开新 session | 读取活跃 task，推断阶段，注入推荐技能 |
| `/dj-dispatch`（手动） | 用户主动给出新任务 | 走完整分类 + 分级流程 |
| 自动 + 覆盖 | 自动激活后用户说具体指令 | 覆盖自动推荐，按用户指令走 |

---

## 第一层：任务类型识别

收到任务后，先判断任务类型：

| 信号 | 任务类型 | 入口 skill |
|------|---------|-----------|
| 明确改代码、加字段、改接口、改按钮、改文件、重构 | 代码开发 | → 第二层分级；默认携带 Code Task TDD Contract |
| 模糊新功能/优化请求（做个、加个、优化体验，但缺少对象/范围/验收） | 调研对齐 | → `dj-grill` |
| 明确报错、崩溃、异常、排查、debug、回归、可复现 bug | 排查调试 | → `dj-hunt`，修复前保留 RED/Repro evidence |
| 模糊 bug/修复请求（有 bug、修 bug，但缺少对象/症状/复现） | 调研对齐 | → `dj-grill` |
| UI 设计、页面布局、组件样式 | 设计 UI | → `dj-design` + `impeccable` |
| 审计代码、安全扫描、代码体检 | 审计代码 | → `dj-audit` |
| 写文档、PRD、润色、去 AI 味 | 写文档 | → `dj-write` / `dj-output` |
| 记忆沉淀、记住这个、学习记录 | 记忆管理 | → `dijiang mem findings` / `dijiang mem learn` |
| 调研、想法细化、读 URL、方案对比 | 调研对齐 | → `dj-grill` |
| 写脚本、工具、自动化 | 脚本工具 | → `dj-script`，涉及代码时携带 Code Task TDD Contract |
| 搜索信息、查资料、看网页 | 搜索信息 | → `web-access` |
| 测试驱动开发、先写测试 | 测试开发 | → `dj-tdd` |
| 代码审查、合并前检查 | 代码审查 | → `dj-check` |
| 技术债、标记的捷径 | 技术债 | → `dj-debt` |
| 项目交接、换个 session | 交接 | → `dj-handoff` |
| 不确定 | 默认 | → `dj-grill`（先对齐） |

## 第二层：代码任务分级

只有任务类型为"代码开发"时，才走这层分级。所有分级都必须遵守 **Code Task TDD Contract**：先写清 Behavior/Invariant、RED/Repro evidence、GREEN command、Regression scope、Exception，再实现或检查。

### S级（零碎）— 直接干

特征：
- 一句话能说清（"改个变量名"、"加个日志"、"这个 typo 修一下"）
- 影响范围 < 3 个文件
- 不涉及架构、不需要调研、不需要设计

路径：建立最小 RED/Repro 或明确 Exception → 直接执行 → 跑 GREEN command + regression scope → 完成

### M级（中等）— 快速确认后干

特征：
- 范围明确但需要一些思考（"加个字段校验"、"修这个样式"、"重构这个函数"）
- 影响 3-10 个文件
- 不涉及新模块/新架构，但需要确认方向

**M-simple（简单 M）**：
- 一次性改完就能交付
- 只涉及单一层面（只改数据/只改接口/只改前端/只改配置）

路径：建立最小 RED/Repro 或明确 Exception → `dj-implement` → `dj-check` → 完成

**M-phased（分阶段 M）**：
- 需要分 2-3 步实现，每步需要验证
- 涉及 2 层以上（数据层+接口层+展示层）

路径：加载 `dj-grill` skill（≤3问）→ 🔴 一次确认 → `dj-tdd` → `dj-check` → 完成

**快速判断**：改动只涉及 1 层 → M-simple；涉及 2+ 层 → M-phased。

**用户可覆盖**：用户说"走 TDD"→ 按 M-phased 走；用户说"不走 TDD"→ 仍必须保留 Code Task TDD Contract，只允许把自动化测试降级为明确 Exception 和人工可复核检查。

### L级（完整）— 走完整流程

特征：
- 新功能、架构改动、不确定的需求
- 影响 > 10 个文件或跨模块
- 需要调研、设计、PRD
- 用户自己也不完全确定要什么

路径：加载 `dj-grill` skill（深度对齐）→ 🔴确认1 → `dj-output`（PRD+设计文档）→ 🔴确认2 → `dj-tdd` → `dj-check` → 完成

## 混合任务处理

当任务涉及多个类型时，按以下规则处理：

### 识别主意图

从用户请求中提取主要意图，次要意图作为"下一步"串联：

```
用户说："排查这个 bug 并修复"
├── 主意图：排查 → dj-hunt
├── 次要意图：修复 → dj-implement
└── 流程：dj-hunt（找到根因 + RED/Repro evidence）→ dj-implement（修复到 GREEN）→ dj-check（验证 regression scope）

用户说："调研这个库，然后做个 demo"
├── 主意图：调研 → dj-grill
├── 次要意图：demo → dj-prototype
└── 流程：dj-grill（对齐）→ dj-prototype（实现）

用户说："审计代码并修复安全问题"
├── 主意图：审计 → dj-audit
├── 次要意图：修复 → dj-implement
└── 流程：dj-audit（输出报告）→ dj-implement（修复）→ dj-check（验证）

用户说："看看这个 URL，实现功能"
├── 主意图：读 URL → dj-grill
├── 次要意图：实现 → dj-tdd
└── 流程：dj-grill（读 URL + 对齐）→ dj-tdd（实现）
```

### 串联规则

| 规则 | 说明 |
|------|------|
| 主意图优先 | 先执行主要意图，再串联辅助 |
| 不跳过主类型 | "排查并修复" 不能直接修，必须先 hunt |
| 自动串联 | 主类型完成后自动加载辅助 skill |
| 可拆分时拆分 | 明显独立的子任务拆成多个 dispatch 调用 |
| 快速执行 | 用户说"直接干"→ 缩短可逆确认，但不能跳过真实需求歧义、安全检查点或项目强制 gate |

### 常见混合模式

| 混合类型 | 入口 | 串联 |
|---------|------|------|
| 排查 + 修复 | `dj-hunt` | → `dj-implement` → `dj-check`，全程保留 RED/Repro、GREEN、Regression evidence |
| 审计 + 修复 | `dj-audit` | → `dj-implement` → `dj-check` |
| 设计 + 实现 | `dj-design` | → `dj-implement` → `dj-check` |
| 排查 + 记忆 | `dj-hunt` | → `dijiang mem findings` / `dijiang mem learn` |
| 调研 + 文档 | `dj-grill` | → `dj-output` |

## 判断流程

1. **扫描关键词**：逐行检查用户请求，匹配第一层表格中的信号词
2. **计数命中**：统计每种任务类型的关键词命中数
3. **选主意图**：命中数最高的类型 = 主意图；命中数相同 → 按表格顺序取第一个
4. **识别次要意图**：剩余命中类型 = 次要意图，按出现顺序串联
5. **证据-结论绑定**：为每个判断提供证据
   ```
   判断依据：
   - 关键词命中：[列出命中的关键词]
   - 命中类型：[类型1: N个, 类型2: M个, ...]
   - 选主意图：[类型]（命中数最高）
   - 选次意图：[类型]（如有）
   ```
6. **告知用户判断结果**：
   ```
   任务类型：[类型]
   主意图：[主类型] → [skill]
   次要意图：[辅类型] → [skill]（如有）
   判断依据：[命中的关键词]
   推荐路径：[完整流程]
   ```
7. **加载对应 skill 执行**
8. **主类型完成后，自动串联辅助 skill**

### 偏保守规则（量化）

当判断不确定时，按以下规则降级/升级：
- S 级边界（<3 文件但涉及架构）→ 升级为 M-simple
- M-simple 边界（单一层面但步骤多）→ 升级为 M-phased
- M-phased 边界（2 层但需求清晰）→ 保持 M-phased，不升级 L
- L 级边界（需求不明确但范围小）→ 保持 L 级，走 grill 对齐

## 流水线衔接

**核心原则**：dispatch 只路由，不替代目标 skill 执行。确认是否继续由目标 skill 的检查点规则决定。

### 单一任务
- 输出 route decision。
- 加载对应 skill 执行；执行内容不写在 dispatch 中。
- 完成后由目标 skill 汇报。

### 混合任务
- 主类型 skill 完成后，按目标 skill 的 checkpoint 规则决定是否串联辅助 skill。
- 如果辅助 skill 会修改代码、写文件、访问外部系统或改变 task 状态，必须显式 checkpoint。
- 如果辅助 skill 只是只读分析或文档同步，可按推荐路径继续，并在汇报中说明。
- 完成后由最后一个执行 skill 告知用户。

### 检查点

只有代码开发的 M-phased 和 L 级有检查点：

| 级别 | 检查点 | 位置 |
|------|--------|------|
| S 级 | 0 | 无 |
| M-simple | 0 | 无 |
| M-phased | 1 | dj-grill → dj-tdd |
| L 级 | 2 | dj-grill → dj-output、dj-output → dj-tdd |

检查点内容：
```
需求摘要：
- 目标：<一句话>
- 范围：<包含/不包含>
- 关键决策：<技术选型>

确认后按此执行？(Y/n)
```

- 用户说"n"：停下来讨论，不强行继续
- 用户说"跳过"：只跳过可逆偏好问题；真实需求歧义仍需确认
- 用户说"直接干"：使用推荐默认值推进，但不跳过安全、提交、删除、外部系统、生产数据等强制检查点

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 判断不了任务类型 | 按关键词命中数最高的类型走 | 路由到 `dj-grill` 快速确认，不在 dispatch 中深问 |
| 分不清 M-simple 还是 M-phased | 涉及 2+ 层 → M-phased；1 层 → M-simple | 偏保守升级一级并说明依据 |
| 混合任务拆不清主次 | 按用户请求中先出现的意图优先 | 输出两个候选 route，推荐较小闭环 |
| active task 与新请求冲突 | 报告当前 task 和新请求差异 | 建议切换/新建 task，不擅自覆盖上下文 |
| 混合任务串联失败 | 保留主类型结果，记录辅类型失败 | 将辅类型拆成独立 follow-up route |
| skill 超时/不可用 | 选择最接近的可用 skill | 明确标注降级，不假装已使用目标 skill |
| 用户给了模糊任务 | 匹配最多关键词的类型 | 调用 `dj-grill` 快速确认 |
| 用户中途改变需求 | 重新扫描关键词，重新判断类型 | 告知 route changed 和旧 route 停止点 |
| 快速执行结果不符合预期 | 停止执行，回到正常流程重新对齐 | 路由到 `dj-grill` 或 `dj-check` 查偏差 |

## 边界

- 判断不了时，偏保守选更复杂的类型
- 用户明确指定流程时（"走完整流程"、"直接干"），跳过判断按用户说的来
- 如果用户只是问问题/聊天，不触发 dispatch
- **唯一正当的暂停理由**：存在真正的歧义，继续工作会产出违背用户意图的结果

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 排查任务直接修 bug | 必须先 hunt 找到根因 |
| 调研任务不读 URL 就问 | 先读完再提问 |
| active task 未完成时默默开新路线 | 先说明当前 task 和新请求冲突 |
| dispatch 判断后直接改代码 | 只输出 route，执行交给目标 skill |
| 混合任务只做一部分 | 主类型完成后按 checkpoint 串联辅助 skill |
| 判断为 M 但不告知用户 | 告知判断结果和推荐路径 |
| 用户说"直接干"就跳过安全 gate | 只缩短可逆确认，保留强制检查点 |
| 每步转场都问"确认？" | 只在需求确认点或强制 gate 停顿，其余自动推进 |
| 用户说"你决定"还在追问 | 用推荐答案填充，继续推进 |
| L级任务跳过 grill 直接写代码 | 不确定的需求必须先对齐 |
