---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '233cb8e0-6c6b-4637-a18b-570e54332dd1'
  PropagateID: '233cb8e0-6c6b-4637-a18b-570e54332dd1'
  ReservedCode1: '61f57e34-ea23-436b-a886-8025e345766d'
  ReservedCode2: '61f57e34-ea23-436b-a886-8025e345766d'
---

# ADR 0006: 引入 Regression Guard 与 Fullstack Testing 到纯 Skill 架构

**状态**: Proposed  
**日期**: 2026-09-20  
**决策者**: 老大

## 上下文

DiJiang 作为纯 skill 架构（mattpocock/skills 风格），拥有完整的改码工作流：`dj-implement` 实现、`dj-tdd` 红绿循环、`dj-hunt` bug 排查、`dj-check` 交付质量闸门。然而"改码回归保护"这一任务级纪律在项目中是**碎片化**的：

1. **`dj-check`** 有"回归风险"维度，但它是**交付后**的质量门禁，只管改完检查，不管改前基线
2. **`dj-hunt`** 铁律含"每次修复后跑 regression"，但只针对 bug 修复场景，且没有改前基线机制
3. **`dj-implement`** 有 "Code Task TDD Contract" 与 regression scope，但没有独立的回归协议承载

缺少一个**任务级的改码三明治协议**：**改前基线 → 修改 → 专项验证 → 改后回归**。这导致 agent 改码时难以区分"本来就坏的"和"我改坏的"，回归归因失效。

外部 `regression-guard` 技能已经把这套协议打磨成熟（S0–S6、零基建退路、铁律），且有配套的全量回归引擎 `fullstack-testing`（API 深度验证、契约校验、UI 交互、UX 评估、覆盖度核对）。本次决策将这两个外部技能**融合进 DiJiang**（落地为 `dj-regression-guard` 与 `dj-fullstack-testing`）。

## 决策

将 `regression-guard` 与 `fullstack-testing` 融合进 DiJiang 纯 skill 架构：

1. **新增 `dj-regression-guard`**（engineering bucket，model-invoked）：承载完整的"改码三明治回归协议"（S0–S6 + 零基建退路），作为改码类任务的前置纪律，可被 `dj-implement`/`dj-hunt`/`dj-tdd`/`dj-check` 引用
2. **新增 `dj-fullstack-testing`**（engineering bucket）：作为全量回归引擎，供 `dj-regression-guard` 引用其 diff 映射表、冒烟最小集、`docs/project-understanding.md` 文档格式；"引用不复制"——机制以 `dj-fullstack-testing` 文本为准
3. **改造现有 skill 收敛碎片**：将 `dj-check` 的"回归风险"维度、`dj-hunt` 的"每次修复后跑 regression"、`dj-implement` 的 "regression scope" 统一改为**引用 `dj-regression-guard`**，消除重复的回归纪律描述
4. **更新 CONTEXT.md 术语**：补充 `dj-regression-guard`、`dj-fullstack-testing`、`快速层`、`回归基线` 等术语
5. **记录本 ADR**：为后续维护者说明"为什么 DiJiang 里会有回归三明治协议"以及"如何定位各回归机制"

## 方案权衡

最终选择**新增独立 skill + 改造现有收敛**（而非只新增、也不溶解进现有 skill）。

- **新增独立 `dj-regression-guard`**：回归协议是任务级横切纪律，独立成 skill 才能被所有改码入口统一引用；溶解进现有 skill 会导致协议碎成三份、各自演进失真
- **引入 `dj-fullstack-testing` 作为引用对象**：全量回归引擎与任务级轻量协议分离——回归守卫引用 `dj-fullstack-testing` 的三处机制，不内嵌副本；闭环成立且无重复代码

## 放弃的方案

- **只溶解进现有 skill（dj-check/dj-hunt/dj-implement）**：协议会碎成碎片，无法统一演进；回归纪律作为独立横切关注点不应寄生在单个 skill 内
- **引入但不改造现有 skill**：散落在 dj-check/dj-hunt/dj-implement 里的旧回归描述继续存在，造成"回归纪律有两套表述"的冗余
- **外部 `fullstack-testing` 完整搬运**：外部版 647 行重引擎完整照搬会过度吸收（DiJiang 是纯 skill 仓库，无运行时/测试基建）；改按 DiJiang 风格精简保留完整工作流与三处被引用机制，命名统一为 `dj-` 前缀

## 影响

- **正面**：改码回归纪律从碎片化为单一真相源；agent 能明确区分"我改坏的"与"本来就坏的"；回归成本可控（快速层）
- **正面**：Dj-check 专注于交付收口，dj-regression-guard 专注于改码过程，职责边界清晰
- **注意**：`dj-fullstack-testing` 面向有测试基建的真实项目，DiJiang 本身是纯 skill 仓库无运行时——引入它作为可被各项目引用的引擎，而非 DiJiang 自身跑全量测试
- **新增 skill 命名与 DiJiang 惯例一致**（`dj-` 前缀 + 语义名）

## 关联

- 关联技能：`dj-regression-guard`、`dj-fullstack-testing`、`dj-check`、`dj-hunt`、`dj-implement`
- 关联 ADR：ADR-0005（纯 Skill 架构）
- 关联规范：`docs/references/code-task-contract.md`、`docs/references/model-invocation.md`