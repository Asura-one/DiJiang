---
name: dj-channel
description: "多 agent 协作通道：生成子代理执行任务、跨 agent 协作、进度检查、通道日志调试。 Use when you need to spawn sub-agents, coordinate multiple agents, or debug channel logs. 触发词：子代理、多 agent、协作通道、生成 agent、spawn、channel、派任务。"
disable-model-invocation: true
---

# dj-channel

多 agent 协作通道：让另一个 AI agent 独立执行任务、并行验证、跨 agent 协作。**本 skill 不自带 CLI**——DiJiang 是纯 skill 库（ADR-005），通道能力来自当前 harness 的原生多 agent 设施，本 skill 定义的是组织纪律与角色边界。

典型信号："和另一个 agent 讨论"、"派一个 implement/check worker"、"让 agent review"、
"channel 没输出了"、"看看这个 thread"。

## 运行形态自适配（按当前 harness 能力选一档）

| harness 能力 | 形态 | 示例 |
|---|---|---|
| 有多 agent CLI / channel 工具 | 直接用平台命令（list/spawn/send/status/stop 语义） | 以平台 `--help` 实测为准，不假设语法 |
| 有子代理 spawn 能力（Task/subagent） | 每通道 = 一个子代理调用，task 描述即通道契约；进度靠子代理返回值与 trace | TeleAgent：spawn subagent、结果报告回主 session |
| 无多 agent 设施 | 降级为单 agent 串行：本 skill 只作组织约定参考（下表 + references） | 角色边界/trace 协议照样适用 |

发现命令不存在（command not found）时立即换档，不重试不硬跑（anti-patterns #18/#23）。

## 通道生命周期（与形态无关的通用语义）

| 动作 | 语义 |
|---|---|
| spawn | 声明角色（Scout/Builder/Verifier）+ 明确 task 描述 + 工具边界 |
| send | 跨通道传递中间结果（一句话增量，不全文转发） |
| list / status | 监控活跃通道，不假设已完成 |
| stop | 不再需要的通道清理 |

## 何时使用

| 场景 | 做法 |
|------|------|
| 需要另一个 agent 独立 brainstorm | spawn brainstorm 角色，task 里写清探索域 |
| 并行验证 | spawn Verifier（只读权限）与 Builder 同时运行 |
| 长时间运行的任务 | spawn 后台 worker，定期查进度 |
| 跨 agent 消息传递 | send 中间结果，附角色边界 |
| 调试卡住的 worker | 先列活跃通道定位，再查该通道 trace/状态 |

## 组织模式与角色纪律

三模式：主脑+专才（任务可拆分）/ 独立共享（多方案对比）/ **混合**（默认——主脑持目标、专才独立执行）。spawn 时在 task 里声明角色工具边界（coder 不碰浏览器、reviewer 只读、Builder 不得自审自己的产出）；spawn 模板要求子代理把关键过程观察（报错/意外响应/顺带发现）落盘 trace，主 agent 跨通道按关键词检索命中步骤——未升格为正式产物的观察不丢失，边界不变。Scout/Builder/Verifier 权限矩阵与 trace 格式 → `references/role-separation.md`

## 核心规则

- spawn 时指定明确的 task 描述与角色边界。描述越清晰，输出越精确。
- 监控活跃通道状态，不要假设 agent 已完成。
- 长时间运行的任务定期检查进度。
- 不再需要的通道清理。

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| `spawn` 报 agent 名不存在 | `channel list` 确认可用 agent 类型 | 报告可用类型，请用户确认 |
| 通道长时间无输出 | `channel status <id>` 查状态 | `channel stop <id>` 后重 spawn，收紧 task 描述 |
| `execute` 报错 | 读通道日志定位 | 换同步方式单跑，不阻塞主流程 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 假设 agent 已完成就继续 | 先查活跃通道状态确认 |
| 同步阻塞等长任务跑完 | spawn 后台 + 定期检查进度 |
| 结束前不清理通道 | 清掉不再需要的通道 |
| 用 channel 做单次静态审查 | 一个 prompt 直接做 |
| 命令不存在还硬跑/重试 | 换运行形态档位（见上表） |

## 参考文件

- `references/role-separation.md` — 角色权限矩阵与 trace 协议

## 不属于此 skill 的内容

- 单次静态审查，一个 prompt 就够了
- 长期记忆检索 → dj-memory
