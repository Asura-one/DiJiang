# Code Task Contract

所有涉及代码实现或 bug 修复的任务自动适用以下合约。

## 合约

| 条款 | 含义 |
|---|---|
| **RED / Repro evidence** | 实现前先写失败测试，或 bug 修复前先确认可复现 |
| **GREEN command** | 实现/修复后执行验证命令，确认通过 |
| **Regression scope** | 确认改动不影响已有行为 |
| **Exception** | 无法自动化验证时，记录原因并说明验证方式 |

## 注意

- 本合约由 `dj-implement` / `dj-hunt` 在各自治流程中执行，`dj-dispatch` 只进行路由，不执行合约
- 不可自动化验证的场景（如 UI 测试不可达、外部 API 无 sandbox）需要显式记录在异常条款中
