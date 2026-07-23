# 版本管理流程

## 权威面（2026-07-23 决策）

**以 Cargo workspace 的 `0.x` 为项目版本权威：**

- 文件：根目录 `Cargo.toml` → `[workspace.package].version`
- 当前值：`0.13.5`
- 所有 crate 包版本：`version.workspace = true`（含 cli / task / mem / configurator / mcp-server）
- `dijiang --version` / MCP `serverInfo.version`：`CARGO_PKG_VERSION` → 与 workspace 同号
- `finish-work --version-impact` 递增目标：workspace 权威面
- 根目录 `VERSION`：与权威面同号的从属投影（供脚本/展示）

## 各版本面（对齐后）

| 面 | 角色 |
|----|------|
| **workspace Cargo** `[workspace.package].version` | **权威** |
| 各 crate `version.workspace = true` | 继承权威 |
| 根 `VERSION` | 从属投影，应与权威同号 |
| CLI / MCP 运行态 | `CARGO_PKG_VERSION`，与权威同号 |

## VERSION 文件

```
0.13.5
```

不含前缀 `v`。与 workspace 权威保持一致。

## 发版 / bump 步骤

1. **只 bump 权威面**：`Cargo.toml` `[workspace.package].version`（或 `finish-work --version-impact major|minor|patch`）
2. 同步根 `VERSION` 到同号（若脚本未自动做）
3. 抽查：`cargo build -p dijiang` 后 `dijiang --version` 为新号
4. 提交：`chore: bump to <workspace-version>`

## 版本策略（作用于 workspace 权威）

| 影响范围 | 变更 | 示例 |
|----------|------|------|
| 不兼容公开行为 / API | major | 0.13.5 → 1.0.0 |
| 向后兼容新功能 | minor | 0.13.5 → 0.14.0 |
| 向后兼容修复 | patch | 0.13.5 → 0.13.6 |
| 仅文档/测试/workflow | none | 不改权威版本 |

## 检查脚本

`.dijiang/scripts/check-version.sh` 若存在，应以 workspace 权威与 `VERSION` 同号为准则；未改前人工核对 `Cargo.toml` 与 `VERSION`。

## 决策记录

- **2026-07-23**：用户确认「以 workspace 0.x 为准」；随后将 cli/task/mcp 改为 `version.workspace = true`，`VERSION` 对齐为 `0.13.5`。
