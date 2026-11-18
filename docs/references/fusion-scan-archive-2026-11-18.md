# 参考项目融合扫描归档 · 2026-11-18

> 本文件为归档详情（候选卡片全表 + 三批落地记录 + 审计口径），速览与当前结论见 `docs/guide/fusion-scan-2026-11-18.md`。

# 参考项目融合扫描 · 2026-11-18

> 扫描对象：`referens/` 六个参考项目 + 校样台对比报告（https://c92r26.vnxt.cc/#/routing）+ 琅嬛知识库 11-09 后增量
> 方法：5 个只读子代理并行拆解 → 主 session 归并去重 → 逐条回源码四态重验（已有 / 真实差距 / 前提失效 / 不适合）
> 结论口径：只收「机制级」可迁移点；按 `SCOPE.md` 门槛，假设性改进不收；落点优先现有技能增量，不新建重复技能。

---

## 一、输入清单与版本基线

| 来源 | 版本 / 状态 | 本轮取用 |
|---|---|---|
| `referens/agent-skills`（Addy Osmani） | plugin 0.6.16，25 技能 | 全量技能 + `evals/` + `references/definition-of-done.md` |
| `referens/skills`（mattpocock） | 分叉基线 `1631f1c` 之后 87 提交，`skills/` 46 文件 +822/-184 | 增量技能：pr / implement-spec / retro / glossary / software-factory / chief-of-staff |
| `referens/pstack` | 0.15.43（本地镜像）；`SYNC-LOG` 2026-11-18 无上游变更 | worker.md、sharpen、ketchup、blast-radius、figure-it-out、swarm、statechart、benchmark-checklist |
| `referens/Waza` | 3.42.0 | health（prune 过度删除审计 + `check_moved_lines.py`）、check（skeptic）、anti-patterns、write |
| `referens/ponytail` | 5.3.3 | ponytail ultra 语义、review 负载判据 |
| `referens/Trellis` | 最新 PR #629 | 仅判断归档/pathspec 修复，无迁移价值 |
| 校样台对比报告 | 26 个生命周期阶段 × Addy/Matt/Pstack 三套 | 覆盖缺口地图（下节） |
| 琅嬛知识库 | 11-11、11-17 两批 ingest | GEPA 冻结口径、skill-doctor、验证者≠发现者、description 签名 |

**注意一个版本差**：校样台里 pstack 列出的 `deslop`、`thermo-nuclear-review`、`make-pr-easy-to-review` 在本地 0.15.43 镜像中不存在，来自更新上游。本轮以本地镜像为准；这三项记为「上游可见、本地未取」，需要时先更新镜像再评估。

---

## 二、校样台给出的缺口地图（结论先行）

26 个阶段里，DiJiang 与 Matt（母体）主干几乎逐段对齐，与 Pstack 的原则层重叠，真正的空白集中在三类：

1. **Addy 独占的七个域 checklist**：前端可访问性、性能、可观测性、CI/CD、废弃迁移、发布前清单、质量约束。DiJiang 在这些域没有独立流程，但也没有对应的真实失败记录 → 本轮按门槛**不收**，列观察项。
2. **项目级质量契约**：三方里只有 Addy 把「质量线」写成带命令的四列表；DiJiang 所有门禁都在任务级验证管道，没有一层在 diff 级守「门禁本身不被削弱」。
3. **多 agent 治理层**：Matt 有 phase-boundaries，Pstack 有新加的 worker 契约与委托验收，DiJiang 的 dj-channel 只有角色边界，没有开工契约、产出验收、放行文书、待拍板播报。

---

## 三、候选卡片

四态：**真缺口** / **部分覆盖**（说明已有部分）/ **已有** / **不适合**。价值按「真实失败背书 + 复用面 + 成本」排序。

### A 批：门禁与验证强化（7 项）

| # | 机制 | 来源证据 | DiJiang 四态 | 落点 | 成本 |
|---|---|---|---|---|---|
| A1 | **质量契约 + floor-guard 五招 + ratchet**：CONSTRAINTS 四列表（维度/规则/裁决命令/运行位置），无命令的数字不算约束；diff 级检测五种「最便宜变绿」动作（阈值下调、测试变易、检查器被禁、未完成 stub、例外表凭空多行）；无目标时记录今日值只许变好 | `agent-skills/skills/constraint-driven-development/SKILL.md:110-124,205-219`；`references/floor-guard.md`（exit 0/1/2，**2 不得读作 0**） | 部分（只有「禁止改/删/跳失败测试」一条铁律，无项目级契约，无 diff 级检测） | `dj-regression-guard/references/protocol-details.md` 增节 + `skills/engineering/dj-regression-guard/scripts/floor_guard.py` | 中 |
| A2 | **触发层实测**：description 是路由杠杆而非简介；每个触发子句配 fire / stays-quiet 配对案例；报数字必须钉死模型与 CLI 版本；给 description 加「即使内容被内联粘贴也适用」子句的实测效果 7/27→21/27 | `agent-skills/evals/README.md:71-76` | 真缺口（有行为层 `tests/behavior-cases.json`，无触发层；11-18 全量换精简 description 后从未实测） | `dj-meta/references/trigger-testing.md`；dj-evolve 验证步骤挂触发层核对 | 低 |
| A3 | **审查发现的反驳复核 + 严重度语义 + 结构处方 + 突变式审查**：每条发现是待验证命题，直读即溶解的降级；Critical/Required/Nit/FYI 四档语义（Nit 作者可忽略）；结构问题必须给命名的重构动作；反转改动的一个条件跑套件，保持绿=发现缺失测试 | `Waza` commit `6b6c736`（skeptic）；`agent-skills/skills/code-review-and-quality/SKILL.md:164,189` | 部分（「checker 不读 maker 叙事」已覆盖；缺发现复核、Nit 语义、结构处方、突变式审查） | `dj-check/references/review-dual-axis.md` 增量 | 低 |
| A4 | **单一安全事实 + 五级证据阶梯**：先找出整个安全性所依赖的一两个事实并用运行代码证明；每个风险主张标注停在 ①口头→②file:line→③走完失败路径→④脚本证伪→⑤运行时复现 | `pstack/skills/blast-radius/SKILL.md:17,21-27` | 部分（证据三态管「有没有证据」，不管「风险主张停在哪级」） | `dj-check/references/delivery-risk.md` 增两栏 | 低 |
| A5 | **实现期版本敏感文档验证**：从依赖文件读精确版本 → 抓具体文档页（SO/博客/AI 生成内容永不作一手来源）→ 与现有代码冲突显式上报 → 全 URL 深链或写 UNVERIFIED | `agent-skills/skills/source-driven-development/SKILL.md:76-81,176` | 真缺口（dj-research 管调研期，dj-implement 无实现期验证纪律） | `dj-implement/references/code-task-contract.md` 增段 | 低 |
| A6 | **已提交瘦身的事后审计**：`check_moved_lines.py <base> --head <rev>` 机械列出两次修订间消失的指令行，逐行对台账；由没参与瘦身的复核者 grep 关键词复核，命中即原文恢复 | `Waza` commit `6095341`：30 个 prune 提交复查抓到 5 条被误删约束 | 部分（context-diet 步骤 11/12 有等价 prose，无机械列举） | `dj-meta/scripts/check_moved_lines.py` + context-diet 步骤 11 挂一行 | 低 |
| A7 | **分发态坍塌反模式**：声称 installed/released/done 前必须逐分发层单独取证（源/包内容/安装运行时/registry/生效面），缺层是显式 gap，不默认通过 | `Waza/rules/anti-patterns.md` row 28 | 部分（dj-gov 有 merged/deployed/live verified，但不常驻） | `dj-meta/references/anti-patterns.md` 加一行 | 极低 |

### B 批：多 agent / 无人值守治理（6 项）

| # | 机制 | 来源证据 | DiJiang 四态 | 落点 | 成本 |
|---|---|---|---|---|---|
| B1 | **worker 开工契约 + 注入协议**：spawn 子代理时 kickoff 第一行强制读 `worker.md`；契约写清完整验收线、不得自行加门禁、可撤销自行决定、只在四类不可逆事件停、绝不调用会弹问人的工具 | `pstack/skills/poteto-mode/worker.md:1-11` | 真缺口（dj-channel 只有 Scout/Builder/Verifier 角色边界，无操作者契约） | `dj-channel/references/worker-contract.md` + spawn 一节注入规则 | 低 |
| B2 | **待拍板播报事务**：假定用户只记得最后一条消息；本轮只读；<100 字白话进展 + 每个待决项 ~30 字（链接/影响/证据/风险/推荐）；「回复本身必须足以拍板」；不编造待决项凑数 | `pstack/skills/ketchup/SKILL.md:9,23` | 部分（dj-decision 有 Attention 段，dijiang-continue 只恢复状态） | 并入 `dj-decision` 一节，或新 `dj-recap`（需路由三处） | 低 |
| B3 | **委托产出验收协议**：每个委托单元配裁判；worker 骗过门禁→重置并加固契约，门禁本身错→单独改动修门禁；产出必须记录 brief 指定的 SHA 与方法，缺失丢弃重派一次，二次失败记 gap，gap 不算通过 | `pstack/skills/figure-it-out/SKILL.md:41`、`swarm/SKILL.md:39` | 真缺口 | `dj-channel` 增「worker 产出验收」节 | 低 |
| B4 | **模型路由三护栏**：任何角色不超 xhigh；slug 被拒即拉黑（当次会话内不重试） + 四级 fallback；昂贵模型例外门只认「用户点名」或「同 brief 两次失败」 | `pstack/skills/poteto-mode/SKILL.md:94,96` | 部分（已有模型分层，无护栏） | `dj-channel` 模型节三行 | 极低 |
| B5 | **任务图并行 frontier + 集成分支 + merger**：ticket 是任务图不是列表，永远有可领取 frontier；每票全新 implementer 子代理各自 worktree，先确认基于集成分支；完成后由独立 merger 合入；全票完成后整支一次 review | `skills/skills/engineering/implement-spec/SKILL.md:29-34` | 部分（dj-split 有阻塞边与并行标注，dj-implement 串行 blockers-first） | `dj-split/references/task-graph.md` + dj-implement 增「并行 kickoff 档」 | 中 |
| B6 | **放行文书**：PR body 三段——Summary（最小可视化）、Evidence（Before/After 成对）、Merge Danger（单向门/双向门 + 一词爆炸半径） | `skills/skills/engineering/pr/SKILL.md:14-32` | 真缺口（放行靠逐个读 diff） | `dijiang-finish-work` 增「放行申请」输出档 | 低 |

### C 批：微增量（11 项，单行或小段）

| # | 机制 | 来源 | 落点 |
|---|---|---|---|
| C1 | 假设附 0-100% 置信度；「如果不用向任何人交代，你实际想要什么」探针；「whatever you think」是委托不是决策，换两个具体选项重问 | agent-skills interview-me:90,115-136 | dj-grill 增量 |
| C2 | seam 注记（每条 seam 写清能抓什么/漏什么）；红是突变造出来的要 diff 证明落盘；答案位置轮换；wayfinder:<type> 标签独占 + 第二遍连边用真实编号 + research 分支只推不合并 | skills 分叉后微修 | dj-tdd / dj-hunt / dj-teach / dj-wayfinder 各一行 |
| C3 | ponytail `ultra` 语义补「动手前质疑需求本身」 | ponytail/SKILL.md:52 | dj-ponytail 一行 |
| C4 | 词汇表纪律：每词带 `_Avoid_` 别名压制、通用编程概念不入表、多上下文用 MAP 声明关系（**不随行改名 CONTEXT→GLOSSARY**） | skills/domain-modeling/GLOSSARY-FORMAT.md:37 | dj-domain-modeling/references/context-format.md |
| C5 | 记忆断言带日期 + commit 引用；禁写「currently」；变更追加新断言并标旧的 superseded，不改写 | skills/chief-of-staff/SKILL.md:44-47 | dj-memory references |
| C6 | 岔路按剩余上下文预算估算；切片上限 = 在全新会话里能否稳定做完（Context Rot：18 模型实测非均匀下降） | 琅嬛《六阶段 AI 工作流风险地图》 | dj-ask 主流程岔路 + dj-split 原则各半句 |
| C7 | 报告任何性能数字前的七问（限在哪 / 双方是否按生产配置调优 / 算术破限 / 错误数了没 / 5 次可复现 / 占端到端多少 / 工作真的发生了吗） | pstack benchmark-checklist:27 | dj-check references |
| C8 | 形式化验证两轻量机制：每个 check 先在破坏变体上证明会失败；覆盖记录（文件/函数 + 最后确认 commit），代码变动后验证需复核 | pstack create-formal-verification-skill:24,29 | dj-verify references（完整 TLA+ 工具链挂账） |
| C9 | loop 刹车词汇：run 永不等人（人通过 tracker 重入）；每个 trigger 都是加速器，没刹车是 slop cannon（自动检查/自动评审/人工评审三档）；先节流后开阀；每任务五问（含如何防双跑） | skills/software-factory/SKILL.md:56-62 | dj-loop references |
| C10 | 进化口径冻结 + 逐 case 归因（子版本只与直接父版本按失败 case 比较，非平均分）；存量体检补「从未触发=路由盲区或退休候选」「接受率」「覆盖率」 | 琅嬛《Agent Skill 自进化》11-17 节；dj-evolve §5 已有雏形 | dj-evolve references |
| C11 | 知识/规则分层裁决序：references 是建议、SKILL.md 判据与反模式是命令，冲突时命令优先 | 琅嬛《Agent Skills 进阶设计模式》 | dj-meta anti-patterns 一行 |

### 观察项（本轮不做，写明理由）

| 项 | 理由 |
|---|---|
| Addy 七个域：frontend-ui-engineering / performance-optimization / observability / ci-cd / deprecation / security-and-hardening / shipping-and-launch | 无真实失败背书；部分已被 dj-design / dj-audit / dj-check / dj-hunt 覆盖。出现首个真实漏判再收。 |
| software-factory 完整服务槽词汇表 | 词汇冲突痛点尚未在本仓发生第二次；只收 C9 的刹车语义，整表留给 pentest-core 设计时按需引用。 |
| Skill 工厂（Skill IR / 多平台编译 / SkillOps） | 团队级多平台治理，对单人 44 技能纯文本体系过重，且引入平台锁定。 |
| 总控 Skill 自动编排 | 与 dj-ask 分流重复，且与「对外动作逐项放行、一次一个变量」冲突。 |
| Trellis 状态层（workflow 状态机、task tree、hooks、spec manifest） | ADR-005 已退役纯 skill 架构；近期改动全在其自属状态层。 |
| ponytail band / board / statusline | 编辑器 hook 注入层，纯 skill 库无载体。 |
| 双语发布说明配对 | 现有项目全部中文单语，为不存在的工作流加规则。 |
| CONTEXT → GLOSSARY 改名 | 涉及 dj-ask/dj-hunt 多处引用与「CONTEXT 兼架构速览」的取舍，属架构决策，需单独拍板。 |
| 校样台的 deslop / thermo-nuclear-review / make-pr-easy-to-review | 本地镜像 0.15.43 未包含，需先更新镜像。 |

---

## 四、实施约束（来自 dj-absorb / dj-meta）

- 不新建重复技能：除 B2 待定外，全部走「现有技能增量 / references / scripts」。
- 脚本落技能内 `scripts/`，随技能目录整目录投放；仓库根 `scripts/` 不放技能专用脚本。
- SKILL.md 正文 3000 字符红线；references 不受红线约束，但按 context-diet 纪律只写判据不写教程。
- 每条落地前先写进本文件对应的卡片（方案 → diff → 验证），一条一条做，不堆验收。
- 任何写库/提交动作前展示 diff 待确认；push / 合并 main / live 同步逐项待放行。

---

## 五、拍板记录（2026-11-18）

| 决策 | 结果 |
|---|---|
| 吸收范围 | A 批先做，B 批随后；C 批微增量在动手时顺手并入 |
| A1 形态 | references + 通用脚本（`dj-regression-guard/scripts/floor_guard.py`，自带八类突变自测） |
| A2 深度 | runbook + 一次性结构体检；真实火率等有独立评测入口再补 |
| A3 分档 | 改四级：阻塞 / 应改 / Nit / FYI |

## 六、A 批落地记录（2026-11-18）

| # | 落点 | 验证 |
|---|---|---|
| A1 | `dj-regression-guard/references/quality-contract.md` + `skills/engineering/dj-regression-guard/scripts/floor_guard.py` | `tests/test_floor_guard.py` 8/8 通过 |
| A2 | `dj-meta/references/trigger-testing.md`；dj-meta 引用行、dj-evolve 验证步接入 | 结构体检：44 技能、触发词 44/44、负向边界仅 2 处 |
| A3 | `dj-check/references/review-dual-axis.md` + SKILL 报告模板 | 四级分档与复核节写入 |
| A4 | `dj-check/references/delivery-risk.md` | L1–L5 证据阶梯 |
| A5 | `dj-implement/references/code-task-contract.md` | DETECT/FETCH/CONFLICT/CITE + UNVERIFIED |
| A6 | `dj-meta/scripts/check_moved_lines.py` + context-diet 步骤 11/12 | `tests/test_check_moved_lines.py` 4/4 通过 |
| A7 | `dj-meta/references/anti-patterns.md` #33 | 一行 |

机械守卫：PASS 141 / WARN 28 / FAIL 0；门禁突变测试 9/9。新增 WARN 来源：本报告 2 条（长度/历史词）、SKILL 超红线清单 14→15（dj-evolve 增一行判据，存量债务）。

## 七、B 批落地记录（2026-11-18）

| # | 落点 | 说明 |
|---|---|---|
| B1 | `dj-channel/references/worker-contract.md` + SKILL 一节 | WORKER.md 开工契约、注入协议、四类停止条件 |
| B2 | `dj-decision` 第 5 步「回来播报」 | 拍板并入 dj-decision，不新建技能 |
| B3 | 同 worker-contract §2 | 委托产出验收：契约一致性 / SHA 记录 / 三态 / 门禁骗过则加固判据 |
| B4 | 同 worker-contract §3 | 模型三护栏：不超顶、slug 拒绝记忆、昂贵模型例外门 |
| B5 | `dj-split/references/task-graph.md` + 拆分原则一行 | frontier、集成分支、merger；票少或冲突面大仍走串行 |
| B6 | `dijiang-finish-work/references/merge-request.md` + 步骤 6.5 | Summary / Evidence / Door + Blast Radius；申请不等于授权 |

机械守卫：PASS 141 / WARN 28 / FAIL 0；三组自测 21/21。

## 八、C 批落地记录（2026-11-18，微增量顺手并入）

| # | 落点 | 内容 |
|---|---|---|
| C1 | dj-grill | 推荐答案附把握度；「你定」是委托不是决策；want-vs-should 探针；可检验的停止判据 |
| C2 | dj-tdd / dj-hunt / dj-teach / dj-wayfinder | seam 一行注记；突变造红的 diff 证明；答案位置轮换；标签独占 + 真实编号 + research 只推不合 |
| C3 | dj-ponytail | ultra 增「动手前质疑需求」 |
| C4 | dj-domain-modeling/context-format.md | `_Avoid_` 别名、通用概念不入表、多上下文 MAP、改名不自动做 |
| C5 | dj-memory + quality-gates.md | 断言带日期/commit、旧条目标 superseded 不改写 |
| C6 | dj-ask / dj-split | 按剩余上下文预算判岔路；切片上限 = 新会话能否稳定做完 |
| C7 | dj-check/references/measurement-review.md | 性能数字七问 |
| C8 | dj-verify/verify-template.md | 先证明检查会失败 + 覆盖记录绑 commit；工具链挂账 |
| C9 | dj-loop/loop-design.md | run 永不等人、刹车三档、先节流后开阀、防双跑五问 |
| C10 | dj-evolve/evolution-criteria.md | 口径冻结、逐 case 归因、存量体检四维（含从未触发） |
| C11 | dj-meta/anti-patterns.md #34 | 建议与命令分层，冲突时命令优先 |

## 九、遗留

1. **超红线瘦身**：15 个 SKILL.md 超 3000 字符（基线 14，本批 dj-evolve 增一行判据跨线），另开一批处理。
2. **触发层实测**：等平台提供独立评测入口后再跑，不在会话内自测。
3. **观察项**：Addy 七个域 checklist、software-factory 完整词汇表、Skill 工厂、Trellis 状态层——维持不收，出现真实失败再评估。