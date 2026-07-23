# Changelog Convention — 更新日志约定

## 版本格式

遵循 [Semantic Versioning](https://semver.org/)：
- `MAJOR.MINOR.PATCH`（如 `0.1.0`）
- 有 Cargo workspace 时，版本权威为 `Cargo.toml` `[workspace.package].version`（ADR 004）

## 版本影响

`finish-work` 流程要求指定版本影响：

```bash
dijiang finish-work --version-impact <major|minor|patch|none>
```

- `major` — 不兼容的 API 变更
- `minor` — 向后兼容的功能新增
- `patch` — 向后兼容的 bug 修复
- `none` — 不影响版本（文档、重构等）；若权威版本相对 HEAD 已变，CLI 拒绝

## CLI 硬门禁

`version-impact ≠ none` 时，CLI **强制**校验根 `CHANGELOG.md`：

1. 文件存在
2. 含目标版本标题：`## [X.Y.Z]` 或 `## X.Y.Z`（可带日期）
3. 至少一个标准 section 含非空 bullet  
   - EN: Added / Changed / Fixed / Removed  
   - ZH: 新增 / 变更 / 修改 / 修复 / 移除

缺省则 finish 失败，并打印最小模板。CLI **不**自动撰写正文。

版本读取顺序：Cargo workspace → 根 package.json → 根 VERSION。  
自动 bump **仅** Cargo workspace，并同步已有 `VERSION` 文件。

## 发布流程

1. 在根 `CHANGELOG.md` 写好目标版本条目
2. 指定 version-impact 并运行 finish-work（CLI bump + 校验）
3. Git commit 包含版本与 CHANGELOG 更新
4. Tag 使用 `v{version}` 格式（如 `v0.1.3`）

## 双 changelog

| 文件 | 角色 |
|------|------|
| 根 `CHANGELOG.md` | 产品发版日志；finish gate 只校验此文件 |
| `crates/configurator/src/changelog.md` | `dijiang update` 展示用，不参与 finish gate |

## 更新日志文件结构

`CHANGELOG.md`（项目根目录）按版本倒序排列，每个版本包含：
- ### Added / 新增
- ### Changed / 变更
- ### Fixed / 修复
- ### Removed / 移除
