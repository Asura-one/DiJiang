---
name: dj-split
description: "将 PRD 文档拆分为独立可执行的 task。衔接 dj-output 的输出，产出可通过 Call the Skill tool with \"dj-dispatch\" 路由的任务列表。 Use after a PRD is written, when you need to break the work into independently implementable tasks. 触发词：拆分任务、分任务、切成 task、work breakdown、分批做。"
disable-model-invocation: true
---

# Split: PRD → 可执行任务

将 PRD 拆分为独立可执行的任务。每个任务应可独立实现、独立验证、独立交付。

## 输入

- `.dijiang/prd/` 下的 PRD 文档
- 或用户提供的 PRD / issue / 功能描述

## 拆分原则

- **独立可发布** — 每个任务完成时系统仍然可用（不破坏已有功能）
- **独立可验证** — 每个任务有自己的验收标准
- **合理的颗粒度** — 不粗到一个任务做一周，不细到一个任务只改一行
- **前后不依赖** — B 不因 A 未完成而无法开始（除非明确有依赖的，标记为 blocking）

## 工作流

### 1. 分析 PRD

从 PRD 中提取：
- User Stories 或功能点列表
- 各功能的依赖关系
- 数据模型变化（如果需要）
- 接口变化（如果需要）

### 2. 生成任务列表

按优先级和依赖排序，输出任务列表。每个任务格式：

```markdown
## Task: <任务名>

**目标**：一句话说清楚这个任务做什么

**PRD 引用**：<PRD 中的对应 User Story 或章节>

**验收标准**：
- [ ] <可验证的条件 1>
- [ ] <可验证的条件 2>

**建议 skill**：dj-implement / dj-tdd

**依赖**：<无 / 依赖 Task X>
```

### 3. 建议执行顺序

列出推荐的开发顺序（哪些先做、哪些可以并行）：

```text
Phase 1（基础能力）：Task A, Task B（可并行）
Phase 2（核心功能）：Task C（依赖 A+B）
Phase 3（增强功能）：Task D, Task E（可并行，依赖 C）
```

### 4. 输出

- 任务列表用于后续 Call the Skill tool with "dj-dispatch"；不要用 Call the Skill tool with "dijiang-start" 进入实现状态。
- 保存到 `.dijiang/tasks/` 或输出到对话中

## 同步到 issue 跟踪器（可选）

若项目在本地任务之外还有真实跟踪器（Forgejo/Gitea/GitHub 等），生成任务列表后可将每个任务**镜像**为一条 issue，本地任务为唯一权威源，不得只更新一侧：

- 每个任务 → 在跟踪器创建一条 issue（标题 + 正文：目标 / 验收标准 / 依赖），并在本地记录返回的 issue 编号；
- 任务状态变化 / 完成 → 补评论说明并关闭对应 issue；
- 依赖关系：优先用跟踪器原生 blocking 关系，否则在正文标注「Blocked by」。

**格式统一（必须）**：issue 一律套用下方固化模板——**模板即统一格式，无需每次查看线上存量 issue**。统一约定：标题 `<前缀>-<编号> [<类型>] [P0|P1|P2|P3] <一句话摘要>`（例：`KUAJI-015 [task][P1] 支持账单导入`；前缀沿用项目既有工单前缀，如 `KUAJI`，新项目取项目名缩写并在首个 issue 确立，之后不变）、正文用固定章节骨架、尾注本地任务路径；**不得自创格式**。

**类型区分（必须）**：任务类 issue 标题用 `[task][P?]`；任务源于缺陷修复 → 另挂 `bug` 标签，源于需求实现 → 另挂 `enhancement` 标签。

**分类标签**：创建时必须挂类型标签（`task`），并按需挂领域分类标签（如 `db`、`ui`、`sync`，按项目实际）。机制：`GET /repos/<owner>/<repo>/labels` 查名称→ID → 创建 issue 的 body 带 `"labels": [<ID>, ...]`，或创建后 `POST /repos/<owner>/<repo>/issues/<n>/labels` body `{"labels": [<ID>, ...]}`。**只用仓库已有标签**；缺的先建：`POST /repos/<owner>/<repo>/labels` body `{"name","color"}`。P 级标签约定色：P0 `#b60205` / P1 `#d93f0b` / P2 `#fbca04` / P3 `#c5def5`（沿用同主人既有仓库配色）。

**Issue 模板**（`[task]` 任务类）：

```markdown
## 目标
<一句话：任务完成后系统可演示 / 可验证什么>

## 验收标准
- [ ] <可验证条件 1>
- [ ] <可验证条件 2>

## 依赖
<Blocked by：`<前缀>-<编号>`；无依赖则写「无（可立即开始）」>

> 本地任务：`<path>`
```

**结题评论模板**（任务完成 / 状态变化时评论；完成后 PATCH 关闭）——验收逐条对照 + 边界如实声明，不夸大：

```markdown
## 已完成（commit <hash>，验证 <方式> 通过）
<交付摘要>
<验收标准逐条对照：标准 → 达成情况/证据>
<回归结果>
<能力边界如实声明：未覆盖范围/已知限制>
```

**关闭正规流程（必须）**：先核实再关闭、先评论再关——①**三重证据核实**（Forgejo issue 状态 + 本地提交记录 + 代码实证 grep，**不信推测**，防误关未完成项——「看起来完成了」不等于完成）；②结题评论（模板如上）→ **同步勾选正文验收 checklist**（`PATCH` body 逐条 `- [ ]`→`- [x]`，勾选依据 = 结题评论的证据链，缺证据的项不得勾）→ `PATCH` 置 closed → **GET 复核** state=closed 且 checklist 全勾（2026-10-22 实战教训：只发结题评论不勾 body，验收状态在 UI 上仍显示未完成）；③本地 task.json 同步 completed 并补 branch/commit/issue 编号，两侧状态一致才收口。

Forgejo/Gitea API：基址 `<base>/api/v1`，头 `Authorization: token <TOKEN>`（token 从项目配置/密钥读取，**切勿回显到输出、URL 或文件**）。创建 `POST /repos/<owner>/<repo>/issues` body `{"title","body"}`；评论 `POST /repos/<owner>/<repo>/issues/<n>/comments` body `{"body"}`；关闭 `PATCH /repos/<owner>/<repo>/issues/<n>` body `{"state":"closed"}`。

**私有仓库假 404 陷阱**（pentest-core 2026-10-22 实战）：直接 `GET /repos/<owner>/<repo>` 对私有仓库可能 404 假象、`/users/<owner>/repos` 只列公开仓库——存在性存疑时用 `GET /repos/search?private=true&q=<关键词>` 复核；辅以 SSH `git ls-remote` + `Forgejo: Cannot find repository` 报错判定网关身份，不要因单次 404 就判定仓库不存在。

Windows/企业环境：用 `curl.exe`，JSON 先写**临时文件**再 `--data-binary @file`（PowerShell 内联引号会破坏 JSON）；以 UTF-8 写文件避免中文乱码；关闭/状态变更后 **GET 复核**（PATCH 响应可能不含 state）；回环被企业代理拦截时加 `--noproxy "*"`；**`$env:` 变量不跨工具调用保留**（每次调用全新会话）——token 赋值与使用必须在同一条命令内完成。

## 边界

- 不实现任务——那是 `dj-implement` / `dj-tdd` 的事
- 不估计工时——只做功能拆解
- 不修改 PRD——如果发现 PRD 有遗漏，报告而不是改

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 任务颗粒度太大（一整天） | 拆分到半天可交付 |
| 任务互相依赖无法独立 | 重新划分边界使其独立 |
| 跳过 PRD 直接拆分 | 先确认需求文档化再拆 |
| 每个任务都给 dj-tdd | 简单的给 dj-implement，复杂行为给 dj-tdd |
