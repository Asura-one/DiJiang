---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'bf02c36c-0665-4712-a759-892e81a734c0'
  PropagateID: 'bf02c36c-0665-4712-a759-892e81a734c0'
  ReservedCode1: 'e0fcb3d6-1a05-4f00-b86e-06c1b93b00f0'
  ReservedCode2: 'e0fcb3d6-1a05-4f00-b86e-06c1b93b00f0'
---

# 版本管理流程

## 权威面

纯 skill 架构无 Cargo workspace，版本权威面为 `CHANGELOG.md` 的最新版本标题。

- 文件：根目录 `CHANGELOG.md`
- 当前版本：`1.3.0`
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

- ZH: 新增 / 变更 / 修复 / 移除
- EN: Added / Changed / Fixed / Removed