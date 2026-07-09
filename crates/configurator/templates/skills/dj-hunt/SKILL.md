---
name: dj-hunt
description: >
  系统化排查 bug：先定位根因，再修复。尤其擅长回归和"以前好现在坏"的情况。
  Use when the user reports errors, crashes, regressions, failing tests,
  or unexpected behavior changes — anything that needs root cause investigation.
  触发词：修 bug、出错了、报错、crash、不工作、坏了、hunt、调查、排查。
---

# Hunt: 系统化排查 bug

先定位根因，再修复。不跳步骤、不猜原因。

## 铁律

1. **先复现，再分析** — 不能稳定复现的问题，优先找复现条件，不猜原因。
2. **先定位，再修复** — 必须找到根因（特定函数/变量/逻辑行）再修。不"顺便修周边"。
3. **一次改一个变量** — 修复后不绿就回退，换方向，不堆叠补丁。
4. **每次修复后跑 regression** — 确认没引入新问题。

## 工作流

### 1. 复现

- 从用户的错误报告或描述中提取最小复现步骤。
- 用调试输出、测试用例或人工操作确认问题可复现。
- 不可复现 → 记录已知条件和尝试过的环境变化，缩小搜索范围。

### 2. 代码定位

有明确错误线索时溯源：

- 查询项目历史教训：`dijiang mem recall --query "<错误信息>"`，检查是否有类似的 findings/patterns。
- 报错信息 → 堆栈 → 生产代码入口
- 二分法：`git bisect` 定位引入 commit（回归场景）
- 搜索关键词：grep 错误信息、变量名、函数名
- 检查 Init/Load/Compile 这些初始化路径是否被实际调用

### 3. 根因确认

找到根因候选后，用最小方式确认：

```text
假设：<认为根因是 X>  // RED/Repro evidence
验证：<修改 X 后问题消失> / <不改 X 但绕过它，问题依然存在>
结论：<确认/排除 X>
```

如果验证不能自动化，保留人工可复核的输入输出证据。

### 4. 修复

遵守 Code Task TDD Contract — 先固定边界，再修复。
- 只改找到了根因的部分，不改周边代码。
- 通知用户 worktree 路径，后续验证在 worktree 中执行。
- 修复后跑 regression（GREEN command），确认问题不复现且相关行为不受影响。
- 跑全量测试 Regression scope（或记录不跑的原因）。
- 无法验证时记录 Exception 及原因。

### 5. 收尾

准备给 `dijiang-finish-work` 的数据：
- `git status --short --branch`
- `git diff --stat HEAD`
- 根因分析摘要（一句话）
- 验证命令和结果
- 版本决策建议

参考 `references/hitl-loop.sh` 在人工介入调试时使用。

## 修复失败计数器

同一 bug 连续 3 次修复尝试失败后，停止当前方向，做 Break-Loop 报告：
- 当前假设和验证结果
- 还有哪些路径没查过
- 是否需要换工具（调试器、log 增强、分支排查）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不复现就猜原因 | 先稳定复现再分析 |
| 一次性改多个地方 | 一次改一个变量，验证再继续 |
| 修复完不跑回归 | 确认相关行为没坏 |
| 同一方向死磕 3 次以上 | 换思路，做 Break-Loop 报告 |
