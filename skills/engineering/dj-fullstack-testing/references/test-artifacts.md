# 测试产物契约：用例与报告的机读标准

> 摘要：dj-fullstack-testing 两大落盘产物的硬契约——docs/test-cases/ 用例文件与测试报告机读摘要段。`scripts/case_lint.py` 校验 §1 红线；§2 由报告产出方遵循（校验脚本落地时以本文件为准）。契约是权威定义，脚本只做可机械判定部分。

## 1. 用例文件契约（docs/test-cases/*.md）

**结构**：`## N. 模块名` 分章（N 从 1 连续不重复）；代码证据、缺陷记录一律进文末附录章（`## 附录…`），不混入用例正文。

**编号**：每条唯一 `TC-{模块号}-{序号}`（如 TC-02-03），模块号 = 所在 `## N.` 的 N。

**优先级**：名称行尾 `[P0|P1|P2]`。P0 = 冒烟级（挂了核心功能不可用）；冒烟最小集按 P0 抽取。

两种写法，同一文件可混用；复杂用例用展开式：

```markdown
- **TC-01-02 连续输错密码锁定账户** [P1]
  - 前置条件: 已知账号 admin，密码已输错 5 次
  - 操作步骤: 第 6 次输错密码并提交
  - 预期结果: 页面提示「已锁定 15 分钟」，且该请求返回 423
  - 验证方式: 查 ops_audit 表最新 login_lock 记录时间戳
```

紧凑单行（信息简单时）：`- TC-01-03 未登录访问列表页，预期 401 且响应体含 error 字段 [P1]`

**红线（case_lint E 级，用例写完必跑）**：

1. 预期结果必须是**可观察的具体断言**——「不报错 / 正常 / 正确处理 / 符合预期 / 视情况」类占位出现在预期里即错（SKILL 步骤 1 原文红线）
2. 正文禁占位符：`{xxx}`、`<xxx>`、`某某`类值；待补数据写 `TODO：向 {谁} 索取`
3. 异步/延迟类预期必须带判定时限（如「15 分钟内」「轮询 3 次内」）（W1）
4. 预期唯一：含「或」的预期拆条（W4）
5. P0 占比 ≥35% 时逐条复核是否真属冒烟级（W3）——冒烟集失去筛选意义即失效

## 2. 报告机读摘要段（步骤 7 报告的末节）

每份测试报告**文末最后一个代码块**为以下 yaml 摘要；流水线与后续门禁**只解析此段**：

```yaml
report_meta:
  project: <项目标识>
  date: 2026-11-18            # 报告日期
  source_run: manual-full     # pr-smoke | nightly-full | release-gate | manual-full
  summary:
    total: 48                 # checklist 总项数
    passed: 40
    failed: 3                 # 含分流后待处置项
    blocked: 1                # 环境阻塞待重跑
    missing: 0                # 覆盖度核对 MISSING，必须为 0 才允许出报告
  exit_code_hint: 1           # 0 可发布 | 1 存在阻断缺陷 | 2 有条件发布 | 3 未完成
  triage:                     # failed 条目的失败六分类计数（判定树见 execution-detail.md §8；六类之和 = failed）
    A: 2                      # 真缺陷 → 缺陷清单
    B1: 0                     # 预期变更未跟上（须挂变更依据）
    B2: 0                     # 用例自身写错 → 修订用例
    C: 1                      # 环境与依赖故障 → 修复后原样重跑
    D: 0                      # flaky → 跨轮追踪，不算通过
    U: 0                      # 证据不足待人工，不许硬编结论
  bugs:
    - id: BUG-01
      severity: S0            # S0 阻断 | S1 严重 | S2 一般
      status: 新建            # 新建 | 已修复待验证 | 已验证关闭 | 不予修复
```

**约束**：摘要段唯一且在文末；必填字段齐全；枚举不越界；`passed+failed+blocked+missing = total`；`failed = A+B1+B2+C+D+U`；bugs 每条三字段齐全。

## 3. 校验脚本

| 脚本（技能内 scripts/） | 校验对象 | 契约 |
|------|---------|---------|
| `case_lint.py` | docs/test-cases/*.md | 本文件 §1 |
| `report_lint.py` | docs/testing/*.md 的报告机读段 | 本文件 §2 |

来源：用例格式与 lint 思路 Adapted from fishzjp/qa-skills core/{case-format.md, case_lint.py}（MIT）；报告机读段 Adapted from core/report-template.md + report_meta_check.py（MIT）。
