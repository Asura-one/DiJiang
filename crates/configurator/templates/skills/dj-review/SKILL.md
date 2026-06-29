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

与 `dj-check` 的关系：
- `dj-check`：深层验证（运行测试、类型检查、lint）— 偏执行
- `dj-review`：浅层审查（读 diff、识问题、checklist）— 偏阅读

先跑 `dj-check`，再跑 `dj-review`。

## 流程

### Step 1: 获取变更

```bash
git diff --stat HEAD   # 变更概览
git diff HEAD          # 完整 diff
git diff --cached      # staged changes
```

也可以是针对特定文件或目录的对比。

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

#### B. 安全（CRITICAL / HIGH）

- [ ] 用户输入有校验/转义？
- [ ] 敏感信息有泄露风险？
- [ ] 权限校验正确？
- [ ] 没有 shell 注入/ eval / exec ？

#### C. 可维护性（MEDIUM / HIGH）

- [ ] 命名清晰？(is/has/get/set prefix, 无缩写)
- [ ] 函数 <= 50 行（有例外但需理由）
- [ ] 没有死代码/注释掉的代码
- [ ] 没有魔法数字/字符串（应提取为常量）
- [ ] 测试覆盖率合理？
- [ ] 错误信息可理解？

#### D. 一致性（MEDIUM / LOW）

- [ ] 与项目现有风格一致
- [ ] 与 `.trellis/spec/` 规范一致
- [ ] 与同类功能使用相同模式
- [ ] 遵循了项目使用的框架惯例

#### E. 可评审性（LOW）

- [ ] diff 大小合理（建议 < 400 行）
- [ ] 一个 commit = 一个逻辑变更
- [ ] 有注释解释"为什么"（而非"是什么"）
- [ ] 测试清晰可读

### Step 4: 输出审查报告

```
=== 审查报告：<范围> ===

变更摘要:
  <stat 信息>

发现问题（按严重程度排序）:

[CRITICAL] <file>:<line> <问题描述>. <建议修复>.

[HIGH] <file>:<line> <问题描述>. <建议修复>.

[MEDIUM] <file>:<line> <问题描述>. <建议修复>.

[LOW] <file>:<line> <问题描述>. <建议修复>.

总结:
  <N> CRITICAL + <M> HIGH + <P> MEDIUM + <Q> LOW
  总体评估: <通过 / 有条件通过 / 需修改后重审>
```

### Step 5: 评估结果

| 判定 | 条件 | 动作 |
|------|------|------|
| ✅ 通过 | 无 CRITICAL，<=1 HIGH | 可直接合并 |
| 🔶 有条件通过 | 无 CRITICAL，2-3 HIGH | 修复 HIGH 后合并 |
| 🔴 需修改后重审 | 有 CRITICAL，或 >3 HIGH | 修复后重新 review |

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| diff 太大无法逐一审查 | 只审查关键文件（lib.rs, core/, API 层） | 用 dj-check 替代 |
| 需要 spec 对照但无 spec | 跳过一致性检查 | 标注"无 spec，无法检查一致性" |
| 项目有数百个 lint 警告 | 只关注 CRITICAL/HIGH 级别 | 标注"lint 警告过多，建议先清理" |
| 用户要求快速审查 | 跳过 LOW 级别 | 只输出 CRITICAL + HIGH |

## 边界

- 不运行测试（那是 dj-check 的事）
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
| 只报问题不给正面 | 好的代码也要肯定（"这部分处理不错："） |
