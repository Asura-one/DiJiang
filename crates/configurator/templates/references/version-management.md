# 版本管理流程

## 权威面（2026-07-23 决策）

**以 Cargo workspace 的 `0.x` 为项目版本权威：**

- 文件：根目录 `Cargo.toml` → `[workspace.package].version`
- 当前值：`0.13.5`
- `finish-work --version-impact` 递增目标就是这一面

本轮（决策收尾）**不改任何版本数字**，只固定策略。

## 各版本面关系

| 面 | 当前值（写文档时） | 角色 |
|----|-------------------|------|
| **workspace Cargo** `[workspace.package].version` | `0.13.5` | **权威**：项目/发版语义版本 |
| 根目录 `VERSION` | `3.0.0` | **从属/遗留**：历史项目文件；`check-version.sh` 仍可能读它与 `package.json` 对比。**不得**再称为唯一权威；与 workspace 不一致时以 workspace 为准 |
| `crates/cli` / `crates/task` 包 `version` | `0.6.3` | **实现面**：独立声明的 crate 版本；`dijiang --version` 来自 CLI 的 `CARGO_PKG_VERSION`。与 workspace 权威尚未对齐，对齐属后续实现任务 |
| `crates/mcp-server` | `0.1.0` | **独立面**：MCP 包可与主项目不同步；发布策略可单独定 |

依赖项若写 `workspace = true`，跟的是 workspace **依赖版本表**，不是把 crate 自身 `version` 绑到 `0.13.5`。cli/task 目前仍自带 `version = "0.6.3"`。

## VERSION 文件

```
3.0.0
```

不含前缀 `v`。它是遗留从属文件，不是运行态或发版权威。

## 发版 / bump 步骤

在「不改号」任务之外真正发版时：

1. **先 bump 权威面**：`Cargo.toml` `[workspace.package].version`（或由 `finish-work --version-impact major|minor|patch`）
2. 决定是否同步从属面（本决策不要求本轮做）：
   - `VERSION` 是否改成与 workspace 同号或退役
   - CLI/task crate `version` 是否改为 `version.workspace = true` 或与权威同号
   - mcp 是否独立 bump
3. 抽查：`cargo metadata`、重新安装后的 `dijiang --version`（在 CLI 对齐前仍可能显示 crate 自有版本）
4. 提交：`chore: bump to <workspace-version>`（中文 Conventional Commits 亦可）

## 版本策略（作用于 workspace 权威）

| 影响范围 | 变更 | 示例（workspace） |
|----------|------|-------------------|
| 不兼容公开行为 / API | major | 0.13.5 → 1.0.0 |
| 向后兼容新功能 | minor | 0.13.5 → 0.14.0 |
| 向后兼容修复 | patch | 0.13.5 → 0.13.6 |
| 仅文档/测试/workflow | none | 不改权威版本 |

## 检查脚本

`.dijiang/scripts/check-version.sh` 当前主要核对 `VERSION` 与 `package.json`，**不能**单独证明与 workspace 权威一致。在脚本未改前，人工以 `Cargo.toml` 的 `[workspace.package].version` 为准。

## 决策记录

- **2026-07-23**：用户确认「以 workspace 0.x 为准，本轮不改号」。
- 此前「VERSION 唯一权威 / 收敛 pending」表述废止。
