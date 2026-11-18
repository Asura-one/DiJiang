---
name: dj-fullstack-testing
description: "全栈系统回归测试引擎：后端分批 pytest、API 深度验证（契约/Schema 校验、状态码语义、限流、幂等、落盘回读）、业务逻辑正确性校验、Playwright UI 交互测试、UX 启发式评估、结构化测试报告。 触发词：全量回归、全面测试、系统测试、全栈测试、fullstack testing、覆盖度核对。"
---

参考规范：`skills/engineering/dj-meta/references/anti-patterns.md`、`skills/engineering/dj-check/references/output-markers.md`、`skills/engineering/dj-implement/references/code-task-contract.md`。深度细节（步骤细化/性能七问/产物契约）：`references/execution-detail.md`、`references/benchmark-checklist.md`、`references/test-artifacts.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 全面回归测试报告（覆盖度 + 各层验证结论 + 发布建议） |
| **Done when** | 覆盖度核对完成（MISSING=0）且报告已输出 |
| **Evidence** | 测试日志、截图、覆盖率、checklist |
| **Output** | 结构化测试报告（docs/testing/）+ 功能枚举文档 |

## 职责

资深测试工程师视角的全面回归，覆盖 API/功能/业务逻辑/UI-UX 四层（各层详释 → `references/execution-detail.md`）

任务级轻量护送由 `dj-regression-guard` 承担（分工 → `references/execution-detail.md`）

## Recursion Guard

你已身为 dj-fullstack-testing。直接执行测试流程，不得再调用或派发另一个 dj-fullstack-testing。

## 工作流

**0. 文档触发检查（commit-id 守门）**：比对枚举文档 commit 与 `git log -1`，不同才枚举；**缺陷验证 ≠ 回归测试**，先窄范围复测 → `references/execution-detail.md`

**1. 功能枚举（🔴 强制）**：从代码自动提取不靠记忆；断言写具体结果、禁“不报错”占位；标需求依据 vs 经验臆测；交叉校验独立计数走 `scripts/enum_count.py` → `references/execution-detail.md`

**2. 后端分批执行**：不一次跑全量；Web 每批 1-3 文件、Rust 每批一模块；清单与覆盖率互校（Cargo 陷阱 → `references/execution-detail.md`）

**3. 启动前后端**：按建档优先级；多服务走 `scripts/with_server.py`；无设备降级标 SKIP；代理陷阱 → `references/execution-detail.md`

**3a. 冒烟测试（🔴 强制关卡——不过禁止进 4/5/6）**：服务启动 + health 200 + 首页无白屏 + 登录走通 + 高风险核心 API 正常。任一失败 → 🛑 STOP 输出阻断性缺陷报告。

**4. API 端点深度验证**：契约/Schema、状态码语义、限流、幂等、落盘回读（矩阵 → `references/execution-detail.md`）

**5. UI 交互测试（双层）**：API 通过 ≠ UI 通过；命令 CRUD → 矩阵+视觉基线+axe+控制台（`scripts/webapp_probe.py`）；UX 评估；oracle 比对（UI→HTTP→后端→DB→下游）→ `references/execution-detail.md`

**6. 覆盖度核对（🔴 强制——完成后才许出报告）**：🛑 MISSING > 0 必须补测；逐项 ✅/❌/⏭️/⬜，只验证“无报错”未比对结果标 SKIP。

**7. 结构化测试报告（→ docs/testing/）**：覆盖度+各层结论+缺陷清单+**发布建议**（✅/⚠️/❌）；六分类分流；机读摘要 `scripts/report_lint.py`（契约 `references/test-artifacts.md`）

## 配套专项技能

性能/安全/跨浏览器 → `nonfunctional-testing`（结论回填）。**性能数字写入报告/决策前必过七问**，说不出限制器 → 标 inconclusive → `references/benchmark-checklist.md`

## 陷阱

环境与工具陷阱（PowerShell/AIGC hook/截图残留/脚本幂等）→ `references/execution-detail.md`

## 验证清单（覆盖度之外）

- [ ] project-understanding.md 已建立/复用
- [ ] 冒烟通过才进正式测试
- [ ] commit-id 已比对，变更才枚举
- [ ] 功能清单从代码自动提取 + 交叉校验完成
- [ ] API 契约/状态码/限流/幂等/落盘已验证
- [ ] UX 已评估；业务逻辑 oracle 已比对
- [ ] **覆盖度核对完成：MISSING = 0**
- [ ] 报告生成 + 发布建议明确

## 🔴 反例与黑名单

25 条反模式全表见 `references/execution-detail.md`。核心：凭记忆写清单、只测 API 不测 UI、不做覆盖度核对、弹窗只测确认、不测边界、全量一次跑、不查 commit-id、只看状态码不查结构、不测限流幂等、业务流不测、越权只靠前端隐藏、不验业务正确性、不冒烟直接全量、报告不给发布建议、浅层排查就下「没问题」结论（无缺陷验证走失败升级阶梯 L1-L7，未到 L4 改写为「未充分排查」，见 `skills/engineering/dj-audit/references/coverage-ledger.md`）。多轮审计/测试的覆盖范围记入跨轮 additive 台账（同上文档），新轮只补缺口。

## 实战关键教训

五条实战教训（重复选项/金额比对/筛选子集/字段截断/库存扣减的多层核对）见 `references/execution-detail.md`。
