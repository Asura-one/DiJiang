---
name: dj-review
description: >
  代码评审辅助：对变更进行并行审查，同时检查 spec 匹配度和代码质量。
  使用 delegate_task 并行执行两个独立维度的审查，最后汇总。
  Use when reviewing PRs, inspecting diffs, before committing, or after code changes.
  触发词：review、审查、检查代码、审一下、code review、帮我看一下代码。
summary: 轻量只读审查
phases: [check]
risk: low
  代码评审辅助：对变更进行并行审查，同时检查 spec 匹配度和代码质量。
  使用 delegate_task 并行执行两个独立维度的审查，最后汇总。
  Use when reviewing PRs, inspecting diffs, before committing, or after code changes.
  触发词：review、审查、检查代码、审一下、code review、帮我看一下代码。
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 两个维度（spec 匹配度 + 代码质量）的审查汇总报告 |
| **Done when** | 两个子 agent 都完成审查并汇总 |
| **Evidence** | 审查日志、问题列表 |
| **Output** | 结构化审查报告（问题列表 + 严重度 + 所属维度） |

# Review: 并行代码审查

使用 `delegate_task` 并行执行两个维度的审查，最后汇总。两个子 agent 独立运行，互不污染上下文。

## 流程

### 0. 准备

```bash
git diff --stat HEAD
git log --oneline -5
```

确认变更范围、涉及的 PRD / issue 引用。

### 1. 并行审查（delegate_task）

同时提交两个子任务：

**子任务 A — Spec 匹配度**
```goal
审查以下 git diff 是否忠实实现了源需求文档（PRD / issue / spec）。
报告：遗漏的行为、超出范围的新增、与 spec 矛盾的设计。
只输出问题清单，不修改代码。
```
```context
- git diff 内容（从当前会话获取）
- 关联的 PRD / issue / spec 内容
- 变更的文件列表
```

**子任务 B — 代码质量**
```goal
审查以下 git diff 的内部质量。
报告：与项目现有模式的偏差、过度工程/过早抽象、潜在缺陷（空指针/资源泄漏/并发）、风格不一致。
只输出问题清单，不修改代码。
```
```context
- git diff 内容
- 项目现有代码风格/模式参考（从当前代码库推断）
- 变更的文件列表
```

### 2. 汇总

两个子任务完成后，汇总为一份审查报告：

```text
## 审查：<变更描述>
- Spec 匹配度：<通过/有问题>
- 代码质量：<通过/需改进>
- 安全性：<人工审阅重点，如涉及>
- 问题汇总：
  1. [严重] <问题描述>（来自：Spec/质量）
  2. [中等] <问题描述>
  3. [建议] <问题描述>
- 结论：<通过 / 需修改 / 需重新审查>
```

### 3. 输出

给用户最终结论和问题清单。不修改代码。

## 边界

- 不修改代码——只报告
- 不深度架构评审（那是 `dj-pattern` / `dj-reason` 的事）
- 子任务不互相引用——各自的发现独立汇报

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 串行审查两个维度 | 用 delegate_task 并行执行 |
| 改人家的代码 | 只报告问题 |
| 引入自己的风格偏好 | 按项目现有模式判断 |
参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 不修改代码——只报告审查结果
2. 不深度架构评审——那是 dj-pattern 的职责范围
3. 两个维度的子 agent 并行执行，互不引用
4. 每个问题必须标注严重度：严重/中等/建议

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 串行审查两个维度 | 时间翻倍 | 用 delegate_task 并行 |
| 改人家的代码 | 行为越界 | 只报告问题清单 |
| 引入自己的风格偏好 | 主观审查 | 按项目现有模式判断 |
| 只报问题不给建议 | reviewer 没有价值 | 问题 + 建议方向 |

参考规范：`.dijiang/references/intensity-levels.md`（强度等级：支持 lite/full/ultra）。
