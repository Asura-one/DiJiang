---
name: dijiang-finish-work
name_cn: 收尾工作
description_cn: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。"
description: "收尾当前 session：质量验证、版本决策、提交、归档任务。所有 git 提交和任务归档只在这里发生。 触发词：收尾、提交、归档、收工、finish work、commit、结束会话。
**下一步**：收口归档后，建议下一步调用 `dj-evolve` 复盘本 session 的可复用经验（有升格候选时按 SCOPE 升格判据筛选）。"
disable-model-invocation: true
---

# 收尾工作

收尾当前 DiJiang session。提交、归档只在这里发生，不依赖 CLI，全部通过文件读写和 git 命令完成。

深度细节（各步检查/模板/判定原文、反模式全表）→ `references/finish-gates.md`；远程部署 → `references/restricted-network-deploy.md`。

## 调用契约

加载本 skill 后，不要总结 skill，也不要询问是否继续。立即开始 finish-work 流程。

## 步骤

### 1. 验证状态和质量

先检查真实项目状态，不依赖对话历史（命令清单 → `references/finish-gates.md`）

代码或行为变化时，收尾前跑相关测试、typecheck、lint，失败阻塞 finish-work。**失败先归因**（HEAD 基线复测 → `references/finish-gates.md`）。变更须含 **TDD Contract** 证据：RED/Repro、GREEN、Regression scope、Exception；纯文档可 `n/a` 写明原因

### 清洁状态

无活跃任务、无变更文件、无未 push commit → 直接报告清洁状态并停止。

### 2. Pre-finish Gate

暂存前报告下列各项（模板与逐项要求 → `references/finish-gates.md`）：任务、分支、变更文件、验证、RED/Repro、GREEN、Regression scope、Exception、文档同步、版本决策、记忆、Issue 同步、Live 同步、提交模式

缺验证、代码变更缺 TDD evidence、缺文档同步证据、缺记忆或版本决策不清 → 停止。

### 3. 确认 Git 隔离

执行 `git worktree list` + `git rev-parse --show-toplevel` 确认隔离。如果当前在主 checkout 且有代码变更，停止。diff 混入无关文件时，只暂存已审查路径。

两个例外（主 checkout 遗留成果、仓库尚无 commit 首次提交）→ `references/finish-gates.md`

### 4. 判断版本影响

major=不兼容公开 API/行为变更；minor=兼容新增；patch=兼容修复；none=文档/测试/内部。有版本元数据且非 `none` 才更新版本文件和 CHANGELOG。

### 5. 同步产物

行为变化时，收尾前更新任务记录、spec、docs 或 changelog；gate 说明同步决策和原因。

### 6. 提交已审查范围

提交前先执行 `git diff --stat HEAD` 获取变更事实；commit message 必须基于变更事实总结。

**未识别文件分类（并行开发防护）**：无法识别的 dirty 文件单列展示、默认不入库、逐一确认 → `references/finish-gates.md`

一次性列出完整提交计划（文件清单 + message）等用户确认；拒绝后转手动模式，不生成第二版；commit message 中文、Conventional Commits 格式

**批量 add 前看 `git status --short` 首列**：已 staged 删除（`D`）不进 add 列表（pathspec 落空整条 fatal）；提交后回读 `N files changed` 比对 → `references/finish-gates.md`

### 6.5 放行申请（合并 / 推送 / live 同步 / 部署前）

每个要用户点头的动作写三段：Summary / Evidence（Before-After 成对）/ Merge Danger（门 + 爆炸半径）→ `references/merge-request.md`；写不完整 = 没准备好放行，申请不等于授权。

### 7. 清理 worktree 与分支

提交及 merge 完成后清理任务 worktree 与已合并分支。只清理已合并（或用户确认放弃）的；未合并的保留并报告分支名与路径。主 checkout 永不清理。

### 8. 归档任务

如果有活跃任务，将 task.json 的 status 更新为 `archived`，并从 `.dijiang/active_task.txt` 移除指针。

**工单已镜像 Forgejo 时**：归档前必须同步 issue 侧、两侧一致才收口（判定规则 → `references/finish-gates.md`）

### 9. 记录记忆

调用 `dj-memory` skill 记录本次 session 的发现和经验。

### 10. 远程部署交付（如任务含远程/生产部署）

执行时机：第 6 步提交后、第 8 步归档前——**部署验证通过才算交付闭环**（备份/回退/容器内验证/凭证要点 → `references/restricted-network-deploy.md`）

### 11. Live 副本同步（技能增删改批，合并 main 后执行）

repo 是权威源，live 目录（`~/.config/TeleAgent/users/<id>/skills/` dj 系）是投放快照——**只单向 repo→live，永不反向**。命令/三验收/跳过条件 → `references/live-sync-checklist.md`；第三方技能零触碰。

## 反模式

核心：不从主 checkout 提交、不盲目 `git add .`、不隐藏验证失败、不写模糊记忆、不带脏改关任务、issue 双向同步、关 issue 须三重证据、已合并 worktree 及时清理、部署必验容器内代码。全表见 `references/finish-gates.md`。
