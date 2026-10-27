# dj-triage Issue 跟踪器同步细节

本文件是「同步到 issue 跟踪器」的完整机制与模板。SKILL.md 保留双同步硬规则与格式判定；执行创建/更新/关闭 issue 或写模板时读本节。

## 格式统一

标题 `<前缀>-<编号> [<类型>] [P0|P1|P2|P3] <一句话摘要>`（例：`KUAJI-013 [bug][P1] 删除交易后余额未回滚`；前缀沿用项目既有工单前缀，新项目取项目名缩写并在首个 issue 确立，之后不变）、正文用固定章节骨架、尾注本地工单路径；**不得自创格式**。**模板即统一格式，无需每次查看线上存量 issue。**

每条 issue 必须标类型，与分诊类别角色（`bug` / `enhancement`）一一对应：

| 类型 | 标题标记 | 跟踪器标签 | 正文骨架 |
|---|---|---|---|
| `bug` 缺陷 | `[bug][P?]` | `bug` | 现象 → 根因 → 影响 → 修复建议 → 关联 |
| `feat` 需求/增强 | `[feat][P?]` | `enhancement` | 需求 → 方案建议 → 影响/收益 → 验收标准 → 关联 |

## Issue 模板（bug 缺陷）

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

## Issue 模板（feat 需求）

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

## 状态/结题评论模板

（修复 / 复验后先评论、再 PATCH 关闭）关闭前**三重证据核实**（Forgejo 状态 + 本地提交 + 代码实证 grep，不信推测），结题评论含验收对照与边界如实声明：

```markdown
## 已修复（方案 <A/B>，commit <hash>，验证 <方式> 通过）
<改动摘要>
<验收/复现逐条对照>
<回归结果>
<边界如实声明：未覆盖范围/已知限制>
<结论 + 解除阻塞的关联项>
```

关闭正规流程：结题评论 → `PATCH` 置 closed → **GET 复核** state=closed → 本地工单状态同步；`wontfix` 类关闭同样先评论原因再关，两侧状态一致才收口。

## 分类标签机制

创建时必须挂类型标签，并按需挂领域分类标签（如 `db`、`ui`、`sync`、`security`，按项目实际）。机制：`GET /repos/<owner>/<repo>/labels` 查名称→ID → 创建 issue 的 body 带 `"labels": [<ID>, ...]`，或创建后 `POST /repos/<owner>/<repo>/issues/<n>/labels` body `{"labels": [<ID>, ...]}`。**只用仓库已有标签**；缺的先建：`POST /repos/<owner>/<repo>/labels` body `{"name","color"}`（约定色：`bug` 红 / `enhancement` 绿 / `task` 灰 / `documentation` 蓝；P 级：P0 `#b60205` / P1 `#d93f0b` / P2 `#fbca04` / P3 `#c5def5`）。

## Forgejo/Gitea API 要点

基址 `<base>/api/v1`，头 `Authorization: token <TOKEN>`（token 从项目配置/密钥读取，**切勿回显到输出、URL 或文件**）。创建 `POST /repos/<owner>/<repo>/issues` body `{"title","body"}`；评论 `POST /repos/<owner>/<repo>/issues/<n>/comments` body `{"body"}`；关闭 `PATCH /repos/<owner>/<repo>/issues/<n>` body `{"state":"closed"}`。

**私有仓库假 404 陷阱**：直接 `GET /repos/<owner>/<repo>` 对私有仓库可能 404 假象、`/users/<owner>/repos` 只列公开仓库——存在性存疑时用 `GET /repos/search?private=true&q=<关键词>` 复核；辅以 SSH `git ls-remote` + `Forgejo: Cannot find repository` 报错判定网关身份。

**Windows/企业环境**：用 `curl.exe`，JSON 先写**临时文件**再 `--data-binary @file`（PowerShell 内联引号会破坏 JSON）；以 UTF-8 写文件避免中文乱码；关闭/状态变更后 **GET 复核**（PATCH 响应可能不含 state）；回环被企业代理拦截时加 `--noproxy "*"`；**`$env:` 变量不跨工具调用保留**（每次调用全新会话）——token 赋值与使用必须在同一条命令内完成。