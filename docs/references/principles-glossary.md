# principles-glossary: pstack 原则 → DiJiang 技能映射

来源：pstack（poteto agent 系统）24 条原则的融合索引（2026-11，dj-absorb）。本表是**映射不是复制**——原则的正文留在源项目（`referens/pstack/skills/principle-*/`），这里只回答「这条原则在 DiJiang 里找谁」。skill 正文需要引用某原则时，指本表行，不展开内容。

## 一、已有对应（直接用 dj 技能）

| pstack 原则 | DiJiang 对应 |
|---|---|
| Laziness Protocol | `dj-ponytail`（最懒真正有效，YAGNI） |
| Foundational Thinking | `dj-codebase-design`（data shape 先行） |
| Subtract Before You Add | `dj-ponytail`（先删后建） |
| Minimize Reader Load | `dj-codebase-design`（深度/局部性词汇） |
| Model the Domain | `dj-domain-modeling` |
| Experience First | `dj-design` |
| Exhaust the Design Space | `dj-channel`（竞争择优：`references/arena.md`） |
| Build the Lever | `dj-script`（造工具不手工，部分覆盖） |
| Sequence Work into Verifiable Units | `dj-split` + `dj-tdd`（垂直切片） |
| Prove It Works | `dj-check`（evidence-gate 证据三态） |
| Fix Root Causes | `dj-hunt`（根治不打补丁） |
| Test Behavior, Not Implementation | `dj-tdd` |
| Guard the Context Window | `dj-channel`（批量路由子代理、主线程留摘要） |

## 二、本轮融合已落地（2026-11 P0/P1）

| pstack 原则 | DiJiang 落点 |
|---|---|
| Explain the Number | `dj-fullstack-testing/references/benchmark-checklist.md`（七问） |
| Encode Lessons in Structure | `dj-correct`（逐级强制：架构→类型→lint→测试→文档）+ `dj-evolve` 候选分类优先机制层 |
| show-me-your-work（决策轨迹） | `dj-decision` |
| in-your-own-words（回述目标） | `dj-ask` 单请求路由规则（回述即开工） |

## 三、真缺口（无对应物，待 dj-evolve 升格判断——不主动建技能）

| pstack 原则 | 一句话含义 | 潜在落点 |
|---|---|---|
| Attack the Premise | 共享同一前提的两次修复都失败 → 写下前提直接测它，别再写第三个假设它的修复 | `dj-hunt` 疑难杂症段 |
| Redesign from First Principles | 新需求并进现有设计时，按它第一天就该存在重新设计一遍 | `dj-codebase-design` |
| Outcome-Oriented Execution | 迁移/重写有明确阶段边界，收敛到目标架构，不留兼容垫片残态 | `dj-implement` |
| Type System Discipline | 让非法状态不可表示，边界处 parse 外部数据 | 工程常识层，暂无 skill 落点 |
| Make Operations Idempotent | 命令/生命周期在崩溃重试下收敛到同一终态 | 工程常识层 |
| Migrate Callers Then Delete Legacy APIs | 新内部 API 落地时同批迁移全部调用方并删旧路径 | `dj-implement` |
| Separate Before Serializing Shared State | 并发写同一文件/分支/键 → 先消除共享再考虑加锁 | `dj-channel`/工程常识层 |

## 四、明确不采纳

| pstack 原则 | 不采纳理由 |
|---|---|
| Never Block on the Human | 与确认红线冲突：破坏性操作、对外发送、人审决策必须停下等确认；已用「回述即开工 + 确认红线分层」替代（`dj-ask` 路由规则） |

## 维护

pstack 原则集更新（upstream 合并）时对照本表重审三、四节；真缺口条目被 dj-evolve 升格落地后移入一、二节。
