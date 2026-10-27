---
name: dj-fullstack-testing
description: "全栈系统回归测试引擎：后端分批 pytest、API 深度验证（契约/Schema 校验、状态码语义、限流、幂等、落盘回读）、业务逻辑正确性校验、Playwright UI 交互测试、UX 启发式评估、结构化测试报告。 Use when the user asks for comprehensive/systematic/regression testing covering all modules, pages, and interactions. 触发词：全量回归、全面测试、系统测试、全栈测试、fullstack testing、覆盖度核对。"
---

参考规范：`docs/references/anti-patterns.md`、`docs/references/output-markers.md`、`docs/references/code-task-contract.md`。步骤 1-7 细化操作：读 `references/execution-detail.md`。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 全面回归测试报告（覆盖度 + 各层验证结论 + 发布建议） |
| **Done when** | 覆盖度核对完成（MISSING=0）且报告已输出 |
| **Evidence** | 测试日志、截图、覆盖率、checklist |
| **Output** | 结构化测试报告（docs/testing/）+ 功能枚举文档 |

## 职责

资深测试工程师视角的全面回归，覆盖四层：**API 层**（端点 + 契约/Schema + 状态码 + 限流 + 幂等 + 落盘回读）、**功能层**（从代码自动枚举的功能 + 异常路径）、**业务逻辑层**（单点结果正确性 + 跨模块流程/状态机）、**UI/UX 层**（交互控件 + UX 评估）。

任务级轻量护送由 `dj-regression-guard` 承担，它引用本技能三处机制（diff 映射、冒烟最小集、project-understanding 格式），不内嵌副本。

## Recursion Guard

你已身为 dj-fullstack-testing。直接执行测试流程，不得再调用或派发另一个 dj-fullstack-testing。

## 工作流

**0. 文档触发检查（commit-id 守门）**：读枚举文档头 commit 字段比对 `git log -1`；相同复用，不同才枚举。首次建档 `docs/project-understanding.md`（技术栈/启动/产物/风险分级），基础设施变更时才增量更新。变更按 diff 映射缩范围分级（见 references）。**缺陷验证 ≠ 回归测试**：先窄范围复测被修复项。

**1. 功能枚举（🔴 强制）**：从代码自动提取，不靠记忆。路由/后端命令/交互入口/异常路径/业务场景五清单合并为 checklist 总表（**预期结果必须写具体断言**，禁止"不报错"占位），落盘 enumeration 文档。🔴 交叉校验：每类 grep 独立计数，大于清单 → 回补。变更时生成测试用例（可自动化 → tests/，否则 test-case-spec.md）。

**2. 后端分批执行**：不一次跑全量。Web 每批 1-3 文件 `pytest ... --timeout=30`；Rust 每批一模块；清单与代码覆盖率双指标互为校验。注意 Cargo 子目录 `.rs` 发现陷阱（需显式 `[[test]]`）。细节见 references。

**3. 启动前后端**：按建档优先级（产物→Makefile→scripts→兜底），不临时现拼。FastAPI 例 `NO_PROXY=127.0.0.1,localhost uvicorn ... --port 8010 --reload` + curl health 200。Tauri 走 CDP；无设备降级（Rust 走 cargo test、UI 走 Playwright，报告标 SKIP 原因）。**平台陷阱**：系统代理拦截 localhost → `--noproxy`/`NO_PROXY`；httpx `trust_env=False`。

**3a. 冒烟测试（🔴 强制关卡 — 不通过禁止进 4/5/6）**：服务启动 + health 200 + 首页无白屏 + 登录走通 + 高风险核心 API 正常。任一失败 → 🛑 STOP，输出阻断性缺陷报告。

**4. API 端点深度验证**：基础（状态码+体量+耗时）+ 契约/Schema（有 OpenAPI 用 schemathesis，无则快照落盘 diff）+ 状态码语义（201/204/400/401/403/404/409）+ 限流（429+Retry-After）+ 幂等（重复 POST/Idempotency-Key）+ 数据落盘回读。矩阵细节见 references。

**5. UI 交互测试（双层）**：API 通过 ≠ UI 通过。5.1 逐个调用命令验 CRUD+边界；5.2 逐页验证矩阵 + 视觉基线 + axe + 控制台；5.3 UX 维度评估（能用 ≠ 好用，UX 清单独立计数）；5.4 oracle 比对（好用 ≠ 结果对），核心写操作多层次追踪（UI→HTTP→后端→DB→下游）。细节见 references。

**6. 覆盖度核对（🔴 强制 — 完成后才允许输出报告）**：🛑 STOP：MISSING > 0 必须补测。逐项 ✅/❌/⏭️/⬜；只验证"无报错"未比对结果的标 SKIP 注明。

**7. 结构化测试报告（→ docs/testing/）**：覆盖度总表 + API 结论 + 业务逻辑校验 + UX 清单 + 双覆盖率差异 + 视觉/可访问 + 非功能引用 + 缺陷清单 + **发布建议**（✅/⚠️/❌）。结构细节见 references。

## 配套专项技能

性能/安全/跨浏览器 → `nonfunctional-testing` 承担，完成后如需转它执行，结论回填报告。

## 陷阱

- 全量 pytest 超时→分批；PowerShell 常驻卡死→日志重定向+后台
- AIGC hook 异步改写文件 → 编辑前重读；反复冲突用一次性 Python 补丁脚本
- 截图留项目根 → 清理或入 .gitignore；测试脚本幂等 → 先清理同名残留

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

25 条反模式全表见 `references/execution-detail.md`。核心：凭记忆写清单、只测 API 不测 UI、不做覆盖度核对、弹窗只测确认、不测边界、全量一次跑、不查 commit-id、只看状态码不查结构、不测限流幂等、业务流不测、越权只靠前端隐藏、不验业务正确性、不冒烟直接全量、报告不给发布建议。

## 实战关键教训

五条实战教训（重复选项/金额比对/筛选子集/字段截断/库存扣减的多层核对）见 `references/execution-detail.md`。