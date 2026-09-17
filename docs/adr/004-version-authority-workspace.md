# ADR 004：项目版本权威面

- **状态**：Accepted
- **日期**：2026-07-23
- **关联任务**：task-20260723060718（承接 dj-gov 遗留）

## 决策

以根目录 `Cargo.toml` 的 `[workspace.package].version`（`0.x` 线，当前 `0.13.5`）为**项目版本权威**。

本决策回合**不修改任何版本数字**。

## 后果

- `finish-work --version-impact` 与发版叙述以 workspace 为准
- `VERSION` 文件视为从属/遗留，不再写「唯一权威」
- CLI/`dijiang --version`（crate `0.6.3`）与 mcp（`0.1.0`）允许暂时偏离；对齐代码另开任务
- `check-version.sh` 未改前不能当作 workspace 一致性证明

## 备选（放弃）

- 以 `VERSION=3.0.0` 为唯一权威（与 finish-work / Cargo 现实冲突）
- 以 CLI `0.6.3` 为项目权威（与 workspace bump 目标冲突）
- 本轮强行统一改号（用户明确「本轮不改号」）

## 后续落地

- task `cli-task-workspace-bump`：cli/task/mcp 改为 `version.workspace = true`；`VERSION` 对齐 `0.13.5`；MCP serverInfo 用 `CARGO_PKG_VERSION`。
