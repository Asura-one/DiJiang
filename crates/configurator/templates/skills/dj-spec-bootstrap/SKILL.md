---
name: dj-spec-bootstrap
description: 扫描项目 crates 目录，为每个 crate 生成初始 spec 文件
summary: 扫描 crates 目录并为每个 crate 生成初始 spec 文件
phases: [align]
risk: low
---

# dj-spec-bootstrap — Spec 初始化

扫描项目 crates 目录，为每个 crate 生成初始 spec 文件（`.dijiang/spec/<crate>/index.md`）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 每个 crate 有对应的 `.dijiang/spec/<crate>/index.md` |
| **Done when** | 扫描完成，缺失的 index.md 已创建 |
| **Evidence** | `ls .dijiang/spec/` 列出所有 crate 子目录 |

## 流程

1. 读取 `.dijiang/config.toml` 获取 `project.name`
2. 扫描 `crates/` 目录获取 crate 列表
3. 对每个 crate：
   - 检查 `.dijiang/spec/<crate>/index.md` 是否存在
   - 不存在时，生成包含基本模板的 index.md
4. 输出摘要（创建 N 个，跳过 M 个）

## Hard Rules

1. 永不修改已存在的 spec 文件
2. 只创建 `.dijiang/spec/<crate>/index.md`，不创建子指南
