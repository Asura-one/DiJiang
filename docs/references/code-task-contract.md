---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'cc608c04-f228-4c88-82a6-4911e5fd5e53'
  PropagateID: 'cc608c04-f228-4c88-82a6-4911e5fd5e53'
  ReservedCode1: '9c37cb11-2107-4dc9-8269-b346c2bf35cd'
  ReservedCode2: '9c37cb11-2107-4dc9-8269-b346c2bf35cd'
---

# Code Task Contract

所有涉及代码实现或 bug 修复的任务自动适用以下合约。

## 合约

| 条款 | 含义 |
|---|---|
| **RED / Repro evidence** | 实现前先写失败测试，或 bug 修复前先确认可复现 |
| **GREEN command** | 实现/修复后执行验证命令，确认通过 |
| **Regression scope** | 确认改动不影响已有行为（改前基线+改后回归由 `dj-regression-guard` 三明治协议承载） |
| **Exception** | 无法自动化验证时，记录原因并说明验证方式 |

## 注意

- 本合约由 `dj-implement` / `dj-hunt` 在各自治流程中执行，`dj-dispatch` 只进行路由，不执行合约
- 改码任务的改前基线与改后回归护送由 `dj-regression-guard` 承接，全量回归由 `dj-fullstack-testing` 承接
- 不可自动化验证的场景（如 UI 测试不可达、外部 API 无 sandbox）需要显式记录在异常条款中