---
name: dijiang-finish-work
name_cn: 收尾工作
description_cn: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。"
description: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。 Use when the session's work is done and ready for commit, archive, or issue closure. 触发词：收尾、提交、归档、收工、finish work、commit、结束会话。"
disable-model-invocation: true
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '98b087bd-b56f-4463-947d-538b55e7cfe9'
  PropagateID: '98b087bd-b56f-4463-947d-538b55e7cfe9'
  ReservedCode1: '7165379b-57b6-45fc-821e-42211c127833'
  ReservedCode2: '7165379b-57b6-45fc-821e-42211c127833'
---

# 收尾工作

收尾当前 DiJiang session。提交、归档只在这里发生。本 skill 不依赖 CLI，所有操作通过文件读写和 git 命令完成。

## 调用契约

加载本 skill 后，不要总结 skill，也不要询问是否继续。立即开始 finish-work 流程。

## 步骤

### 1. 验证状态和质量

先检查真实项目状态，不依赖对话历史。

```bash
test -f .dijiang/active_task.txt && cat .dijiang/active_task.txt
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
git log --oneline @{u}..HEAD 2>/dev/null || true
```

如果有活跃任务，读取 `.dijiang/tasks/<name>/task.json` 和产物文件。

如果代码或行为发生变化，收尾前运行相关测试、typecheck、lint。检查失败会阻塞 finish-work。

**失败先归因，再决定是否阻塞**：待提交变更可能是工作区遗留成果，测试失败未必由它引入。用 HEAD 干净基线复测区分「本来就坏」与「本次改坏」，结论写入 TDD Contract 的 Exception 字段：

```bash
# 1. 在仓库根建 HEAD 基线 worktree（不碰工作区，比 git stash 安全，无 untracked 丢失风险）
git worktree add .temp/wt-verify-head HEAD --detach
# 2. 复跑失败测试。两个坑：workdir 指向 worktree 内路径的命令必须等 worktree 建好后再执行（分两步跑）；
#    构建文件在子目录（如 src-tauri/Cargo.toml）时必须进入该子目录跑，否则报 could not find Cargo.toml
cd .temp/wt-verify-head/<构建文件所在目录> && <复跑失败测试的命令>
# 3. 用完即清
git worktree remove --force .temp/wt-verify-head
```

- 基线同样失败 → 历史遗留：不阻塞提交，Exception 附基线复测证据；先 `git diff HEAD -- <文件> | grep "<失败相关代码>"` 确认本次 diff 未碰该处，再用 `git log -S "<失败相关代码>" -- <文件>` 追溯引入 commit，向用户标记为待修项
- 基线通过 → 本次改动引入的回归：阻塞 finish-work，先修复再提交

代码或行为变更必须在验证输出中包含 **Code Task TDD Contract** 证据：RED/Repro evidence、GREEN command、Regression scope 和 Exception。纯文档、文本、格式化或无代码变更可将这些字段设为 `n/a`，并写明原因。

### 清洁状态

没有活跃任务、没有变更文件、也没有未 push commit 时，直接报告清洁状态并停止。

### 2. Pre-finish Gate

暂存任何文件前先报告：

```text
任务: <name or none>
分支: <branch>
变更文件: <paths>
验证: <commands => result>
RED/Repro evidence: <command or n/a + reason>
GREEN command: <command> => <result>
Regression scope: <checks> => <result>
Exception: <none or justified gap>
文档同步: <updated / none / skipped; reason=...>
版本决策: <major|minor|patch|none; reason=...>
记忆: <written / skipped; reason=...>
Issue 同步: <closed+checklist 全勾+GET 复核 / none / skipped; reason=...>
提交模式: <yes/no; reason=...>
```

如果缺少验证、代码变更缺少 TDD evidence、变更工作缺少文档同步证据、缺少记忆决策、版本决策不清楚，停止。

### 3. 确认 Git 隔离

```bash
git worktree list 2>/dev/null
git rev-parse --show-toplevel
```

如果当前在主 checkout 且有代码变更，停止。diff 混入无关文件时，只暂存已审查路径。

例外：变更非本 session 产生、而是工作区既有遗留成果，且用户明确要求提交时，不硬停——在 Pre-finish Gate 如实报告主 checkout 状态，把提交拆分与是否 push 作为决策点交用户拍板，确认后再执行。

**仓库尚无任何 commit（首次提交）**：上述例外的特例，`git diff --stat HEAD` 无基线可比（全部 untracked）。处理要点：

- 改动证据不依赖 git diff，直接读文件关键行（`sed -n`/`rg`）展示
- 只提交本 session 改动的文件会造出**残缺首个 commit**（依赖文件未入库，仓库处于无法运行的中间态）——提交模式（整仓首提交 vs 仅本次改动）必须交用户拍板
- 本 session 改动已落在工作区文件中，无改前基线可拆分，只能并入初始 commit，message 中注明
- 首次提交前先建 `.gitignore` 并审计入库范围：
  1. 排除垃圾：`.temp/`（venv/日志）、`__pycache__`/`*.pyc`、`.pytest_cache`、本地 DB/日志
  2. 证书/密钥类文件不盲目排除：先查是否被 Dockerfile/deploy 引用（如 `--ssl-certfile` + `COPY . .`），部署必需的自签名证书保留入库并向用户说明
  3. `git add -A` 后核对暂存清单：`git diff --cached --name-only | rg '\.temp/|__pycache__|\.pyc$'` 应无输出，确认干净再提交

### 4. 判断版本影响

| 决策 | 使用场景 |
|---|---|
| major | 不兼容的公开 API 或行为变更 |
| minor | 向后兼容的功能新增 |
| patch | 向后兼容的 bug 修复 |
| none | 文档、测试、内部 workflow 变更 |

只有存在版本元数据且决策不是 `none` 时，才更新版本文件和 CHANGELOG。

### 5. 同步产物

行为变化时，收尾前更新任务记录、spec、docs 或 changelog。

gate 必须说明文档同步决策和原因。

### 6. 提交已审查范围

提交前先执行 `git diff --stat HEAD` 获取变更事实；commit message 必须基于变更事实总结。

**未识别文件分类（并行开发防护）**：dirty 文件先分两组——

- **AI 本次编辑**：本 session 内 agent 明确改动的文件
- **无法识别**：来源不明的文件（用户手动改的、其他窗口/worktree 的、历史遗留）

无法识别组单列展示，**默认不包含**在提交内，逐一向用户确认 include/exclude。多 worktree 并行开发时，误提交其他窗口的改动是真实风险。

一次性列出完整提交计划（文件清单 + message）等用户确认；用户拒绝后转手动模式，不自动生成第二版计划。

所有 commit message 使用中文编写，遵循 Conventional Commits 格式 `<类型>(<范围>): 中文描述`。

```bash
git add <reviewed paths>
git commit -m "<类型>(<范围>): 中文描述"
```

### 7. 清理 worktree 与分支

提交（及 merge，如有）完成后，清理本次任务使用的 worktree 与已合并分支：

```bash
git worktree list
git worktree remove <path>           # 已合并的 worktree
git worktree remove --force <path>   # 有未跟踪/残留文件时，先确认改动已提交再强制移除
git branch -d <branch>               # 只删除已合并分支
```

- 只清理已合并（或用户确认不再需要）的 worktree；未合并的保留，并向用户报告分支名与 worktree 路径。
- 主 checkout 永不清理。

### 8. 归档任务

如果有活跃任务，将 task.json 的 status 更新为 `archived`，并从 `.dijiang/active_task.txt` 移除指针。

```bash
# 更新 task.json status 为 archived
# 移除 .dijiang/active_task.txt
```

**工单已镜像到 Forgejo 的（task.json 记有 issue 编号）**：归档前必须同步 issue 侧，机制与陷阱以 dj-split「同步到 issue 跟踪器」章节为权威（此处不重复）：

- 任务已完成 → 关闭正规流程：三重证据核实（Forgejo 状态 + 本地提交 + 代码实证 grep，不信推测）→ 结题评论（commit 引用 + 验收逐条对照 + 边界如实声明）→ 同步勾选正文验收 checklist（先 GET 原文 body，仅替换 `- [ ]`→`- [x]` 标记后回写，勿凭记忆重写正文）→ PATCH closed → GET 复核（state + checklist 全勾）→ 本地同步 completed 并补 branch/commit；
- 任务未完成/放弃/挂起 → issue 保持 open，评论说明当前状态、原因与再启动触发条件（挂起工单写明触发条件，防数据空白期提前动工）；
- 两侧状态一致才收口，Gate 的「Issue 同步」字段必须如实报告。

### 9. 记录记忆

调用 `dj-memory` skill 记录本次 session 的发现和经验。

Call the Skill tool with "dj-memory" to store findings and learnings.

### 10. 远程部署交付（如任务含远程/生产部署）

执行时机：第 6 步提交之后、第 8 步归档之前——**部署验证通过才算交付闭环**。完整命令与陷阱见 `references/restricted-network-deploy.md`。

要点：

1. **改前备份**：服务器代码 tar 快照（排除 datas/reports）+ config + .env；旧镜像保留作回滚锚点
2. **构建决策树**：服务器 compose build 拉基础镜像超时（shell curl 通而 daemon 不通）→ 转本地 podman 构建（`--env` 置空代理注入、清华 pypi/debian 源、ForceIPv4）→ `podman save` → scp → `docker load` → re-tag 到 compose 引用的 tag
3. **compose 卡死时手动重建**：`docker inspect` 提全参（env/挂载/网络/healthcheck/compose labels）→ stop+rm → `docker run -d` 复刻；从 symlink 目录跑 compose 会换项目名导致挂起；依赖顺序停启（先 dispatcher 后 server，反向恢复）
4. **部署后验证**：`docker exec <ctr> grep -c <新代码标记>` 逐项确认 + health + 日志；不验证 = 没部署
5. **凭证卫生**：sudo 密码/临时文件双端即用即毁（服务器 /tmp 传输件、本地 .temp、密码编码文件）

## 反模式

| 不要 | 改为这样做 |
|---|---|
| 不要从主 checkout 提交代码变更 | 只从任务 worktree 收尾 |
| 不要盲目 `git add .` | 暂存已审查路径 |
| 不要隐藏验证失败 | 报告命令和失败 |
| 不要写 "fixed bug" 这类模糊记忆 | 写入有 source、scope、已验证的 finding |
| 不要带着无关脏改关闭任务 | 拆分或清理范围 |
| 不要只关本地不关 issue（或反之） | 双向同步，本地为权威源，两侧状态一致才收口 |
| 不要仅凭对话印象就关 issue | 三重证据核实：Forgejo 状态 + 本地提交 + 代码实证 grep |
| 不要留着已合并的 worktree 和分支不管 | 提交/merge 后清理 worktree、删除已合并分支 |
| 不要只改服务器代码文件不重建镜像 | compose 烤入镜像的部署，源码改了不生效；必须 build/load + recreate 容器 |
| 不要 compose 卡死就死等 | 换真实目录/`-p 原项目名`重试，仍卡则 inspect 提参 + docker run 手动复刻（带上 compose labels） |
| 不要部署完不进容器验证 | `docker exec grep` 新代码标记 + health + 日志；不验证的部署不算交付 |