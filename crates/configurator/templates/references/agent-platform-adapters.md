# Agent 平台适配器

DiJiang 的技能定义（SKILL.md）以 Pi agent 平台为第一目标，但设计上保持模型无关性。

## 当前平台

| 平台 | 状态 | 备注 |
|------|------|------|
| **Pi** | ✅ 主平台 | SKILL.md 原生格式 |
| **Claude Code** | 🟡 可移植 | 需要 translate 为 CLAUDE.md |
| **Codex CLI** | 🟡 可移植 | 需要 translate 为规则文件 |
| **Cline** | 🟡 可移植 | 遵循规则文件格式 |

## 指令共享模式

当需要支持多平台时：

1. **SKILL.md 保持单一真相源** — 所有内容先写在 SKILL.md
2. **平台适配器** — 按需将 SKILL.md 内容 translate 为目标平台格式
3. **过滤** — 按平台裁剪 SKILL.md 内容（如 Pi 特有的 tool 定义对非 Pi 平台不可见）

## 不要做的事

- 不在 SKILL.md 中写平台 if-else 条件
- 不为多平台创建多个版本的相同内容
- 不在 SKILL.md 中引用平台特定工具（如 `delegate_task` 对非 Pi 平台不可见）

## 参考

Ponytail 项目的 `hooks/ponytail-instructions.js` 提供了按平台+强度过滤 SKILL.md 内容的参考实现。
