---
name: dj-verify
description: "为项目生成可复用的本地验证技能 verify-<app>：interview repo → 生成五段技能（Launch/Doctor/Drive/Evidence/Cleanup）+ 用户视角 feature map → 端到端跑通后才交付。 Use when 要给一个 repo 建立可持续复用的验证能力、生成功能地图，或厌倦每次回归都重新摸一遍。 触发词：做个验证技能、验证资产、功能地图、feature map、验证可复用、verify skill。"
disable-model-invocation: true
---

# Verify: 生成项目本地验证技能

## 职责

把对 repo 的验证能力**物化成可复用资产**（verify-<app> 技能 + feature map），后续回归/巡检直接复用。与 `dj-fullstack-testing` 互补：它是测试引擎，本 skill 造资产。写给从未见过这个 app 的冷启动 agent。

## 工作流

### 1. Interview repo，不问用户

从代码回答五问（Surface/Run/Drive/Observe/Isolate），只问代码观察不到的；问法与判据照 `references/verify-template.md` § Interview。repo 起不来先修或精确报告——对坏基座写技能会教错步骤。

### 2. 生成 verify-<app>

写 `<repo>/.verify/<app>/SKILL.md`，五段（Launch/Doctor/Drive/Evidence/Cleanup）实填无占位符，每段验收判据照 template § 五段。红线：Evidence 走真实用户路径而非内部 setter；Cleanup 按句柄杀不按进程名，证据永不清理。

### 3. 种 feature map

features/：README 索引 + 每功能一文件（先 top 3–5），四段式格式照 template § feature map。种子数据缺前置 → 原因进 Gotchas，**绝不弱化旅程让它通过**。

### 4. 先跑通再交付

端到端跑自己写的指令一次：launch → doctor → drive 一个已映射功能 → 存证据 → cleanup → 确认证据还在（被吃 = 失败）。失败就修，每轮失败后都跑 cleanup 防残留。**没执行过的是草稿不是交付**。替身环境（容器替真机/本地替远端）验证的，列出替身多做的每步——那些未验证。

## 维护

feature map 是 repo 活文档：功能变更后走 `dj-gov` 对账，不新建维护机制。

## 失败模式

| 触发 | 一线修复 | 兜底 |
|---|---|---|
| repo 起不来 | 先修或精确报告 | 不对坏基座生成 |
| 无法程序化驱动 | 找替代面（调试端口/测试端点） | 报告边界不硬驱动 |
| cleanup 吃证据 | 证据放技能点名的固定位置 | 判失败重修 |

## 反例

| ❌ | ✅ |
|---|---|
| 写完直接交付 | 跑通一次才算交付 |
| Drive 用示例选择器 | 抄本 repo 真实句柄 |
| 弱化旅程让 map 全绿 | 失败原因进 Gotchas |
