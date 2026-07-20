---
name: dj-script
description: >
summary: 脚本或工具实现
phases: [implement]
risk: medium
  脚本和工具编写：独立于特性实现，专注于一次性或可复用的自动化脚本。
  Use when the user needs a script, CLI tool, automation, or utility that is not part of the main application.
  触发词：脚本、script、工具、utility、自动化、写个命令、CLI、批量处理。
---

参考规范：`.dijiang/references/decision-ladder.md`（写脚本前的决策阶梯）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 可运行的脚本/工具 |
| **Done when** | 脚本通过运行验证，输出符合预期 |
| **Evidence** | 脚本文件 + 运行结果输出 |
| **Output** | 脚本文件 + 使用说明（一次性脚本除外） |

# Script: 脚本和工具编写

写脚本和工具。与 `dj-implement` 的区别：脚本不进入主应用代码库，单独存放执行。

## 工作流

- 将任务状态推进到 `in_progress`：`dijiang task status $(dijiang task current) in_progress`


### 1. 理解需求

- 输入是什么？
- 输出是什么？
- 是一次性的还是需要维护的？

### 2. 实现

- 选择最直接的方式写——Shell、Python、JS 都可以，选最快的
- 硬编码 > 配置；stdin/stdout > 文件交互；同步 > 异步
- 一次性脚本：不写 help、不写错误处理、不写日志
- 要维护的工具：加 `--help`、退出码、关键路径错误提示

### 3. 验证

- 跑一次确认输出正确
- 有副作用（删除/覆盖/网络请求 / 远程操作）→ 先在 dry-run 或隔离环境验证

### 4. 保存

- 一次性脚本：告诉用户怎么跑，不存 repo
- 可复用工具：存到 `scripts/` 目录，加一句话注释说明用途

## 安全规则

- 有破坏性操作的脚本必须加 `--dry-run` 或 `--confirm` 模式
- 执行删除、覆盖、网络请求前先输出将要影响的对象

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 一次性脚本写 help/日志/错误码 | 写完就跑，不留痕迹 |
| 脚本放 repo 根目录 | 放 scripts/ 目录 |
| 删除文件直接 rm | 先 `--dry-run` 确认目标 |
| 删除文件直接 rm | 先 `--dry-run` 确认目标 |

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 有破坏性操作的脚本必须加 `--dry-run` 或 `--confirm`
2. 一次性脚本不写 help/日志/错误处理
3. 要维护的工具放 `scripts/` 目录
4. 选择最直接的语言——Shell > Python > JS，选最快的

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 一次性脚本写 help 和日志 | 花时间在不必要的事情上 | 一次性脚本写完就跑 |
| 脚本放 repo 根目录 | 污染仓库根 | 放 scripts/ |
| 用 TDD 节奏写一次性工具 | 过度工程 | 跑一次验证就行 |
