---
name: dj-script
description: >
  脚本和工具编写：独立于特性实现，专注于一次性或可复用的自动化脚本。
  Use when the user needs a script, CLI tool, automation, or utility that is not part of the main application.
  触发词：脚本、script、写个工具、自动化、写个脚本、tool、utility、CLI工具。
---

# Script: 脚本和工具

## 职责

编写独立的脚本和工具——一次性任务脚本、CLI 工具、自动化脚本、数据处理管道。

## 与 implement 的区别

| | implement | script |
|---|---|---|
| 目标 | 应用功能 | 独立工具 |
| 生命周期 | 长期维护 | 可能用完即弃 |
| 测试 | 必须 | 视重要程度 |
| 代码规范 | 严格 | 务实 |

## 输入 / 输出

| Item | One-off script | Reusable tool |
|---|---|---|
| Input contract | explicit file/stdin/args sample | documented args and `--help` |
| Output contract | stdout/file diff/sample output | stable stdout, exit codes, help text |
| Verification | run once on fixture or real safe input | run happy path plus at least one bad input |
| Location | temp path or task-local scratch when disposable | project `scripts/`, `tools/`, or existing CLI area |
| Cleanup | report path and cleanup recommendation | keep as project artifact |

## 工作流

### 1. 确认需求

Collect exactly:

```text
Purpose: <what the script automates>
Inputs: <files / args / stdin / env vars>
Outputs: <stdout / files / side effects>
Safety: <read-only / writes files / deletes files / network / system>
Lifetime: <one-off / reusable>
Language: <project default or reason for another>
```

If the script writes, deletes, calls network, or touches system state, require dry-run output before the real run.

### 2. 设计最小接口

- One-off script: prefer direct constants plus clear top-of-file settings; avoid CLI framework.
- Reusable tool: add `--help`, input validation, non-zero exit codes, and stable output shape.
- Default to the project language and standard library.
- Do not introduce a dependency unless the script would become meaningfully riskier without it.

### 3. 实现

- Keep file placement explicit before writing.
- Use structured parsers for JSON/YAML/TOML/CSV instead of ad hoc string edits.
- Print what changed or would change.
- Never print secret values; redact suspicious tokens as `[REDACTED]`.

### 4. 运行验证

Minimum validation matrix:

```text
happy path: <command> => <result>
empty input: <command or n/a> => <result>
bad input: <command or n/a> => <result>
dry-run destructive path: <command or n/a> => <result>
```

For data-changing scripts, verify output files or diffs after running. If validation cannot run locally, mark it `not run` with the blocker.

### 5. 交付

Report: script path, exact command, sample output, validation result, and cleanup recommendation. Do not delete generated scripts unless the user explicitly asks in a separate cleanup request.

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 脚本运行报错 | 检查错误信息，修复语法/逻辑 | 用最简方式重写（去掉复杂部分） |
| 环境缺少依赖 | 用标准库替代 | 标注依赖需求，让用户安装 |
| 涉及删除/修改操作不安全 | 先 dry-run 展示影响 | 只输出将被修改的文件列表，不执行 |
| 输出格式不对 | 检查输入数据格式，调整解析逻辑 | 手动处理异常数据，输出到单独文件 |

## 🔴 CHECKPOINT · 破坏性操作确认

涉及删除、覆盖、网络操作时：
```
即将执行：<操作描述>
影响范围：<文件数/数据量>
可回滚：<是/否>

确认执行？(Y/n)
```

## 安全规则

- 脚本不处理凭证（不硬编码 token/password）
- 涉及删除/修改操作时先 dry-run 展示影响
- 不自动执行可能影响网络/系统的操作，除非用户确认

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 脚本里硬编码用户机器路径 | 用参数、环境变量，或顶部配置常量 |
| 不跑就声称能用 | 实际执行验证，或明确写 `not run` 和原因 |
| 一次性脚本用完直接删除 | 报告路径和清理建议，等用户明确要求 |
| 脚本引入整个框架 | 几行标准库能搞定的不引依赖 |
| 不加错误处理就跑破坏性操作 | 先 dry-run 展示影响 |
| 用字符串拼接改 JSON/YAML/TOML | 用结构化 parser 读写 |
| 输出 token/password 原文 | 输出 `[REDACTED]` 和字段名 |
