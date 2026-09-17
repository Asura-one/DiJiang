---
name: dj-channel
description: 多 agent 协作通道。通过 Call the Skill tool with "dj-channel" 生成 AI agent 执行任务、跨 agent 协作、 进度检查、通道日志调试。
disable-model-invocation: true
---

# dj-channel

Call the Skill tool with "dj-channel" 是 DiJiang 的多 agent 协作运行时。当需要让另一个 AI agent 独立执行任务、
并行验证、或跨 agent 协作时使用。

典型信号："和另一个 agent 讨论"、"派一个 implement/check worker"、"让 agent review"、
"channel 没输出了"、"看看这个 thread"。

## 基本命令

```bash
dj-channel skill --help

# 列出所有活跃通道
dj-channel skill list

# 生成一个 agent 执行任务
dj-channel skill spawn <agent> --task "..."

# 向通道发送消息
dj-channel skill send <id> <message>

# 查看通道状态
dj-channel skill status <id>

# 在通道中执行 agent
dj-channel skill execute <id>

# 并行执行所有活跃通道
dj-channel skill execute-all

# 停止通道
dj-channel skill stop <id>
```

## 何时使用

| 场景 | 做法 |
|------|------|
| 需要另一个 agent 独立 brainstorm | `channel spawn brainstorm-agent --task "..."` 然后用 list/status 跟踪 |
| 并行验证 | spawn 一个 check agent，与 implement 同时运行 |
| 长时间运行的任务 | spawn worker 作为后台进程，定期检查 status |
| 跨 agent 消息传递 | `channel send` 在 agent 之间传递中间结果 |
| 调试卡住的 worker | 先 `channel list` 找到 id，再用 `channel status <id>` 检查 |

## 核心规则

- 生成 agent 时指定明确的 task 描述。描述越清晰，agent 输出越精确。
- 使用 `channel list` 监控活跃通道，不要假设 agent 已完成。
- 长时间运行的任务使用 `channel status <id>` 定期检查进度。
- 不再需要的通道用 `channel stop <id>` 清理。

## 参考文件

- Call the Skill tool with "dj-channel" — 所有子命令的最新参考

## 不属于此 skill 的内容

- 单次静态审查，一个 prompt 就够了
- 长期记忆检索。使用 Call the Skill tool with "dj-memory" to recall（dj-session-insight skill）
