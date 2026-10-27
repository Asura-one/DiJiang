---
name: dj-triage
description: "将 issue 和外部 PR 通过分诊角色状态机流转：分类、验证、必要时拷问、编写 agent-ready briefs。适配本地文件任务存储。 Use when issues arrive—bug reports, feature requests, external PRs—and need classification before dispatch. 触发词：分诊、triage、issue 处理、新 issue、问题分类、工单流转。"
disable-model-invocation: true
---

# Triage 分诊

将本地任务队列中的 issue/任务通过小型状态机分诊。任务存储在 `.dijiang/tasks/` 或用户指定的 issue 目录中。

## 分诊角色

两个**类别**角色：

- `bug`：某处坏了
- `enhancement`：新功能或改进

五个**状态**角色：

- `needs-triage`: 维护者需要评估
- `needs-info`: 等待报告者提供更多信息
- `ready-for-agent`: 完全明确，agent 可直接执行
- `ready-for-human`: 需要人工实现
- `wontfix`: 不会处理

每个任务恰好带一个类别角色和一个状态角色。状态角色冲突时，先标记并询问维护者。

状态流转：未标记任务通常先到 `needs-triage`，然后转 `needs-info`、`ready-for-agent`、`ready-for-human` 或 `wontfix`。`needs-info` 在报告者回复后回到 `needs-triage`。维护者可随时覆盖；异常流转先询问。

## 调用

维护者描述想做什么，解释意图并执行。示例：

- "给我看看需要我关注的东西"
- "看看 task-xxx"
- "把 task-xxx 移到 ready-for-agent"
- "有哪些任务 agent 可以直接做"

## 分诊单个任务

1. **收集上下文**。读取任务完整内容（正文、评论、标签、作者、日期）。解析先前的分诊 notes，避免重复问已解决的问题。在代码库中做两个检查：(a) **冗余**：按领域概念搜索是否已有实现；(b) **先前拒绝**：阅读 `.out-of-scope/`（或 docs/references/ 中的对应知识）看是否有相似请求。

2. **推荐**。给出类别和状态推荐及理由，加上与请求相关的简短代码库摘要。等维护者指示。

3. **验证主张**。进行任何拷问前，验证任务的核心主张。bug 就按报告者的步骤复现；确认后报告。验证通过使 agent brief 更强。

4. **拷问（如需）**。如果请求需要充实，调用 `dj-grill` 和 `dj-domain-modeling`，逐轮拷问成型。

5. **应用结果**：
   - `ready-for-agent`: 写一份 agent brief（任务说明 + 上下文 + 验收标准）
   - `ready-for-human`: 同样结构，但注明为什么不能委派
   - `needs-info`: 贴分诊 notes
   - `wontfix`: 关闭任务，按原因注释：
     - **已实现**: 指出实现位置
     - **拒绝（bug）**: 给出礼貌解释后关闭
     - **拒绝（enhancement）**: 记录拒绝理由后关闭

## 同步到 issue 跟踪器（可选）

若项目在本地任务之外还有真实跟踪器（Forgejo/Gitea/GitHub 等），每个任务的分诊结果要**双向同步**，本地任务为唯一权威源，不得只更新一侧：

- 结果落地（写 agent brief / 贴 needs-info notes）→ 在跟踪器创建或更新对应 issue（标题 + 正文）；
- `wontfix` → 按原因评论后**关闭**对应 issue；
- 状态变化 → 补评论说明，保持两侧一致。

**格式统一（必须）**：issue 一律套用下方固化模板——**模板即统一格式，无需每次查看线上存量 issue**。统一约定：标题 `<前缀>-<编号> [<类型>] [P0|P1|P2|P3] <一句话摘要>`（例：`KUAJI-013 [bug][P1] 删除交易后余额未回滚`；前缀沿用项目既有工单前缀，如 `KUAJI`，新项目取项目名缩写并在首个 issue 确立，之后不变）、正文用固定章节骨架、尾注本地工单路径；**不得自创格式**。

**类型区分（必须）**：每条 issue 必须标类型，与分诊类别角色（`bug` / `enhancement`）一一对应：

| 类型 | 标题标记 | 跟踪器标签 | 正文骨架 |
|---|---|---|---|
| `bug` 缺陷 | `[bug][P?]` | `bug` | 现象 → 根因 → 影响 → 修复建议 → 关联 |
| `feat` 需求/增强 | `[feat][P?]` | `enhancement` | 需求 → 方案建议 → 影响/收益 → 验收标准 → 关联 |

**分类标签**：创建时必须挂类型标签，并按需挂领域分类标签（如 `db`、`ui`、`sync`、`security`，按项目实际）。机制：`GET /repos/<owner>/<repo>/labels` 查名称→ID → 创建 issue 的 body 带 `"labels": [<ID>, ...]`，或创建后 `POST /repos/<owner>/<repo>/issues/<n>/labels` body `{"labels": [<ID>, ...]}`。**只用仓库已有标签**；缺的先建：`POST /repos/<owner>/<repo>/labels` body `{"name","color"}`（约定色：`bug` 红 / `enhancement` 绿 / `task` 灰 / `documentation` 蓝；P 级：P0 `#b60205` / P1 `#d93f0b` / P2 `#fbca04` / P3 `#c5def5`）。

**Issue 模板（bug 缺陷）**：

```markdown
## 现象
<用户视角可观察的行为，附实证：数据 / 日志 / 复现步骤>

## 根因
<调用链（文件:行号）+ 逐条根因>

## 影响
<波及面 + 被阻塞的关联工单>

## 修复建议
<可选方案 A/B + 推荐 + 回归验证要求>

## 关联
<关联工单编号 / 本地任务路径>

> 本地工单：`<path>`
```

**Issue 模板（feat 需求）**：

```markdown
## 需求
<用户想要什么、为什么；来源场景>

## 方案建议
<建议实现要点与取舍>

## 影响 / 收益
<受益场景 + 关联工单>

## 验收标准
- [ ] <可验证条件>

## 关联
<关联工单编号 / 本地任务路径>

> 本地工单：`<path>`
```

**状态评论模板**（修复 / 复验后先评论、再 PATCH 关闭）——关闭前**三重证据核实**（Forgejo 状态 + 本地提交 + 代码实证 grep，不信推测），结题评论含验收对照与边界如实声明：

```markdown
## 已修复（方案 <A/B>，commit <hash>，验证 <方式> 通过）
<改动摘要>
<验收/复现逐条对照>
<回归结果>
<边界如实声明：未覆盖范围/已知限制>
<结论 + 解除阻塞的关联项>
```

关闭正规流程：结题评论 → `PATCH` 置 closed → **GET 复核** state=closed → 本地工单状态同步；`wontfix` 类关闭同样先评论原因再关，两侧状态一致才收口。

Forgejo/Gitea API：基址 `<base>/api/v1`，头 `Authorization: token <TOKEN>`（token 从项目配置/密钥读取，**切勿回显到输出、URL 或文件**）。创建 `POST /repos/<owner>/<repo>/issues` body `{"title","body"}`；评论 `POST /repos/<owner>/<repo>/issues/<n>/comments` body `{"body"}`；关闭 `PATCH /repos/<owner>/<repo>/issues/<n>` body `{"state":"closed"}`。

**私有仓库假 404 陷阱**：直接 `GET /repos/<owner>/<repo>` 对私有仓库可能 404 假象、`/users/<owner>/repos` 只列公开仓库——存在性存疑时用 `GET /repos/search?private=true&q=<关键词>` 复核；辅以 SSH `git ls-remote` + `Forgejo: Cannot find repository` 报错判定网关身份。

Windows/企业环境：用 `curl.exe`，JSON 先写**临时文件**再 `--data-binary @file`（PowerShell 内联引号会破坏 JSON）；以 UTF-8 写文件避免中文乱码；关闭/状态变更后 **GET 复核**（PATCH 响应可能不含 state）；回环被企业代理拦截时加 `--noproxy "*"`；**`$env:` 变量不跨工具调用保留**（每次调用全新会话）——token 赋值与使用必须在同一条命令内完成。

## Needs-info 模板

```markdown
## 分诊 Notes

**我们已确认：**

- 要点 1
- 要点 2

**我们仍需要你提供（@报告者）：**

- 问题 1
- 问题 2
```

## 快速状态覆盖

如果维护者说"把 task-xxx 移到 ready-for-agent"，信任并直接应用角色。先确认要做什么（角色变化、评论、关闭），然后执行。跳过拷问。若无拷问就直接到 `ready-for-agent`，询问是否要写 agent brief。

## 参考

- `docs/references/` — 跨技能参考
- 任务存储在 `.dijiang/tasks/<id>/`，状态在 `task.json` 的 `status` 字段

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '7ef65ee7-0614-4352-bfea-54cc665e7b0c'
  PropagateID: '7ef65ee7-0614-4352-bfea-54cc665e7b0c'
  ReservedCode1: '0382de80-47e3-4254-9d3e-0d7f975ae386'
  ReservedCode2: '0382de80-47e3-4254-9d3e-0d7f975ae386'
