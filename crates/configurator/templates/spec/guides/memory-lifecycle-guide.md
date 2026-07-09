# 记忆生命周期指南

## 目的

DiJiang 的记忆不是笔记本。它是一个受控的学习循环，帮助未来的 agent 理解任务、复用已验证的策略、避免重复错误，同时不让过时或未经证实的声明污染后续工作。

## 记忆层次

| 层次 | 存储内容 | 何时写入 |
|-------|----------------|---------------|
| **全局记忆** (`~/.dijiang/mem/`) | 跨项目可复用的经验教训、策略、用户偏好和修正。必须通过质量门禁。 | 跨项目层次：发现可在任何项目中复用的东西。由 `dijiang mem backup` 或 `dijiang mem evolve` 提升。 |
| **项目记忆** (`.dijiang/mem/`) | 项目特定的 findings、lessons、corrections、tactics 和 patterns。 | 每次任务后，如果产出物通过了质量门禁。 |
| **任务 artifacts** (`.dijiang/tasks/<task>/*.md`) | 任务产出物：prd、design、implement、journal。无自动提升。 | 每个任务完成时通过 `dijiang finish-work`。 |
| **会话转储** (`.dijiang/workspace/`) | 原始会话日志。AI 生成，人类无需阅读。无提升。 | 每次 `dijiang finish-work` 自动转储。 |

## 质量门禁

每条持久化记忆必须通过以下检查才能写入：

| 标准 | 含义 |
|----------|-------|
| **Source** | 来自可追踪的真实事件、用户纠正或已验证的模式吗？ |
| **Scope** | 适用级别明确吗：全局、项目、特定 package、特定层？ |
| **Confidence** | 多次观察到（策略/模式）或一次明确纠正（教训）？ |
| **Freshness** | 仍然相关吗？旧配置、旧 CLI 行为在重构后需要重新验证。 |
| **Conflict** | 与现有记忆冲突？解决或标记为候选。 |
| **Actionability** | 能改变未来行为吗？（"有用性"阈值） |

## 命令映射

| 时机 | 操作 | 命令 |
|------|------|-------|
| 发现被纠正 | 写入教训 + 纠正 | `dijiang mem learn` + `dijiang mem correction` |
| 发现可复用策略 | 写入 tactics | `dijiang mem tactic` |
| 发现重复模式 | 写入 patterns | `dijiang mem pattern` |
| 会话结束 | 归档 + 提升 | `dijiang mem archive`，然后 `dijiang mem evolve` |
| 跨项目 sync | 备份到全局 | `dijiang mem backup` |

## 常见反模式

- 把每件小事都记下来 → 噪声。问：下次的 agent 会真的在意这个吗？
- 写入模糊的 lessons，没有具体的 source → "用户说更喜欢 X" 比 "注意偏好" 更好。
- 在重构后未重新验证记忆 → 关于旧代码路径的建议可能有害。
- 跳过 conflict 检查 → 两条矛盾的记忆比没有记忆更糟糕。
