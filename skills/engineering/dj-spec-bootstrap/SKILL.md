---
name: dj-spec-bootstrap
description: "扫描项目主要源码目录，为每个模块生成初始 spec 文件。 Use when a project lacks .dijiang/spec/ coverage and needs baseline specs generated. 触发词：生成 spec、初始规范、spec 引导、bootstrap、规范初始化、扫描模块。"
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

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 模块探测与实际目录不符 | 以实际源码目录为准修正模块列表 | 停下汇报目录，请用户确认模块划分 |
| 项目类型不在探测表内 | 按「按领域拆分」兜底 | 请用户给出模块边界 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 凭记忆列模块、不扫目录 | 从源码目录实际扫描 |
| 为凑 spec 数把非模块目录也建 index | 只对有实际源码的模块建 spec |

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
