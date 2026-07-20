# 平台路径、加载和体量速查

平台机制会变。优先核对当前官方文档或本机工具输出。

## 通用原则

- 区分三类文件：人工维护规则、Agent 自动记忆、机器生成历史/索引。写入规则不同。
- 规则真身可能是 AGENTS.md、CLAUDE.md、override、导入或软链；以当前项目声明和加载链为准。
- 发现多个平台目录不等于每个都在使用。只审当前运行平台和用户纳入的安装面。

## DiJiang 特有路径

| 用途 | 路径 |
|---|---|
| 全局指令 | `~/.dijiang/AGENTS.md` |
| 项目指令 | 项目根 `AGENTS.md`、`.dijiang/` 下的配置 |
| 项目 Specs | `.dijiang/spec/` |
| 全局 Skills | `~/.dijiang/skills/<name>/SKILL.md` |
| 项目 Skills | `.pi/skills/<name>/SKILL.md` |
| 任务文档 | `.dijiang/tasks/<name>/prd.md`、`design.md`、`implement.md` |
| 工作区记忆 | `.dijiang/workspace/` |
| Git Gate worktree | `.git/worktrees/` 或 `../<project>-<branch>` |

## Claude Code

| 用途 | 路径 |
|---|---|
| 用户指令 | `~/.claude/CLAUDE.md` |
| 项目指令 | `./CLAUDE.md`、`./.claude/CLAUDE.md`、`CLAUDE.local.md` |
| 路径规则 | `.claude/rules/**/*.md` |
| 自动记忆 | `~/.claude/projects/<project>/memory/` |
| Skills | `~/.claude/skills/<name>/SKILL.md` |

CLAUDE.md 全量加载，但建议目标少于约 200 行。自动记忆 MEMORY.md 在会话启动时只加载前 200 行或 25KB（先到者）。

## OpenAI Codex

| 用途 | 路径 |
|---|---|
| 全局指令 | `~/.codex/AGENTS.override.md`，不存在则读 `AGENTS.md` |
| 项目指令 | 项目根到当前目录逐级找 `AGENTS.override.md`、`AGENTS.md` |
| 全局 Skills | `~/.codex/skills/<name>/SKILL.md` |
| 项目 Skills | `.codex/skills/<name>/` |

指令链合并后默认最多 32KiB（`project_doc_max_bytes`）。越靠近当前目录的指令越晚加载。

机器生成记忆（如 Chronicle、rollout summary）默认只读，通过官方 `/memories` 或 correction input 更新。

## 其他平台

在项目根和上级目录找 `AGENTS.md`（跨平台标准）、`CLAUDE.md`、平台专属形态（`.cursor/rules/`、`.cursorrules`）。用三分法归类：人工规则 / 自动记忆 / 机器生成。归类不明时按机器生成处理。

## 共存检查

1. 列出实际存在的平台目录和 skill realpath
2. 核对同名 skill 是否软链到同一真身、复制安装或覆盖
3. 只改权威真身；复制安装需要明确同步机制
4. 验证加载而不是只验证文件存在：使用平台诊断入口

## 官方复核入口

- Agent Skills: <https://agentskills.io/specification>
- Claude Code memory: <https://code.claude.com/docs/en/memory>
- Codex AGENTS.md: <https://developers.openai.com/codex/guides/agents-md/>
