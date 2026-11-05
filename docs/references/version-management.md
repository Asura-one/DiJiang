# 版本管理流程

## 权威面

纯 skill 架构无 Cargo workspace，版本权威面为 `CHANGELOG.md` 的最新版本标题。

- 文件：根目录 `CHANGELOG.md`
- 当前版本：`2.3.0`
- 版本号格式：`## [X.Y.Z] — YYYY-MM-DD`

## 发版 / bump 步骤

1. 在 `CHANGELOG.md` 顶部新增版本标题 `## [X.Y.Z] — 日期`
2. 按变更类型填写 section（新增/变更/修复/移除）
3. 提交：`chore: bump to <version>` 或随功能提交一起

## 版本策略

| 影响范围 | 变更 | 示例 |
|----------|------|------|
| 不兼容公开行为 | major | 1.3.0 → 2.0.0 |
| 向后兼容新功能/新 skill | minor | 1.3.0 → 1.4.0 |
| 向后兼容修复 | patch | 1.3.0 → 1.3.1 |
| 仅文档/测试/workflow | none | 不改版本 |

## CHANGELOG 结构要求

根 `CHANGELOG.md` 必须包含目标版本标题：

- `## [X.Y.Z]` 或 `## X.Y.Z`（可带日期）

且至少一个标准 section 含非空 bullet：

- ZH: 新增 / 变更 / 修复 / 移除 /（按需）弃用 / 安全
- EN: Added / Changed / Fixed / Removed / Deprecated / Security

补充规则：

- 撤回已发布版本时，版本标题加 `[YANKED]` 标记并注明原因；条目不删除，历史可查
- `git log` 堆砌是反模式：变更按类型分组提炼，写「为什么」而非流水
