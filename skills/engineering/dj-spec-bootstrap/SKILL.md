---
name: dj-spec-bootstrap
description: 扫描项目主要源码目录，为每个模块生成初始 spec 文件（.dijiang/spec/<module>/index.md）
disable-model-invocation: true
---

# dj-spec-bootstrap — Spec 初始化

扫描项目主要源码目录，为每个模块生成初始 spec 文件（`.dijiang/spec/<module>/index.md`）。

## 模块探测

按项目类型推断主要源码目录：

| 项目特征 | 模块来源 |
|---|---|
| Cargo.toml | `crates/` 或 `src/` 下的顶层模块 |
| package.json / tsconfig.json | `src/` 或 `lib/` 顶层目录 |
| go.mod | 顶层包目录 |
| 其他 | `.dijiang/spec/` 已有子域，或按领域拆分 |

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 每个模块有对应的 `.dijiang/spec/<module>/index.md` |
| **Done when** | 扫描完成，缺失的 index.md 已创建 |
| **Evidence** | `ls .dijiang/spec/` 列出所有模块子目录 |

## 流程

1. 读取 `.dijiang/config.toml` 获取 `project.name`
2. 按「模块探测」表格推断模块列表
3. 对每个模块：
   - 检查 `.dijiang/spec/<module>/index.md` 是否存在
   - 不存在时，生成包含基本模板的 index.md
4. 输出摘要（创建 N 个，跳过 M 个）

## Hard Rules

1. 永不修改已存在的 spec 文件
2. 只创建 `.dijiang/spec/<module>/index.md`，不创建子指南

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'd47f9e10-beaa-454d-ac00-f26ecbb49df0'
  PropagateID: 'd47f9e10-beaa-454d-ac00-f26ecbb49df0'
  ReservedCode1: 'c02e95c2-7e63-4ea4-9161-06517e9f23d9'
  ReservedCode2: 'c02e95c2-7e63-4ea4-9161-06517e9f23d9'
