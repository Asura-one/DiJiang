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

收尾当前 DiJiang session。提交、归档只在这里发生，不依赖 CLI，全部通过文件读写和 git 命令完成。

深度细节：失败归因基线复测、主 checkout 例外、首次提交特例、issue 同步流程、反模式全表 → `references/finish-gates.md`；远程部署命令与陷阱 → `references/restricted-network-deploy.md`。

## 调用契约

加载本 skill 后，不要总结 skill，也不要询问是否继续。立即开始 finish-work 流程。

## 步骤

### 1. 验证状态和质量

先检查真实项目状态，不依赖对话历史：`cat .dijiang/active_task.txt`（有则读 `.dijiang/tasks/<name>/task.json` 和产物）、`git status --short --branch`、`git diff --stat HEAD`、`git diff --name-only HEAD`、`git log --oneline @{u}..HEAD`。

代码或行为变化时，收尾前运行相关测试、typecheck、lint，失败会阻塞 finish-work。**失败先归因**：用 HEAD 干净基线复测区分「本来就坏」与「本次改坏」（见 references/finish-gates.md），结论写入 TDD Contract 的 Exception 字段。变更必须在验证输出中含 **Code Task TDD Contract** 证据：RED/Repro evidence、GREEN command、Regression scope、Exception。纯文档变更可设 `n/a` 并写明原因。

### 清洁状态

无活跃任务、无变更文件、无未 push commit → 直接报告清洁状态并停止。

### 2. Pre-finish Gate

暂存前先报告：

```text
任务: <name or none>
分支: <branch>
变更文件: <paths>
验证: <commands => result>
RED/Repro: <command or n/a + reason>
GREEN command: <command> => <result>
Regression scope: <checks> => <result>
Exception: <none or justified gap>
文档同步: <updated/none/skipped; reason>
版本决策: <major|minor|patch|none; reason>
记忆: <written/skipped; reason>
Issue 同步: <closed+全勾+复核/none/skipped; reason>
提交模式: <yes/no; reason>
```

缺验证、代码变更缺 TDD evidence、缺文档同步证据、缺记忆或版本决策不清 → 停止。

### 3. 确认 Git 隔离

执行 `git worktree list` + `git rev-parse --show-toplevel` 确认隔离。如果当前在主 checkout 且有代码变更，停止。diff 混入无关文件时，只暂存已审查路径。

两个例外（细节见 references/finish-gates.md）：① 主 checkout 遗留成果且用户明确要求提交 → 提交拆分与是否 push 交用户拍板；② 仓库尚无 commit（首次提交）→ 提交模式交用户拍板，先建 `.gitignore` 审计入库范围。

### 4. 判断版本影响

major=不兼容的公开 API/行为变更；minor=向后兼容新增；patch=向后兼容修复；none=文档/测试/内部变更。只有存在版本元数据且决策非 `none` 时，才更新版本文件和 CHANGELOG。

### 5. 同步产物

行为变化时，收尾前更新任务记录、spec、docs 或 changelog；gate 说明同步决策和原因。

### 6. 提交已审查范围

提交前先执行 `git diff --stat HEAD` 获取变更事实；commit message 必须基于变更事实总结。

**未识别文件分类（并行开发防护）**：dirty 文件分两组——**AI 本次编辑**（本 session 明确改动的）与**无法识别**（用户手动改的、其他窗口/worktree 的、历史遗留）。无法识别组单列展示、默认不入库，逐一向用户确认。

一次性列出完整提交计划（文件清单 + message）等用户确认；用户拒绝后转手动模式，不自动生成第二版计划。commit message 用中文，Conventional Commits 格式：`git add <reviewed paths> && git commit -m "<类型>(<范围>): 中文描述"`。

### 7. 清理 worktree 与分支

提交及 merge 完成后清理任务 worktree 与已合并分支。只清理已合并（或用户确认放弃）的；未合并的保留并报告分支名与路径。主 checkout 永不清理。

### 8. 归档任务

如果有活跃任务，将 task.json 的 status 更新为 `archived`，并从 `.dijiang/active_task.txt` 移除指针。

**工单已镜像到 Forgejo（task.json 记 issue 编号）**：归档前必须同步 issue 侧，机制以 dj-split「同步到 issue 跟踪器」为权威，判定规则见 references/finish-gates.md（已完成→三重证据+结题评论+checklist 勾选+复核；挂起→保持 open 写明触发条件）。两侧一致才收口。

### 9. 记录记忆

调用 `dj-memory` skill 记录本次 session 的发现和经验。

### 10. 远程部署交付（如任务含远程/生产部署）

执行时机：第 6 步提交之后、第 8 步归档之前——**部署验证通过才算交付闭环**。完整命令与陷阱见 `references/restricted-network-deploy.md`。要点：改前备份（tar 快照+config+.env、旧镜像留回滚锚点）、服务器 build 超时→本地 podman 构建+save/load、compose 卡死手动重建、部署后必 `docker exec grep` 验证新代码标记、凭证双端即用即毁。

## 反模式

核心：不从主 checkout 提交、不盲目 `git add .`、不隐藏验证失败、不写模糊记忆、不带脏改关任务、issue 双向同步、关 issue 须三重证据、已合并 worktree 及时清理、部署必验容器内代码。全表见 `references/finish-gates.md`。