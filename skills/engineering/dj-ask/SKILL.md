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

你不可能记住每个 skill，所以问。

一条 **flow** 是穿过 skill 的路径。大多数路径沿一条 **主流程** 走，两条 **入口匝道** 汇入它。其余是独立的，或是在下面运行的词汇层。

## 主流程：idea → ship

大多数工作走的路线。你有一个想法，想把它落地。

1. **`dj-grill`** 通过逐轮拷问打磨想法。在有 working directory 时从这里开始：它是状态化的，把学到的东西留在 `CONTEXT.md` 和 ADR 中。
2. **分支：能在对话中解决所有问题吗？** 如果一个问题需要可运行的答案（状态、业务逻辑、需要看到的 UI），走原型支路，双向用 **`dj-handoff`** 桥接（原型在它自己的目录里，这正是 `dj-handoff` 的用途）：
   - **`dj-handoff`** 交出，然后在新 session 中打开那个文件，
   - **`dj-prototype`** 用一次性代码回答问题，
   - **`dj-handoff`** 把学到的带回来，从原始想法线索引用它。
3. **分支：这是多 session 的构建吗？**
   - **是** → **`dj-output`**（生成 PRD/spec），然后 **`dj-split`** 拆分为追踪子弹式工单，每个声明它的 **阻塞边**。工单 blockers-first 执行：逐个 kick off **`dj-implement`**，每次之间清上下文。每个工单自包含，最后一个的上下文用完即弃。
   - **否** → **`dj-implement`** 就在这里，同一个上下文窗口内。

   无论哪种方式，**`dj-implement`** 内部驱动 **`dj-tdd`**（一次一个红绿切片），改码过程被 **`dj-regression-guard`** 三明治护送（改前基线→修改→专项验证→改后回归），完结时跑 **`dj-check`**（两轴审查：Standards + Spec），然后提交。单独用 **`dj-tdd`** 当你只想测试先行地构建一个具体行为；单独用 **`dj-check`** 当你想对照某个固定点审查分支或 PR。

### 上下文卫生

步骤 1–3 保持在 **一个不断开的上下文窗口** 内（`dj-split` 之前不要 compact 或 clear），这样拷问、spec 和工单都建立在同一套思考上。每个 `dj-implement` 然后从工单开始全新启动。

### 阶段边界

一个 **阶段** 是 session 内的一块工作：拷问、实现、QA。在两个阶段之间的 **边界** 你有五个选项：

- **Continue**：留在原地。不花成本，不丢东西。
- **`dijiang-continue`**：恢复任务，加载历史产物，报告下一步路线。
- **`dj-handoff`**：写一个可移植的 Markdown 文件。窄场景：新 harness、新目录、同事、中期分叉。
- **子 agent**：把一个紧 scoped 任务发到它自己的窗口，拿报告回来。
- **`dj-memory`**：压缩上下文并播种新 session。万不得已时用。

在边界 **做** 决定；阶段中期，continue 或把剩余工作拆成子 agent。

## 入口匝道

一个起始情境产生工作，然后汇入主流程。

- **Issue 和请求堆积** → **`dj-triage`**。它把 issue 通过分诊角色状态机流转，产出 agent-ready issue，后续 `dj-implement` 拾取。
  - Triage 只处理**你没创建的** issue：bug 报告、来的功能请求、任何原始到达的东西。`dj-split` 产出的工单已经是 agent-ready，不要再 triage。

- **东西坏了** → **`dj-hunt`**。硬骨头：一眼看不出的 bug、间歇性 flake、两个已知好状态之间爬进来的回归。它拒绝在建立 **紧反馈环** 之前理论化，然后修复并写回归测试。修复的改码护送走 `dj-regression-guard` 三明治协议。它的复盘在发现"没有好的 seam 锁住 bug"时交给 `dj-codebase-design`。

- **巨大的、模糊的工作：绿地项目或大功能构建，超出一个 session** → **`dj-wayfinder`**。认知要求最高的流程。从这里到目的地还看不清路时，它画一张 **决策工单地图**，一次解决一个，产出 **决策而非交付物**，直到雾被推回、路清晰了。
  - 地图清了后，**它交出，不构建**：在 **`dj-output`** 处汇入主流程（把地图的链接决策折叠为可构建的计划），然后 `dj-split` → `dj-implement`。不要从地图直接跳到 `dj-implement`——那会跳过折叠、丢掉链接细节。

## 代码库健康

不是功能工作，只是养护。

- **`dj-audit`** 在有空时运行，保持代码库对 agent 友好。它扫描过度工程和安全性，也可用 `debt` profile（技术债）或 `health` profile（仓库健康）补充。它发现候选；**`dj-codebase-design`**（下文）是你设计选中项的工作台。
- **`dj-pattern`** 从历史修复和代码模式中学习，发现可复用抽象和反模式。

## 词汇层

两个 model-invoked 参考在其它 skill *下面* 运行，各是其词汇的唯一真相源。直接用它们当 **词语**（而非流程）是问题时；或让上面的 skill 拉它们进来。

- **`dj-domain-modeling`**：打磨项目的*领域*语言：挑战模糊术语、解决重载词（一个"账户"干三件事）、把难逆转的决策记为 ADR。它是 `dj-grill` 驱动以保持 `CONTEXT.md` 干净词汇表的主动纪律。
- **`dj-codebase-design`** 是深模块词汇（模块、接口、深度、seam、适配器、杠杆、局部性），用于设计模块的*形状*：小接口后面的大量行为，在干净的 seam 上。`dj-tdd` 和 `dj-audit` 都说它。
- **`dj-memory`** 是项目记忆的底层支撑：其他 skill 通过 `Call the Skill tool with "dj-memory"` 读写历史发现、经验、模式。
- **`dj-git-guardrails`** 是 Git 安全的底层护栏：所有涉及 git 操作的 skill 都经过它的约束。

## 独立工具

完全在主流程之外。

- **`dj-merge-conflict`**：解决进行中的 merge/rebase 冲突，逐 hunk 按 **意图** 追溯到两侧的原始来源，不靠挑行。永不 `--abort`。已在冲突中时用它。
- **`dj-research`**：把阅读腿活委托给后台 agent：它调查问题、查原始来源、在仓库里留一份带引用的 Markdown 文件。你继续干活，它在读。产出拿进主流程的 `dj-grill`。
- **`dj-questionnaire`**：阻塞你的东西不在你脑子里或代码库里，而在**别人**那里时用。它写一份问卷给那个人填。它是 `dj-grill` 的反面：不拷问你关于主题，而是拷问你关于**发送**（给谁、需要什么回来）。回来的东西是 `dj-grill` 或 `dj-output` 的素材。
- **`dj-wizard`**：只有**人类**能做的步骤：配置基础设施、设置凭据或 CI secret、点过陌生的三方仪表盘、跑一次性迁移。生成一个交互式 bash 脚本，开每个 URL、抓每个值、写进 `.env`，让流程不再每次重新跟 agent 解释。
- **`dj-wait-what`**：消息没接住时的纠正。在对话中任何 skill 内部使用，agent 用 `CONTEXT.md` 的词汇重新讲一遍刚才说的，补上你缺的上下文。
- **`dj-read`**：获取 URL 和 PDF，总结或返回干净 Markdown。
- **`dj-reason`**：用认知矩阵和系统透镜分析复杂判断、架构取舍、流程副作用和长期影响。
- **`dj-write`**：文字润色、去 AI 味。
- **`dj-handoff`**：session 交接——把当前对话压缩为结构化交接文档。
- **`dj-absorb`**：有选择地吸收外部项目的设计模式/交互/功能/视觉语言，融合到自有项目中。
- **`dj-remix`**：系统化 1:1 复刻一个网站/App 的界面与功能，复刻后做差异化改造。
- **`dj-script`**：一次性或可复用的自动化脚本编写。
- **`dj-design`**：前端 UI 设计——产出有观点的界面。
- **`dj-ponytail`**：强制最懒但真正有效的方案：最简、最短、最精简。
- **`dj-channel`**：多 agent 协作通道——生成子 agent 执行任务、跨 session 通信。
- **`dj-gov`**：知识治理收尾——审计项目文档/规则/记忆与代码实际行为一致。
- **`dj-meta`**：Skill 写作指南——记录 dj-* 的设计原则和写作规范。
- **`dj-session-insight`**：跨会话记忆检索。
- **`dj-teach`**：跨 session 教一个概念，用当前目录做状态化工作区。
- **`dj-writing-for-agents`**：为 agent 写文档的规范参考。

## Session 管理

覆盖主流程的 session 生命周期。

| Skill | 角色 |
|-------|------|
| **`dj-setup`** | 项目初始化（首次运行一次） |
| **`dijiang-start`** | 启动 session：加载上下文 → 报告路线 |
| **`dijiang-continue`** | 恢复 session：找到活跃任务 → 加载产物 → 报告路线 |
| **`dijiang-finish-work`** | 收尾 session：质量验证 → 版本决策 → 提交 → 归档 |

## 与 dj-dispatch 的边界

| | dj-ask | dj-dispatch |
|---|---|---|
| **视角** | 全局 flow 路径（多 skill 串联） | 单次请求路由（一进一出） |
| **触发** | "DiJiang 有哪些 workflow？""这个任务该走什么流程？" | "实现用户登录""修个 bug" |
| **输出** | flow 路径图（主流程/入口匝道/独立工具） | 目标 skill + 路由理由 |
| **调用模式** | user-invoked | user-invoked |

两者不互斥：先问 `dj-ask` 了解全局路径，再用 `dj-dispatch` 做逐请求路由。