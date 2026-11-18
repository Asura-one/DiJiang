# dijiang-finish-work 质量门细节

本文件承载收尾流程的深度细节。SKILL.md 保留步骤骨架与 Hard Rules；执行对应步骤遇到细判场景时读本节。

## 1. 测试失败先归因——HEAD 干净基线复测

待提交变更可能是工作区遗留成果，测试失败未必由它引入。用 HEAD 基线复测区分「本来就坏」与「本次改坏」，结论写入 TDD Contract 的 Exception 字段：

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

## 2. Git 隔离例外——主 checkout 遗留成果

当前在主 checkout 且变更非本 session 产生、而是工作区既有遗留成果，且用户明确要求提交时不硬停：在 Pre-finish Gate 如实报告主 checkout 状态，把提交拆分与是否 push 作为决策点交用户拍板，确认后再执行。

## 3. 首次提交特例（仓库尚无任何 commit）

`git diff --stat HEAD` 无基线可比（全部 untracked）。处理要点：

- 改动证据不依赖 git diff，直接读文件关键行（`sed -n`/`rg`）展示
- 只提交本 session 改动的文件会造出**残缺首个 commit**（依赖文件未入库，仓库处于无法运行的中间态）——提交模式（整仓首提交 vs 仅本次改动）必须交用户拍板
- 本 session 改动已落在工作区文件中，无改前基线可拆分，只能并入初始 commit，message 中注明
- 首次提交前先建 `.gitignore` 并审计入库范围：
  1. 排除垃圾：`.temp/`（venv/日志）、`__pycache__`/`*.pyc`、`.pytest_cache`、本地 DB/日志
  2. 证书/密钥类文件不盲目排除：先查是否被 Dockerfile/deploy 引用（如 `--ssl-certfile` + `COPY . .`），部署必需的自签名证书保留入库并向用户说明
  3. `git add -A` 后核对暂存清单：`git diff --cached --name-only | rg '\.temp/|__pycache__|\.pyc$'` 应无输出，确认干净再提交

## 4. Forgejo issue 同步（工单已镜像时）

机制与陷阱以 dj-split「同步到 issue 跟踪器」章节为权威，此处只列判定：

- 任务已完成 → 关闭正规流程：三重证据核实（Forgejo 状态 + 本地提交 + 代码实证 grep，不信推测）→ 结题评论（commit 引用 + 验收逐条对照 + 边界如实声明）→ 同步勾选正文验收 checklist（先 GET 原文 body，仅替换 `- [ ]`→`- [x]` 标记后回写，勿凭记忆重写正文）→ PATCH closed → GET 复核（state + checklist 全勾）→ 本地同步 completed 并补 branch/commit
- 任务未完成/放弃/挂起 → issue 保持 open，评论说明当前状态、原因与再启动触发条件（挂起工单写明触发条件，防数据空白期提前动工）
- 两侧状态一致才收口，Gate 的「Issue 同步」字段必须如实报告

### API 认证兕底（无 token 时不新建）

（perimeter 2026-11-05 实战，8 条 issue 批量关闭全程验证）项目没配 API token 时不必新建——Forgejo/Gitea `/api/v1` 全端点接受 HTTP Basic Auth，凭证直接取 git credential store（`credential.helper=store` → `~/.git-credentials`）中该 Forgejo 主机的条目，git push 用过的账号密码即可调 API：

```powershell
$cred=(Get-Content "$env:USERPROFILE\.git-credentials" | Where-Object {$_ -match '<forgejo-host>'} | Select-Object -First 1)
$m=[regex]::Match($cred,'^http://([^:]+):([^@]+)@')  # 凭证文件可能多条，按主机过滤取第一条
$b64=[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("$($m.Groups[1].Value):$($m.Groups[2].Value)"))
$headers=@{Authorization="Basic $b64"}               # 密码不回显、不落盘
```

### 批量关闭实操（Windows/PowerShell）

- 多条 issue 的 评论→勾选正文→closed→GET 复核 放在**同一个 PowerShell 循环**内完成（变量不跨工具调用保留，凭证读取与使用必须同一条命令）
- JSON body 用 `Invoke-RestMethod` + `@{body=...}|ConvertTo-Json` 自动转义，可替代 curl.exe+临时文件（实测评论/正文回写/关闭全通）
- GET 复核用正则计数断言：`- \[ \]` 计数为 0 且 `- \[x\]` 数与验收项数一致
- 插值坑：`"$n:"` 直接 ParserError（冒号被当变量名分隔符），改用 `("issue {0}" -f $n)` 或 `${n}:`
- 验收措辞与实现有出入但意图达成（如 issue 写 `02-seed.sql`、实现为 `02-seed.sh`）→ 该项可勾选，但结题评论中如实注记差异，不得悄悄改写验收措辞

## 5. 批量 `git add` 的 pathspec 陷阱（列表含已暂存删除文件）

`git add -- <path1> <path2> ...` 是**原子失败**：只要有一个 pathspec 匹配不到工作区文件，整条命令 `fatal: pathspec '<p>' did not match any files` 中止，**其余路径一个都不会进 index**。若此时 index 里已有先前暂存的删除（`git status --short` 首列 `D`，如 `D  console/web/pnpm-workspace.yaml`），紧接着的 `git commit` 会把这个孤立删除单独提交成一笔残缺 commit，看起来「提交成功」但实际只含 1 个文件。

成因：已删除文件在工作区已不存在，把它当 pathspec 传给 `git add` 必然匹配失败；而它的删除早已在 index 中，根本不需要再 add。

处理：

```bash
# 1. 暂存前先看首列：D/ R 等已 staged 项不要放进 add 列表
git status --short
# 2. 只 add 工作区仍存在的已修改文件（M/??）
git add -- <仅 M 与 ?? 的路径>
# 3. 核对暂存清单再提交，确认文件数与计划一致
git diff --cached --stat
# 4. 若已误提交成孤立 commit：软撤销（保留改动与暂存），重新 add 后提交
git reset --soft HEAD~1
```

- 需要把「删除 + 修改」放进同一笔提交时，删除项无需 add（已在 index），或改用 `git add -A -- <目录>` 让 git 自行处理删除
- 提交后必须回读 `git log --oneline -1` 的 `N files changed` 与计划文件数比对，不一致即视为提交失败

## 6. 反模式全表

| 不要 | 改为这样做 |
|---|---|
| 不要从主 checkout 提交代码变更 | 只从任务 worktree 收尾 |
| 不要盲目 `git add .` | 暂存已审查路径 |
| 不要把已 staged 的删除文件放进 `git add` 列表 | 先看 `git status --short` 首列，只 add 工作区仍存在的 M/??；误提交用 `git reset --soft HEAD~1` 重来 |
| 不要隐藏验证失败 | 报告命令和失败 |
| 不要写 "fixed bug" 这类模糊记忆 | 写入有 source、scope、已验证的 finding |
| 不要带着无关脏改关闭任务 | 拆分或清理范围 |
| 不要只关本地不关 issue（或反之） | 双向同步，本地为权威源，两侧状态一致才收口 |
| 不要仅凭对话印象就关 issue | 三重证据核实：Forgejo 状态 + 本地提交 + 代码实证 grep |
| 不要留着已合并的 worktree 和分支不管 | 提交/merge 后清理 worktree、删除已合并分支 |
| 不要只改服务器代码文件不重建镜像 | compose 烤入镜像的部署，源码改了不生效；必须 build/load + recreate 容器 |
| 不要 compose 卡死就死等 | 换真实目录/`-p 原项目名`重试，仍卡则 inspect 提参 + docker run 手动复刻（带上 compose labels） |
| 不要部署完不进容器验证 | `docker exec grep` 新代码标记 + health + 日志；不验证的部署不算交付 |
| 不要因项目没配 API token 就卡住或新建 token | Forgejo `/api/v1` 接受 Basic Auth，凭证取 `~/.git-credentials` 该主机条目 |