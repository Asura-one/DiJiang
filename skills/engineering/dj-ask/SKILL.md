---
name: dj-ask
description: "DiJiang 全局 flow 路由图：以 ask-matt 的 flow 模型组织全部 skill，回答『这个任务该走什么流程』『DiJiang 有哪些 workflow』。与 dj-dispatch 分工——dispatch 是逐请求战术路由（一进一出），dj-ask 是全局战略路由（多 skill 串联路径图）。 Use when the user asks what workflow/flow/skill-path to use, or wants an overview of available flows. 触发词：有什么流程、workflow、flow、该走什么、用哪个 skill、技能路线、ask。"
disable-model-invocation: true
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '14071ce1-e0d5-4bd6-9581-7a4da7823827'
  PropagateID: '14071ce1-e0d5-4bd6-9581-7a4da7823827'
  ReservedCode1: 'bb6c9f32-1b0c-429b-aa6b-a2c42d32d63f'
  ReservedCode2: 'bb6c9f32-1b0c-429b-aa6b-a2c42d32d63f'
---

# Ask: DiJiang 全局 Flow 路由图

你不可能记住每个 skill，所以问。路由图会过时：对任何 skill 下断言或跳步前，先读其 SKILL.md 验证——SKILL.md 是事实源，本图只是索引。

技能详注目录：20 个独立工具 + 词汇层 + 代码库健康的详细描述 → `references/skill-catalog.md`。

一条 **flow** 是穿过 skill 的路径。多数沿一条**主流程**走，两条**入口匝道**汇入；其余独立，或是下面运行的词汇层。

## 主流程：idea → ship

1. **`dj-grill`** 逐轮拷问打磨想法。有 working directory 时从此开始：状态化，所学留在 `CONTEXT.md` 和 ADR。
2. **分支：问题需要可运行的答案？**（状态、业务逻辑、UI）→ 原型支路，双向 **`dj-handoff`** 桥接：交出 → 新 session 打开 → **`dj-prototype`** 一次性代码回答 → 带回来。
3. **分支：多 session 构建？**
   - **是** → **`dj-output`** 生成 PRD/spec → **`dj-split`** 拆子弹式工单（各声明**阻塞边**），blockers-first 逐个 kickoff **`dj-implement`**，之间清上下文。
   - **否** → **`dj-implement`** 就在这里，同一上下文窗口内。

   无论哪种，**`dj-implement`** 内部驱动 **`dj-tdd`**（红绿切片），改码被 **`dj-regression-guard`** 三明治护送，完结跑 **`dj-check`**（Standards + Spec 两轴）后提交。单独用 `dj-tdd` 测试先行；`dj-check` 对照固定点审查。

### 上下文卫生

步骤 1–3 保持在**一个不断开的上下文窗口**内（`dj-split` 前不要 compact/clear），拷问、spec、工单建立在同一套思考上；每个 `dj-implement` 从工单全新启动。

### 阶段边界

一个 **阶段** 是 session 内的一块工作（拷问/实现/QA）。**边界**有五选项：**Continue**（零成本）/ **`dijiang-continue`**（恢复任务）/ **`dj-handoff`**（可移植 Markdown：新 harness、新目录、同事、中期分叉）/ **子 agent**（紧 scoped 发自己窗口拿报告）/ **`dj-memory`**（万不得已）。在边界**做**决定；阶段中期 continue 或拆子 agent。

## 入口匝道

起始情境产生工作，汇入主流程：

- **Issue 堆积** → **`dj-triage`**：分诊状态机流转，产出 agent-ready issue 供 `dj-implement` 拾取。只处理**你没创建的** issue；`dj-split` 工单已 agent-ready，不再 triage。
- **东西坏了** → **`dj-hunt`**：一眼看不出的 bug、间歇性 flake、两好状态之间的回归。拒绝在**紧反馈环**建立前理论化。护送走 `dj-regression-guard`；"没有好 seam 锁 bug"时复盘交 `dj-codebase-design`。
- **巨大模糊工作**（绿地/大功能，超一个 session）→ **`dj-wayfinder`**：认知要求最高。画**决策工单地图**一次解决一个，产出**决策而非交付物**直到路清。地图清后**它交出不构建**：在 **`dj-output`** 折叠链接决策汇入主流程，然后 `dj-split` → `dj-implement`。不要从地图直接跳 `dj-implement`，会丢折叠细节。

## 词汇层

model-invoked 参考在其它 skill *下面* 运行，各是词汇的唯一真相源：`dj-domain-modeling`（领域语言/ADR）、`dj-codebase-design`（深模块词汇：seam、适配器、深度——被 `dj-tdd`/`dj-audit` 引用）、`dj-memory`（项目记忆支撑）、`dj-git-guardrails`（Git 安全护栏）。详注见 references/skill-catalog.md。

## 独立工具

主流程之外（详注见 `references/skill-catalog.md`）：

`dj-merge-conflict` 解决冲突永不 abort · `dj-research` 后台调研 · `dj-questionnaire` 拷问发送 · `dj-wizard` 人才能做的步骤脚本化 · `dj-wait-what` 消息没接住的纠正 · `dj-read` 抓 URL/PDF · `dj-reason` 认知矩阵分析 · `dj-write` 去 AI 味润色 · `dj-handoff` session 交接 · `dj-absorb` 吸收外部项目 · `dj-remix` 1:1 复刻+差异化 · `dj-script` 脚本编写 · `dj-design` 前端 UI · `dj-ponytail` 最懒有效方案 · `dj-channel` 多 agent 通道 · `dj-gov` 知识治理 · `dj-meta` skill 写作指南  · `dj-teach` 状态化教学 · `dj-writing-for-agents` agent 文档规范

## Session 管理

| Skill | 角色 |
|-------|------|
| **`dj-setup`** | 项目初始化（首次运行一次） |
| **`dijiang-start`** | 启动 session：加载上下文 → 报告路线 |
| **`dijiang-continue`** | 恢复 session：找活跃任务 → 加载产物 → 报告路线 |
| **`dijiang-finish-work`** | 收尾 session：质量验证 → 提交 → 归档 |

## 与 dj-dispatch 的边界

| | dj-ask | dj-dispatch |
|---|---|---|
| **视角** | 全局 flow 路径（多 skill 串联） | 单次请求路由（一进一出） |
| **触发** | "有哪些 workflow？""该走什么流程？" | "实现登录""修个 bug" |
| **调用模式** | user-invoked | user-invoked |

两者不互斥：先问 `dj-ask` 全局路径，再 `dj-dispatch` 逐请求路由。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 任务跨多条 flow | 指出主干 + 分支，给串联顺序 | 停下问用户目标，再定路径 |
| 路由图与目标 SKILL.md 不一致 | 以 SKILL.md 为准，指出图过时点 | 报告不一致，不硬套旧图 |
| 用户只有模糊想法（雾太大） | 指到 `dj-wayfinder` 或 `dj-grill` | 先澄清一个决策再定路径 |