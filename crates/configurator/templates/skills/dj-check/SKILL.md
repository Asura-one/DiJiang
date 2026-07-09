---
name: dj-check
description: >
  交付质量闸门：验证 diff 质量、功能完整性、安全性和回归风险，输出 finish-work 收口证据。
  Use when the user needs a delivery quality gate, completion verification, release-blocking check, or finish-work handoff.
  触发词：check、质量门禁、验收、验证、检查交付、finish-work 前检查。
---

# Check: 交付质量闸门

在交付前验证变更质量、功能完整性、安全性和回归风险。输出收口证据给 `dijiang-finish-work`。

## 原则

- **可验证** — 每个断言都必须有对应的运行命令或可复核操作
- **不自欺** — 跑不了的测试写 `not run` + 原因，不美化
- **只报告，不改代码** — 发现问题时标注位置和方向，不在本 skill 修

## 工作流

### 1. 理解变更

```bash
git status --short --branch
git diff --stat HEAD
```

**遵守 Code Task TDD Contract** — 每个断言必须有对应的运行命令。

### 2. 按维度验证

**功能完整性** — 改动的功能是否完整可用？
- 验证命令（构建/测试/lint）是否通过（GREEN command）
- 核心路径是否覆盖
- RED/Repro evidence 是否已确认

**安全性** — 是否有新的安全风险？
- 敏感信息硬编码
- 未校验的外部输入
- 新增网络/文件权限

**回归风险** — 改动的副作用范围？
- 被改模块的调用方是否检查过？
- 相关测试是否跑过？

### 3. 输出审查报告

```text
## 变更验证报告

### 变更摘要
<分支名> · <文件数> 个文件改动

### 验证结果
| 检查项 | 状态 | RED/Repro evidence / Exception |
|---|---|---|
| Typecheck | pass/fail/not run | <命令和输出摘要> |
| 测试 | pass/fail/not run | <命令和输出摘要> |
| Lint | pass/fail/not run | <命令和输出摘要> |
| 安全性 | pass/需要关注 | <发现摘要> |
| Regression scope | 低/中/高 | <理由> |
| Exception | none/原因 | <不可验证的具体原因> |

### 问题清单
- <问题描述>（严重/中等/轻微）

### 结论
✅ 通过 / ❌ 阻塞 / ⚠ 有条件通过
```

### 4. finish-work 数据准备

- `git status --short --branch`
- `git diff --stat HEAD`
- 验证报告摘要
- 版本决策建议

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 验证说"通过"但没跑命令 | 写 `not run` + 理由 |
| 发现问题直接改 | 只报告，不在 check 中修 |
| 只看代码不看测试 | 测试覆盖是质量的核心指标 |
