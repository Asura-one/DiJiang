---
review_before: 2028-11-05
---

# Loop 设计方法论：五件套、四层叠层与 ROI

> 摘要：无人值守 loop 的设计判据——五件套最小闭环、四层叠层、ROI 四条件、三种类型与六坑、L1-L3 成熟度。dj-loop 的知识基座；loop 是运行时设计，DiJiang 只做设计与评审，不运行。

## 定位边界

Loop 不是 prompt 技巧，是**运行时设计**：目标、触发、状态、验证、停止的编排层。DiJiang 是纯 skill 库——本文档供 dj-loop 做 Fitness/Design/Review；产出的设计文档交由用户选择运行时实现。DiJiang 不创建调度器、不自动触发、不自动合并。

## 最小可行 loop：五件套

缺任何一件，就只是「反复手工跑 agent」：

| 件 | 职责 | 关键问题 |
|----|------|---------|
| 1. Trigger | 谁启动 | cron / 事件 / 心跳 / 人 |
| 2. Context+Skill | 带什么知识 | 每轮注入什么、从哪读 |
| 3. Action Surface | 能干什么 | 白名单边界（最小权限） |
| 4. Objective Gate | 怎么算完成 | 强验证优先（evidence-gate.md） |
| 5. State+Stop Policy | 记住什么、何时停 | 状态外置 + 终止/升级条件 |

## 四层叠层（决策权逐层外包，责任与审计保留）

1. **执行循环**（会不会干）：跑一轮任务
2. **验证循环**（干得对不对）：Gate 判定、重试
3. **触发循环**（谁启动）：调度与通知
4. **进化循环**（下次是否更好）：经验回写——过程进 State、方法进 Skill 候选、事实进 Memory（three-asset-boundaries.md）

## Loop ROI：四条件先于技术选型

任务频率 × token 预算 × 验证成本 × review 吞吐——四者不匹配时，loop 从自动化引擎变成积压制造机。开工前量化：

- **频率**：多久真实发生一次？
- **成本**：无人值守消耗远超直觉（实证锚点：6 个 agent 无人值守两天消耗约 10 亿 token）。
- **验证成本**：每轮 gate 的成本（强验证命令 vs 人审）。
- **吞吐**：人能消化的产出速率，超出即积压。
- 核心指标：**cost per accepted change**——生成量廉价，被接受的变更才值钱。

## 三种类型与六坑

类型：观察型（只读报告）/ 生产型（出 PR 人审）/ 自治型（白名单内低风险自动执行）。

| # | 坑 | 规避 |
|---|----|------|
| 1 | 过拟合规则：为单个案例写死规则 | 规则最少化；规则比理由活得久，每条附 why |
| 2 | 出口率错觉：把生成量当产出 | 只计 accepted change |
| 3 | 覆盖率虚高：以为全测了 | 覆盖台账（coverage-ledger.md） |
| 4 | 口径漂移：gate 定义随轮松 | gate 写进 LOOP-SPEC，改动走评审 |
| 5 | 链路暗病：上游静默失败 | 每层显式失败输出（anti-patterns #18） |
| 6 | 执行评估不分离：自跑自评 | maker/checker 分离（evidence-gate.md） |

## 成熟度阶梯

- **L1 只读观察**：产出报告，零写权限——新 loop 一律从这里起步
- **L2 出 PR 人审**：可写，但产物过人审门
- **L3 低风险自治**：白名单内的低风险动作自动执行

跳级上线 = 用生产事故做测试。每级至少稳定运行一个观察周期再升级。

## 状态外置（Loop Run State）

运行轮次状态与 DiJiang 任务态（Task State）是两个东西：

- **Task State**：`.dijiang/tasks/`——一个工作单元的进度
- **Loop Run State**：loop 每轮的 stage / attempts / latest gate result / next action / budget——模板见 dj-loop references

字段纪律：state 是决策依据不是日志仓库——只存「下一轮要读的字段」，叙事性过程进 trace。

## 关联

- `docs/references/evidence-gate.md` — gate 与完成判定
- `docs/references/coverage-ledger.md` — 覆盖记账
- `docs/references/three-asset-boundaries.md` — 知识回写三槽位
- `skills/engineering/dj-loop/` — 设计与评审技能
- 来源：琅嬛 Loop 母线（方法边界与落地路线 / 叠层 Agent Loop / Loop ROI / 六坑五件套 / 长跑 Harness 模式 / 主动型 Agent）
