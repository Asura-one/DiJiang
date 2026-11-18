# pstack 融合评估与落地存档

来源：2026-11-09 dj-absorb 任务（外部目标 `referens/pstack`，Cursor/poteto agent 技能系统：~34 技能 + 23 playbook + 24 原则 + benny 自动化）。**状态：P0+P1 已完成**（v2.5.0，merge `767ba98`）。续接会话从此文件取上下文，无需重跑摸底。

## 结构同构判断

pstack 与 DiJiang 结构同构：poteto-mode hub ↔ dj-ask 单一入口路由。融合评估结论：值得融合的是**机制/流程**而非照搬技能文件。

## 终局规则：融合形式前置判定

定实现策略前先回答『这个机制将来怎么被触发？』——单一入口体系两条合法出路：**内嵌**现有技能流程；或**新建 + 同批挂进入口路由全部触点**（DiJiang 三处）。纠偏链：首轮 4 新技能被「平时只用 dj-ask」推翻 → 全内嵌方案被「新建也可以嵌入 dj-ask 流程」二次纠正 → 终局两条出路。『必须内嵌不可新建』是过度矫正。

## 落地清单（全部完成）

**P0（新建 + 同批挂 dj-ask 三处路由）**：

| 落点 | 触发词 | 衔接 |
|---|---|---|
| `dj-decision` | 无人值守/回头检查/你看着办/留轨迹 | dj-loop 跑的任务人走后留轨迹；审计口径沿 dj-check evidence-gate |
| `dj-correct` | 别老犯这错/又犯同样的错/防复发 | dj-hunt 修单次、correct 灭一类、dj-evolve 文档层兜底、dj-memory corrections 输入 |
| `dj-verify` | 做个验证技能/功能地图/验证可复用 | dj-fullstack-testing 是引擎它造资产；feature map 维护走 dj-gov |
| 回述规则 → 并入 dj-ask 单请求路由 | 自动生效 | 与 grill 分层：清晰任务回述即开工，不清才升级拷问 |

**P1（内嵌补强）**：benchmark 七问 → `dj-fullstack-testing/references/benchmark-checklist.md`；arena 择优嫁接 → `dj-channel/references/arena.md`；reflect 三透镜 + 机制层优先 → `dj-evolve`；24 原则四档映射 → `docs/references/principles-glossary.md`（真缺口 7 条待 dj-evolve 二次复现判、不采纳 1 条留痕）。

**明确不采纳**：poteto-mode 整体路由/playbooks（与 dj-ask 重复）；Never Block on the Human（与确认红线冲突）。

## 摸底方法备忘（续接无需重跑）

四只读研究子代理并行分别摸底：决策轨迹类（判定真缺口——dj-handoff 是结尾快照、dj-memory 是经验沉淀，均非运行中轨迹）、纠正防复发类（边界缺口——dj-evolve/dj-memory 有「记录→规则」出口但缺强制层）、验证技能生成类（真缺口——测试引擎不产可复用资产）、回述目标类（轻量缺口——并入 dj-ask 而非新建）。

## Phase 3 实证教训

- **字节预算写前做**：中文 UTF-8 3B/字，字数直觉低估；三新技能初版 3535/3258/4247B 各精简 2-3 轮才进线；细节外溢 references/ 是仓库既定模式
- **新技能落点**：`skills/engineering/<dj-xxx>/SKILL.md` + frontmatter（name/description 含触发词/disable-model-invocation: true）
- **session 结束时仓库改动未提交需先核对**：finish-work 前确认 git 状态，未提交的融合产物先走收口流程
- **平台进化 × 单向同步冲突（2026-11-09 实证）**：skill-evolution 直接改 live 不改 repo，repo→live 的 rsync `--delete` 会把它抹掉（两轮 dj-absorb 进化被 finish-work 第 11 步覆盖，靠 `.backup` + 会话记录回灌）。护栏已加入 live-sync-checklist「第 0 步：同步前对账」


## 融合形式前置判定细则（正文迁入）

**融合形式前置判定（新增 vs 内嵌，2026-11 pstack 批教训）**：定实现策略前先回答『这个机制将来怎么被触发？』——融合的是机制/流程，不是照搬技能文件。单一入口体系两条合法出路：内嵌进入口已路由的现有技能流程（用户入口零改动），或新建独立技能 + **同批挂 dj-ask 三处路由**（`dj-ask/SKILL.md` 入口匝道、`dj-ask/references/routing-table.md` 触发行、`dj-ask/references/skill-catalog.md` 详注，缺一即死技能）；『必须内嵌不可新建』是过度矫正，形式拿不准先问用户使用入口。

## Gotchas 长条目（正文迁入）

| 单一入口体系下新建技能不挂路由 | 新建必须同批挂进入口路由三处；全内嵌仅当机制天然属现有流程 |

| 融合产出 SKILL.md 写完才查体积红线 | 动笔前预算（硬上限 3000、目标 800-2500；中文 3B/字，字数直觉低估） |
