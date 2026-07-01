---
name: dj-review
description: >
  代码评审辅助：对变更进行轻量级审查，检查质量、一致性、规范性。
  Use when reviewing PRs, inspecting diffs, before committing, or after code changes.
  触发词：review、评审、code review、diff review、PR review、审查。
---

# Review: 代码评审

## 职责

对代码变更进行结构化审查，输出可操作的审查意见。

## 与 `dj-check` 的关系

- `dj-review` 是轻量 diff / PR 人工审查，只读变更并输出 findings-first 报告。
- `dj-review` 不运行完整验证、不修改代码、不替代 `dj-check`。
- 任务需要交付前质量闸门时，只在结论中标记“需要 `dj-check` 后续”，不在 `dj-review` 内自行切换。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | diff 范围、审查目标、相关规范、变更文件 |
| 输出 | findings-first 审查报告，包含严重程度、`file:line` 证据、修复建议和最终结论 |
| 非目标 | 不修改代码、不运行完整质量闸门、不替代 `dj-check` |

## 流程

### Step 1: 获取变更

```bash
git diff --stat HEAD
git diff --name-only HEAD
git diff HEAD
git diff --cached --stat
git diff --cached --name-only
git diff --cached
```

Also accept a user-provided PR diff, file path, commit range, or staged-only scope.

## 🔴 CHECKPOINT · 审查范围

审查前先报告：

```text
范围：<working tree / staged / commit range / files>
文件：<N 个，关键路径>
审查目标：<正确性 / 安全 / 可维护性 / 快速过一遍 / 完整 diff review>
是否运行测试：no
是否修改代码：no
是否需要 dj-check 后续：<yes/no + 原因>
```

🛑 STOP if the user expects a release-blocking quality gate, test execution, or implementation. Report the required follow-up instead of switching skills inside `dj-review`.

### Step 2: 理解变更范围

```
=== 变更摘要 ===
文件数: <N>
新增: <N> 行
删除: <N> 行
涉及模块: <模块列表>
变更类型: <新增功能 / 修复 bug / 重构 / 文档 / 测试>
```

### Step 3: 逐项审查

按以下维度逐项审查，每个发现标注等级：

```
[CRITICAL] — 必须修复（功能错误、安全问题、数据损坏）
[HIGH]     — 应该修复（逻辑模糊、严重的代码坏味）
[MEDIUM]   — 建议改进（代码规范、命名、注释）
[LOW]      — 值得注意（风格偏好、微优化）
```

#### A. 正确性（CRITICAL / HIGH）

- [ ] 有未处理的错误路径？
- [ ] 空值/边界情况有处理？
- [ ] 并发/竞态有考虑？
- [ ] 回滚/幂等有保证？
- [ ] 状态管理一致？

#### B. 对抗式审查（CRITICAL / HIGH）

站在恶意用户、异常数据、资源耗尽和未来维护者的角度走完整路径：
- [ ] 恶意输入、超大输入、未来时间、乱码或空数据会怎样？
- [ ] 重试、缓存、队列、worker、定时任务是否可能无限循环或污染状态？
- [ ] 外部 API、文件系统、网络、数据库失败时是否会泄漏、重复执行或静默丢数据？
- [ ] 并发请求、重复提交、乱序事件是否破坏幂等？

#### C. 安全（CRITICAL / HIGH）

- [ ] 用户输入有校验/转义？
- [ ] 敏感信息有泄露风险？
- [ ] 权限校验正确？
- [ ] 没有 shell 注入/ eval / exec ？

#### D. 可维护性（MEDIUM / HIGH）

- [ ] 命名清晰？(is/has/get/set prefix, 无缩写)
- [ ] 函数 <= 50 行（有例外但需理由）
- [ ] 没有死代码/注释掉的代码
- [ ] 没有魔法数字/字符串（应提取为常量）
- [ ] 测试覆盖率合理？
- [ ] 错误信息可理解？

### Step 4: 输出审查报告

```
=== 审查报告：<范围> ===

发现问题（按严重程度排序）:

[CRITICAL] <file>:<line> <问题描述>. <建议修复>.
[HIGH] <file>:<line> <问题描述>. <建议修复>.
[MEDIUM] <file>:<line> <问题描述>. <建议修复>.
[LOW] <file>:<line> <问题描述>. <建议修复>.

未发现阻塞问题: <yes/no>

变更摘要:
  <stat 信息>

总体评估: <通过 / 有条件通过 / 需修改后重审>
计数: <N> CRITICAL + <M> HIGH + <P> MEDIUM + <Q> LOW
```

发现优先。正面观察只在发现之后出现，且仅用于说明剩余风险。

### Step 5: 评估结果

| 判定 | 条件 | 动作 |
|------|------|------|
| ✅ 通过 | 无 CRITICAL，<=1 HIGH | 可直接合并 |
| 🔶 有条件通过 | 无 CRITICAL，2-3 HIGH | 修复 HIGH 后合并 |
| 🔴 需修改后重审 | 有 CRITICAL，或 >3 HIGH | 修复后重新 review |

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| diff 太大无法逐一审查 | 只审查关键文件（lib.rs, core/, API 层） | 标注“需要 `dj-check` 后续” |
| 需要 spec 对照但无 spec | 跳过一致性检查 | 标注"无 spec，无法检查一致性" |
| 项目有数百个 lint 警告 | 只关注 CRITICAL/HIGH 级别 | 标注"lint 警告过多，建议先清理" |
| 用户要求快速审查 | 跳过 LOW 级别 | 只输出 CRITICAL + HIGH |

## 边界

- 不替代 `dj-check` 作为交付前质量闸门
- 不修改代码
- 设计决策不参与讨论（只检查实现质量）
- 一个 review 一个 scope（不跨变更审查）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|-----------|------------|
| 提风格意见但不指明行号 | 每个问题都标注 `<file>:<line>` |
| 把偏好当问题提 | 标注为 LOW 且注明"个人偏好" |
| 写长篇大论不给具体修复 | 每个 CRITICAL/HIGH 附建议修复 |
| 对设计决策吹毛求疵 | 专注于实现质量和正确性 |
| 先写鼓励再讲问题 | findings first，正面观察放在风险之后 |
| review 过程中直接改代码 | 只报告；修复作为后续项交给 workflow 或用户显式任务 |
| 把没跑测试说成通过 | 明确写 `Will run tests: no` 或 `not run` |
