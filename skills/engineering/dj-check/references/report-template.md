# 变更验证报告模板

> 摘要：dj-check 第 3 步「输出审查报告」的标准模板（SKILL.md 外溢）。问题分四档：阻塞 / 应改 / Nit / FYI，只有前两档进阻塞清单。

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
- <问题描述>（阻塞 / 应改 / Nit / FYI；只有前两档进阻塞清单）

### 结论
✅ 通过 / ❌ 阻塞 / ⚠ 有条件通过
```
