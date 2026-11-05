---
review_before: 2028-11-05
---

# 三槽位职责边界：Skill / Memory / State

> 摘要：判定一条经验/事实/进度该写入 skills/、.dijiang/memory/ 还是任务状态的权威判据，附三种退化模式与症状。dj-memory、dj-meta、dj-evolve、dj-loop 共读。

## 为什么需要边界

DiJiang 的三个运行时槽位物理上已存在，但判据此前未成文。写错槽位的代价不是立即的——是三个月后：技能正文夹带过期任务细节、记忆库堆满一次性流水、任务状态文件膨胀成伪记忆库。

## 槽位定义

| 槽位 | 物理位置 | 回答的问题 | 本质 |
|------|---------|-----------|------|
| **Skill** | `skills/*/SKILL.md` | 「会怎么做」 | 程序性知识：跨任务的方法与纪律 |
| **Memory** | `.dijiang/memory/*.jsonl` | 「知道什么、发生过什么」 | 语义/情节知识：事实、教训、纠正 |
| **State** | `.dijiang/tasks/`、`active_task.txt`、`.temp/regression-baseline-*` | 「做到哪了」 | 工作进度：本任务的可变运行态 |

介于两者之间：`CONTEXT.md`（项目级语义共识）与 `docs/adr/`（不可逆决策的语义层）——写下来就不该频繁变的慢记忆。

## 判别问句（按序问）

1. **它描述「怎么做」且跨任务成立？** → Skill（还需过 SCOPE.md 升格判据）
2. **它是事实/教训/纠正，一个月后仍有效？** → Memory
3. **它只在本任务生命周期内有意义（进度、指针、基线）？** → State
4. **三个都不确定？** → 默认进 Memory（最低信任层），验证后再升格——与 SCOPE 暂存管道一致

## 三种退化模式

| 退化 | 症状 | 修复 |
|------|------|------|
| **State 膨胀成伪 Memory** | 任务目录/session 文件越写越像流水账，塞入「经验」 | 经验抽出到 memory JSONL；State 只留进度与指针 |
| **Memory 冒充 Skill** | findings 被写成操作规则（「应该总是 X」） | 有方法、可复用、过 SCOPE 判据的才升格；否则降回事实陈述 |
| **Skill 吞掉 State** | SKILL.md 正文夹带「本次任务/当前项目」细节 | 技能是通用纪律；任务细节回任务目录或 memory |

## 写入时机（loop 场景延伸）

一次运行产出三类知识，回写边界：**过程**（尝试了什么、走到哪）进 State；**方法**（下次怎么做更快）进 Skill 候选；**事实**（环境真相、教训）进 Memory。

## 关联

- `docs/references/model-invocation.md` — skill 何时被加载
- `SCOPE.md` — 升格判据与暂存管道
- 来源：琅嬛「Skill、Memory、State 的职责边界」「Loop 知识回写边界」
