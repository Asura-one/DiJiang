# DiJiang Changelog

## 0.1.2 (2026-07-03)

### Added
- 版本号追踪：`dijiang update` 现在会显示版本变化并展示变更日志
- 配置文件新增 `dijiang_version` 字段，记录项目上次使用的 DiJiang 版本
- 更新命令现在会展示版本对比和变更摘要

### Changed
- 统一版本号来源：CLI 和 configurator 均使用 `CARGO_PKG_VERSION`，不再硬编码 `0.1.0`
- 明确 Git worktree 完成后的主仓库落地规则：未执行 `--integrate` 时必须在主 checkout 合并任务分支；远端不可达时 push 不阻塞本地 merge 与已合并 worktree 清理
- 统一 CLI、hook、prompt、workflow 和 session skill 的面向用户文案，避免中英文标签混用

## 0.1.1 (2026-06-30)

### Added
- 初始化时写入更新的 `codex/hooks.json` (EventType: Notification)
- 为 Codex 和 Cursor 新增 `inject-workflow-state` hook
- 新增 `.dijiang/spec/` 编码规范引导
- 新增 `.dijiang/spec/guides/` 思考引导
- dj-hunt, dj-implement, dj-script 等实现类 skill 现在提供 TDD 约束

### Changed
- 将默认 task priority 从 Unset 改为 P2
- 将 `dijiang dispatch` 的工作区默认值从空字符串改为 `None`（devType）
- 清理所有 codex 模板中的 tab 字符

## 0.1.0 (2026-06-28)

### Added
- DiJiang 初始版本
- `dijiang init` — 项目初始化
- `dijiang update` — 更新受管文件
- `dijiang start / dispatch / finish-work` — 任务生命周期
- `dijiang task` — 任务管理
- `dijiang mem` — 记忆系统
- `dijiang channel` — 子代理通道
- `dijiang template` — 模板管理
- `dijiang skills` — 技能清单
- `dj-*` skills: dispatch, grill, output, implement, tdd, hunt, check, review, audit, debt, health, pattern, design, prototype, script, ponytail, karpathy, write, handoff
- `dijiang-*` skills: start, continue, finish-work
