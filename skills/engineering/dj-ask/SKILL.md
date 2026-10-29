---
name: dj-ask
description: "DiJiang 全局路由：flow 路径图 + 单请求分流。回答『这个任务该走什么流程』『DiJiang 有哪些 workflow』，也识别单个新请求并路由到对应 skill（显式输出目标与理由）。 Use when the user asks what workflow/flow/skill-path to use, wants an overview of available flows, or gives a new task that isn't already in a specific skill workflow. 触发词：有什么流程、workflow、flow、该走什么、用哪个 skill、技能路线、ask、新任务、帮忙、做一下、有个想法、有个需求。"
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

# Ask: DiJiang 全局 Flow 路由图 + 单请求路由

你不可能记住每个 skill，所以问。路由图会过时：对任何 skill 下断言或跳步前，先读其 SKILL.md 验证——SKILL.md 是事实源，本图只是索引。

一条 **flow** 是穿过 skill 的路径。多数沿一条**主流程**走，两条**入口匝道**汇入；其余独立。详注（独立工具 / 词汇层 / Session 管理 / 阶段边界）→ `references/skill-catalog.md`；单请求关键词表与歧义消解细则 → `references/routing-table.md`。

## 主流程：idea → ship

1. **`dj-grill`** 逐轮拷问打磨想法。有 working directory 时从此开始：状态化，所学留在 `CONTEXT.md` 和 ADR。
2. **问题需要可运行的答案？**（状态、业务逻辑、UI）→ 原型支路：双向 **`dj-handoff`** 桥接，新 session 里 **`dj-prototype`** 一次性代码回答，带回来。
3. **多 session 构建？**
   - **是** → **`dj-output`** 生成 PRD/spec → **`dj-split`** 拆子弹式工单（各声明**阻塞边**），blockers-first 逐个 kickoff **`dj-implement`**，之间清上下文。
   - **否** → **`dj-implement`** 就地，同一上下文窗口内。

   无论哪种，`dj-implement` 内部驱动 **`dj-tdd`** 红绿切片，改码被 **`dj-regression-guard`** 三明治护送，完结跑 **`dj-check`**（Standards + Spec 两轴）后提交。

步骤 1–3 保持在**一个不断开的上下文窗口**内（`dj-split` 前不要 compact/clear）；每个 `dj-implement` 从工单全新启动。阶段边界的五个切换选项（Continue / continue / handoff / 子 agent / memory）见 catalog 的 Session 管理。

## 入口匝道

- **Issue 堆积** → **`dj-triage`**：分诊状态机，产出 agent-ready issue 供 `dj-implement` 拾取。只处理**你没创建的** issue；`dj-split` 工单已 agent-ready。
- **东西坏了** → **`dj-hunt`**：一眼看不出的 bug、间歇 flake、两好状态间的回归。拒绝在**紧反馈环**建立前理论化；"没有好 seam 锁 bug"时复盘交 `dj-codebase-design`。
- **巨大模糊工作**（超一个 session）→ **`dj-wayfinder`**：画**决策工单地图**一次解决一个，产出**决策而非交付物**直到路清。地图清后**它交出不构建**——在 **`dj-output`** 折叠决策汇入主流程再 `dj-split` → `dj-implement`；直接跳 `dj-implement` 会丢折叠细节。

## 单请求路由

收到具体请求（"实现登录""修个 bug"）而非问全貌时：匹配关键词 → 路由到单个 skill，**显式输出目标 skill + 理由**。

1. **混合任务**取主要类型串联执行（先 `dj-grill` 对齐再 `dj-implement`），不并排路由两个 skill
2. **低置信度** → `dj-grill` 澄清，不可猜
3. **re-entry**：任务仍 `planning` 时，用原始用户请求重新走本 skill 路由，不绕过 worktree check（见 `dj-git-guardrails`）
4. 关键词或歧义拿不准 → 读 `references/routing-table.md`（全量关键词表 + 歧义消解细则）

路由器不是执行器：不替其他 skill 决策、不执行实现、不修改代码；需用户确认的操作（发送、删除、推送）显式提示确认。

## 词汇层

model-invoked 参考在其它 skill *下面* 运行，各是词汇唯一真相源（`dj-domain-modeling`、`dj-codebase-design`、`dj-memory`、`dj-git-guardrails`），详注见 catalog。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 任务跨多条 flow | 指出主干 + 分支，给串联顺序 | 停下问用户目标，再定路径 |
| 路由图与目标 SKILL.md 不一致 | 以 SKILL.md 为准，指出图过时点 | 报告不一致，不硬套旧图 |
| 用户只有模糊想法（雾太大） | 指到 `dj-wayfinder` 或 `dj-grill` | 先澄清一个决策再定路径 |
| 请求超出所有 dj-* 覆盖 | 停下说明，不硬套 | 建议澄清目标 |
