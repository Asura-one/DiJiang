---
name: dj-loop
description: "无人值守 loop 的设计与评审：判断任务值不值得 loop 化、产出 LOOP-SPEC 设计文档、跑上线检查清单出评审结论，只设计不运行。 Use when 用户想搭建/设计/评审自动化循环、无人值守任务、定时任务工作流，或现有 loop 出问题要复盘设计。 触发词：loop、无人值守、自动化循环、定时任务、autonomous loop、loop 设计、loop 评审、loop 复盘。"
---

# Loop: 无人值守 loop 设计与评审

## 职责

回答三个问题并产出文档：**这个任务值得 loop 化吗**（Fitness）、**loop 该怎么设计**（Design）、**这个设计可以托管吗**（Review）。产物是 LOOP-SPEC 与评审结论；实现调度器/cron/CI 归 dj-implement 与 dj-script，运行归用户选择的运行时——DiJiang 无运行时（ADR-005）。

## 工作流

### 1. Fitness——先算账再设计

对照 `docs/references/loop-design.md` 的 ROI 四条件（频率 × token 预算 × 验证成本 × review 吞吐）逐项量化。任一条件不成立 → 结论「不值得 loop 化」，给替代路径（人跑 + dj-memory 记经验 / dj-script 单次自动化）。量化不了就先设计一个 L1 只读观察轮收集数据。

### 2. Design——五件套 + 模板

复制 `references/loop-spec.md` 逐节填写；gate 引用 `docs/references/evidence-gate.md`（强验证优先，不自创标准）；运行态用 `references/loop-state.md`。成熟度一律 L1 起步，写明升 L2/L3 的观察条件。

### 3. Review——上线门

对照 `docs/references/loop-runtime-checklist.md` 九组逐项作答；权限/升级/接管面过 `references/agent-risk-review.md`。结论三选一：**可托管 / 有条件通过（列条件）/ 不可托管（列缺口）**——任何一组只能答「应该没问题」即不可托管。

## 边界

- 不创建 cron/调度器/常驻进程，不自动触发/执行/合并——产物是文档与结论（why：纯 skill 库红线，越过撞 .out-of-scope 运行时否决）
- gate 判据不自创——引用 evidence-gate.md 单一权威，防止与 dj-check 双标打架
- session 内工作流的自动化不走本技能——那是 dj-implement + dj-regression-guard 的三明治

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 用户说不清触发频率与量级 | 按 ROI 四条件反推提问 | 先按 L1 只读观察设计，跑一个周期再定 |
| 设计含自动合并/删除/对外发布 | 收回 L1/L2，危险动作升级给人 | 拒绝设计并说明残余风险 |
| 与既有 CI/自动化重叠 | 先盘点既有设施，设计只做增量 | 结论「不需要新 loop」 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 顺手把 cron 建了 | 交付设计文档，实现归 dj-implement/dj-script |
| gate 写「LLM 觉得没问题」 | 强验证优先（evidence-gate.md） |
| 从 L3 自治起步 | 一律 L1 起步，观察周期后升级 |
