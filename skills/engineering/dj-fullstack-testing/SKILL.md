---
name: dj-fullstack-testing
description: "全栈系统回归测试引擎：后端分批 pytest、API 深度验证（契约/Schema 校验、状态码语义、限流、幂等、落盘回读）、业务逻辑正确性校验、Playwright UI 交互测试、UX 启发式评估、结构化测试报告。 Use when the user asks for comprehensive/systematic/regression testing covering all modules, pages, and interactions. 触发词：全量回归、全面测试、系统测试、全栈测试、fullstack testing、覆盖度核对。"
---

参考规范：`docs/references/anti-patterns.md`（跨技能行为约束）、`docs/references/output-markers.md`（输出标记）、`docs/references/code-task-contract.md`（代码任务合约）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 全面回归测试报告（覆盖度 + 各层验证结论 + 发布建议） |
| **Done when** | 步骤 6 覆盖度核对完成（MISSING=0）且报告已输出 |
| **Evidence** | 测试执行日志、截图、覆盖率报告、checklist |
| **Output** | 结构化测试报告（docs/testing/）+ 功能枚举文档 |

# Fullstack Testing: 全栈系统回归引擎

对项目执行**资深测试工程师视角**的全面回归测试，覆盖四层：
- **API 层**：端点验证 + 契约/Schema 校验 + 状态码语义 + 限流 + 幂等 + 落盘回读
- **功能层**：从代码自动枚举的功能逻辑 + 异常路径测试
- **业务逻辑层**：单点结果正确性（预期 vs 实际的断言比对，非"无报错"）+ 跨模块业务流程与状态机迁移
- **UI/UX 层**：交互控件验证 + UX 体验评估

本技能是全量回归**引擎**。任务级改码前后的轻量护送由 `dj-regression-guard` 承担，它引用本技能的三处机制（0.1a diff 映射表、3a 冒烟最小集、`docs/project-understanding.md` 格式），不内嵌副本。

## 工作流

### 0. 文档触发检查（commit-id 守门）

**目标**：避免重复枚举，仅当代码变更时才重新执行枚举与测试用例生成。

#### 0.0 项目理解建档（🔴 首次运行，或技术栈/基础设施大改时执行）

**目标**：避免每次测试都从零摸索"这是什么项目、怎么启动、什么技术栈"，把一次性认知持久化。

**首次运行时**（`docs/project-understanding.md` 不存在）：
1. **技术栈**：前端框架 / 后端框架 / 数据库 / 缓存 / 消息队列
2. **启动方式**：`Makefile` / `package.json` scripts / `docker-compose.yml` 有则记录标准命令；均没有才记录手动步骤
3. **构建产物/已安装应用**：`dist/`、`target/release/` 等产物比 dev server 更接近用户真实版本，优先
4. **核心业务域与风险分级**：推断主要业务模块，标注资金/权限/数据一致性高风险模块
5. 落盘到 `docs/project-understanding.md`：技术栈 | 启动方式 | 构建产物位置 | 核心业务域与风险分级 | 快速层定义（由 dj-regression-guard 追加）

**非首次**：直接读取复用；仅当 commit 变更涉及 `package.json` / `Cargo.toml` / `Makefile` / `docker-compose.yml` 时才增量更新对应字段。

#### 0.1 检查流程
1. 读 `docs/enumeration/feature-enumeration.md` 头部 `commit:` 字段（首次为空，触发全量枚举）
2. 执行 `git log -1 --format=%H`
3. 比对：相同 → 跳过步骤 1，复用现有文档进入步骤 2；不同 → 执行步骤 1 全量枚举并更新 commit id

#### 0.1a Diff-aware 测试范围缩小（commit 变更时）

```bash
git diff <旧 commit>..<当前 commit> --name-only
```

按变更文件路径映射到受影响模块，生成优先级分级：

| 变更文件匹配模式 | 受影响模块 | 优先级 |
|------------------|-----------|--------|
| `src/pages/<page>/**` / `src/views/<page>/**` | 该页路由 + 交互入口 | P0 |
| `src/api/**` / `src/services/**` | 所有调用该 API 的页面 | P0 |
| `src/components/<comp>/**` | 引用该组件的页面 | P1 |
| `src/router/**` / `src/App.tsx` | 全部路由 | P0 |
| `src/utils/**` / `src/lib/**` | 依赖该工具的模块 | P1 |
| `tests/**` | 对应被测模块 | P2 |
| `package.json` / `Cargo.toml` 等依赖 | 全部（可能全局影响） | P0 |

输出：`变更影响分析`（变更文件 | 受影响模块 | 优先级），追加到 `.temp/test-coverage-checklist.md` 头部。步骤 2-5 按优先级执行。

**缺陷验证 ≠ 回归测试**：若本次 commit 是修复上一轮缺陷，先窄范围复测被修复项（确认 PASS），再按优先级顺序做常规回归。

#### 0.2 文件位置约定

| 文件 | 目录 | 用途 | 生命周期 |
|------|------|------|----------|
| `docs/project-understanding.md` | docs/ | 项目认知（技术栈/启动/产物/风险分级） | 首次建立，基础设施变更时增量更新 |
| `docs/enumeration/feature-enumeration.md` | docs/ | 功能枚举总表 | commit-id 触发更新 |
| `docs/test-cases/test-case-spec.md` | docs/ | 不可自动化用例规格 | commit-id 触发更新 |
| `tests/` | tests/ | 可自动化测试脚本 | commit-id 触发更新 |
| `.temp/test-coverage-checklist.md` | .temp/ | 运行时 checklist | 每次从枚举文档复制 |

### 1. 功能枚举（🔴 强制 — 测试前必须完成）

**目标**：从代码自动提取完整功能清单，不靠记忆/经验决定测什么。

#### 1.1 前端路由枚举
提取所有路由：路径 → 页面组件 → 文件位置 → 是否需登录。输出：`路由清单`。

#### 1.2 后端 API / 命令枚举
- **Web 后端**：从路由注册（FastAPI `include_router` / Express `app.use` / Spring `@RequestMapping`）提取端点
- **Tauri 后端**：从 `generate_handler!` / `#[tauri::command]` 提取命令
- 输出：`后端命令清单`（端点 | 方法 | 参数签名 | 是否被前端调用）

#### 1.3 交互入口枚举（最关键 — 漏测的根源）
对每个页面组件文件，提取所有交互入口：`@click/@submit/@change/@input`、`data-test` 标记、表单控件、弹窗/抽屉、条件渲染区域（v-if/v-show）、下拉/选择器、日期/时间选择器（含禁用日期与边界）、开关切换、链接跳转、右键/长按、拖拽、滚动/无限滚动、图表交互、键盘事件、复制粘贴导入导出、列排序。

输出：`交互入口清单`（路由 | 组件 | 交互类型 | 元素标识 | 关联后端命令 | 优先级）。

#### 1.4 异常路径清单（必选）
删除被引用记录、边界值输入（0/负数/超大/空/特殊字符）、弹窗可关闭、空状态、并发/重复提交、限流与配额、幂等/重放、契约/schema 变更、权限/未登录、搜索筛选、日期范围逻辑、分页/加载、网络异常、数据一致性冲突、文件上传异常、图表/大列表性能边界、404/错误页、注入载荷。输出：`异常路径清单`。

#### 1.4a 业务场景清单（🔴 控件全绿 ≠ 流程能走通）
以**业务流程**为单位（非页面）：核心用户旅程、状态机迁移表、多角色数据隔离（API 直调验证，不能只靠前端隐藏按钮）。旅程级测试落 `tests/test_e2e_<scenario>.spec.ts`。

#### 1.5 生成功能清单总表
合并四清单为 checklist 表，作为测试执行的 basis：

| 序号 | 路由 | 组件 | 交互入口 | 类型 | 预期结果 | 后端命令 | 测试方法 | 状态 |
|------|------|------|----------|------|----------|----------|----------|------|

**"预期结果"必须基于已知测试数据/业务规则写出具体断言**，不允许"不报错/正常显示"这类无法证伪的占位描述。此表保存到 `.temp/test-coverage-checklist.md`，测试中逐项更新。

#### 1.6 文档落盘
枚举结果持久化到 `docs/enumeration/feature-enumeration.md`，头部含 `<!-- commit: <id> -->` + `<!-- updated: <ISO> -->`，含路由/后端命令/交互入口/异常路径/总表章节。checklist 复制到 `.temp/`。

#### 1.6a 枚举完整性交叉校验（🔴 强制）
对每个枚举类别用 grep/工具命令给独立计数，与清单条目数交叉比对。独立计数**大于**清单 → 判定枚举不完整，回补再生成，禁止直接进入 1.7。结果写入枚举文档"交叉校验结果"节。

| 枚举类别 | 独立计数示例命令 | 比对对象 |
|---------|----------------|---------|
| 前端路由 | `grep -rE "path:\s*['\"]" src/router \| wc -l` | 路由清单行数 |
| 后端 API | `grep -rE "@app\.(get\|post\|put\|delete)\|#\[tauri::command\]" -r . \| wc -l` | 后端命令清单行数 |
| 交互入口 | `grep -rE "@click\|@submit\|@change\|onClick=" --include="*.vue" --include="*.tsx" -r src \| wc -l` | 交互入口清单行数 |

#### 1.7 TDD 测试用例生成（🔴 commit 变更时）

对功能清单每一项判断是否可自动化：
- 可自动化 → `tests/`：`test_api_<module>.py` / `test_ui_<page>.spec.ts`，脚本头部注释 commit id + `# Covers: TC-xxx`
- 不可自动化 → `docs/test-cases/test-case-spec.md`（TC-xxx：触发方式/预期行为/测试方法/优先级）
- 同一功能点在脚本与文档中**不重复定义**，互相引用编号

### 2. 后端测试分批执行（pytest / cargo test）

- **不要一次跑全量套件**（Windows 全量 pytest 常超时、Rust 大项目编译慢）
- **Web 后端**：按文件分批 `pytest tests/api/test_api.py -q --tb=short`，每批 1-3 文件，`--timeout=30`
- **Rust 后端**：`cargo test --lib commands::<module>` 每批一模块；集成测试 `cargo test --test <target>` 单独跑；覆盖率 `cargo-llvm-cov`/`cargo-tarpaulin`
- 记录每批 passed/failed/error，汇总到报告
- **🔴 Cargo 测试发现陷阱**：`tests/` 子目录下 `.rs` 不会被自动发现，需在 `Cargo.toml` 显式 `[[test]] name path`；执行前 `cargo test -- --list` 确认全部识别（注意需用 `--`），对比 `find tests/ -name '*.rs'` 查遗漏
- **代码级覆盖率双指标**：`pytest --cov=<src> --cov-report=term`，分批用 `--cov-append`；清单覆盖度与代码覆盖率**互为校验**

### 3. 启动前后端服务

**🔴 启动优先级（按 0.0 建档，不临时现拼）**：构建产物/已安装应用 → `Makefile` 标准入口 → `package.json` scripts → 框架特定命令兜底。

**后端**（FastAPI 示例）：
```bash
NO_PROXY=127.0.0.1,localhost uvicorn app.main:app --host 127.0.0.1 --port 8010 --reload
curl -s --noproxy 127.0.0.1,localhost http://127.0.0.1:8010/health/live  # 200 再继续
```
**前端**：`NO_PROXY=127.0.0.1,localhost npx vite --port 3020 --host 127.0.0.1 --strictPort`
**Tauri**：`cargo tauri dev` 或跑构建产物；前端经 `__TAURI_INTERNALS__.invoke()` 通信；CDP 通道：Android `adb forward tcp:9225 ...`、macOS 桌面需 `--remote-debugging-port`
- **🔴 无设备降级**：无 CDP 时 Rust 命令验证靠步骤 2 cargo test，前端 UI 渲染走 Playwright 访问 dev server（mock invoke 或跳过涉及命令的交互），报告中标注 UI 层 SKIP 原因
- **🔴 mock 响应格式陷阱**：mock `invoke()` 响应须按组件实际期望格式（多数为 `{status:'success',data}`），格式不匹配会误判 FAIL，截图走查确认真实渲染后再定性

**平台陷阱**：系统代理 127.0.0.1:12808 拦截 localhost → 请求加 `--noproxy 127.0.0.1,localhost` 或 `NO_PROXY`；Windows httpx 默认 `trust_env=True` 拦截 → 显式 `trust_env=False`；长驻进程用 `screen`。

### 3a. 冒烟测试（🔴 强制关卡 — 不通过禁止进入 4/5/6）

**目标**：判断"这个版本有没有资格继续测"，失败立即打回，不浪费时间跑完整套件。

**最小检查集**（几分钟内）：
- 服务能启动，健康检查端点返回 200
- 首页/核心入口能打开，无白屏
- 登录（或等价认证入口）能走通
- 1-2 个高风险业务域核心 API 正常返回

**判定**：全过 → 进步骤 4；任一失败 → 🛑 STOP，输出阻断性缺陷报告，结论"先修复冒烟阻断项"。

### 4. API 端点深度验证

#### 4.1 基础验证
每个端点记录：HTTP 状态码 + 响应体大小 + 耗时。优先 curl。

#### 4.2 契约与 Schema 校验（状态码 200 ≠ 契约没变）
- 有 OpenAPI：`schemathesis run <url> --checks all`（最全）> pytest+jsonschema > curl+jq 兜底；必查文档漂移与隐性 schema 变更
- 无 OpenAPI：首轮快照响应结构落盘 `docs/testing/api-schema-snapshot.json`，后续回归 diff

#### 4.3 状态码与错误语义
创建 201（或 200+明确语义）、删除 204/200 且 GET→404、参数错 400/422、未认证 401、权限不足 403（不混用）、不存在 404、业务拒绝 409/422。错误体统一结构。

#### 4.4 限流与配额
突发并发超限 → 429 + Retry-After；窗口重置恢复；低配不受污染。无设计则标安全建议。

#### 4.5 幂等性
重复 POST 不产生重复记录；Idempotency-Key 重放；PUT 中断重发状态一致；删除连发第 2 次 404 或幂等。

#### 4.6 数据落盘验证
关键写操作后回读确认数据真实落盘且字段一致；分页/排序/过滤边界参数优雅处理。

### 5. UI 交互测试（双层）

**API 通过 ≠ UI 通过，必须两层分别验证。**

#### 5.1 API 层验证
逐个调用功能清单后端命令，验证 CRUD + 边界值。

#### 5.2 UI 交互层
逐页导航，每页执行验证矩阵：表单提交、弹窗确认/取消、删除流程、列表搜索筛选分页、Tab/切换、按钮跳转、条件渲染、下拉选择、开关、链接、拖拽、滚动加载、图表交互、键盘事件、复制导入导出、列排序。同时：
- **确定性视觉回归**：核心页面 `toHaveScreenshot()` 像素基线（`tests/__screenshots__/`），回归 diff；UI 改版显式更新基线
- **截图走查**（定性兜底）：无基线页面用 image_understanding 检查
- **可访问性**：axe-core 扫描，critical/serious 入缺陷
- **控制台检查**：error/warning

#### 5.3 UX 体验评估（🔴 控件能用 ≠ 体验好用）
对核心页面按矩阵逐维度：状态可见性/错误体验/防错可撤销/一致性/效率/响应性/可访问性/信息架构/空态引导/移动端适配。输出 `UX 问题清单`（维度|页面|现象|建议|严重度），UX 问题与功能缺陷分开统计。

#### 5.4 数据与业务逻辑正确性（🔴 控件能用/好用 ≠ 结果对）
为功能清单每项定义**预期结果（test oracle）**，实际与预期比对，非"无报错"：
- 列表/下拉数据：无重复、数量与去重后 DB 记录一致
- 计算/聚合：已知数据手工算出预期值比对
- 筛选/搜索：结果集恰等于预期子集
- 日期/范围：禁用日期与业务规则完全一致
- CRUD：提交 + 4.6 回读联动
- 有下游影响的写操作：核对库存/余额/关联记录同步变化
**多层次追踪原则**：核心写操作在 UI 反馈 → HTTP 响应 → 后端逻辑 → 数据库记录 → 下游影响 → 异步任务多层核对，只验证 UI"成功"就 PASS 是最大坑。

### 6. 覆盖度核对（🔴 强制 — 完成后才允许输出报告）

**🔴 CHECKPOINT · 🛑 STOP：MISSING > 0 时必须补测，不可跳过。**

逐项标注：✅ PASS（结果与预期一致）/ ❌ FAIL / ⏭️ SKIP（说明原因）/ ⬜ MISSING（必须补测）。**只验证"无报错"未比对结果的项目不能 PASS，标 SKIP 注明。**

```
总功能点：N；已测试：M；跳过：S；遗漏：M'（须=0）；覆盖率=M/N
```

### 7. 结构化测试报告（→ docs/testing/）

- 覆盖度总表 + 功能清单执行表
- **API 深度验证结论**（契约/Schema、状态码、限流、幂等、落盘）
- **业务逻辑校验清单**（独立节，FAIL 可见预期 vs 实际差异）
- **UX 问题清单**（独立节，分开计数）
- **覆盖率双指标**（清单级 + 代码级）差异解释
- 视觉回归 + 可访问性结论
- **非功能测试结论（引用）**：若执行了配套 `nonfunctional-testing`，引用其结论与报告路径
- **缺陷清单**：标题/环境/前置条件/复现步骤/实际结果/预期结果/严重度/证据/定位/修复建议
- **发布建议**：结合缺陷严重度分布 + 覆盖率 + 核心业务风险 → ✅/⚠️/❌

## 配套专项技能（非功能测试 — 不在本技能内执行）

| 维度 | 承担方 | 内容 |
|------|--------|------|
| 性能/负载 | nonfunctional-testing | k6/Locust 并发、p95/p99、错误率、浸泡、SLO |
| 安全 | nonfunctional-testing | 越权矩阵、依赖漏洞、ZAP、CSRF、secrets |
| 跨浏览器 | nonfunctional-testing | Playwright projects、视口矩阵 |

本技能完成后，若用户要求或报告显示性能边界类缺陷，转 `nonfunctional-testing` 执行，结论回填"非功能测试结论"。

## 陷阱

- 全量 pytest 超时 → 分批；PowerShell 常驻卡死 → 日志重定向+后台
- AIGC 水印 hook 异步改写文件 → 编辑前重读；反复冲突改用一次性 Python 补丁脚本批量替换
- 截图留在项目根 → 报告后清理或入 .gitignore
- 测试脚本幂等 → 创建数据前先清理同名残留

## 验证清单（步骤 6 覆盖度之外的 Checklist）

- [ ] `docs/project-understanding.md` 已建立/复用
- [ ] 冒烟通过才进正式测试
- [ ] commit-id 已比对，变更才枚举
- [ ] 功能清单从代码自动提取 + 交叉校验完成
- [ ] 异常路径含：删除被引用/边界值/弹窗关闭/空态/网络/一致性/文件/性能/404/注入
- [ ] 所有路由可达无白屏；所有交互控件响应；弹窗确认+取消可关
- [ ] API 契约/状态码/限流/幂等/落盘已验证
- [ ] UX 10 维已评估；业务逻辑 oracle 已比对
- [ ] **覆盖度核对完成：MISSING = 0**
- [ ] 报告生成 + 发布建议明确

## 🔴 反例与黑名单

| # | 反模式 | 替代做法 |
|---|--------|---------|
| 1 | 凭记忆手写测试清单 | 从代码自动枚举 |
| 2 | 只测 API 不测 UI | 两层分别验证 |
| 3 | 跑完即收工不做覆盖度核对 | 逐项标 PASS/FAIL/SKIP/MISSING |
| 4 | 弹窗只测确认不测取消 | 独立验证关闭 |
| 5 | 数值框不测边界 0 | 必测 0/负数/空/特殊字符 |
| 6 | 删除被引用只拦截不给路 | 提供强制删除/迁移 |
| 7 | 全量 pytest 一次跑 | 分批 + timeout |
| 8 | 每次全量枚举不查 commit-id | 先比对 id |
| 9 | 只枚举 @click/@submit | 全量 16 类交互 |
| 10 | 异常路径只测 CRUD 边界 | 覆盖网络/一致性/文件/性能 |
| 11 | 只看状态码不看响应结构 | 契约/Schema 校验 |
| 12 | 不测限流幂等 | 双发重复验证 |
| 13 | UI 只验证"能用"不评"好用" | UX 10 维评估 |
| 14 | 响应成功就信了不回读 | 落盘回读 |
| 15 | 只信清单覆盖不看代码覆盖 | pytest-cov 双指标 |
| 16 | 截图判定无像素基线 | toHaveScreenshot 基线 |
| 17 | 控件全测但业务流不测 | 用户旅程+状态机 |
| 18 | 越权只靠前端隐藏 | API 直调隔离抽查 |
| 19 | 只验交互不验业务正确性 | oracle 比对 |
| 20 | 日期选择器只测能否打开 | 禁用日期+边界逻辑 |
| 21 | 枚举不完整仍 100% 覆盖 | 独立计数交叉校验 |
| 22 | 不查 Makefile/产物自己拼命令 | 步骤 0.0 建档+优先级 |
| 23 | 不做冒烟直接全量 | 冒烟 STOP |
| 24 | 边界只测中间值 | 等价类+边界值 |
| 25 | 报告只给通过率不给发布建议 | 发布建议 |

## 实战关键教训

- 列表/下拉出现大量重复选项 → 数量与去重后 DB 记录一致
- 汇总金额算错 → 已知测试数据手工比对
- 筛选结果包含不匹配项 → 结果集应恰好等于预期子集
- 前端显示成功但字段被截断 → 与 4.6 回读联动
- 下单成功但库存没扣 → 下游影响多层核对