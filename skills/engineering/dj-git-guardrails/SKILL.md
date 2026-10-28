---
name: dj-git-guardrails
description: "Git 操作安全护栏：防止危险操作，推前检查，保护 main 分支。 Use when about to git push, merge, rebase, or any destructive git operation. 触发词：git 安全、保护分支、推前检查、guardrails、push 前检查、force push。"
---

# Git Guardrails: Git 安全护栏

叠加到其他技能上。在任何 git 操作前检查。

## 铁律

1. **不在 main 上直接工作** — 永远使用 worktree 或分支
2. **不 push 到 main** — 使用 PR/MR 流程
3. **push 前检查** — 确认没有遗留的 debug 代码、凭证、TODO
4. **rebase 前备份** — 确保你知道自己在做什么

## 推前检查清单

```bash
# 确认没有未追踪的敏感文件
git status --short

# 确认 diff 中没有 debug 代码
git diff HEAD | grep -n "console.log\|print(\|dbg!\|TODO\|FIXME"

# 确认没有大型二进制文件
git diff --stat HEAD | grep -E "\.(png|jpg|pdf|zip|dmg)$"
```

## 失败模式

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| push 被拒（远程有新提交） | 先 `git fetch` 再 rebase 本地分支 | 停下报告冲突面，交用户裁决路线 |
| 发现已被提交的敏感文件 | 从工作树移除 + `.gitignore`，**停止** 不自行改写历史 | 报告，请用户在改史/轮换凭证间决策 |
| 需要覆盖远程（force） | 一律 `--force-with-lease`，绝不裸 `--force` | 共享分支禁止强推；停下确认 |

## 配套

参考文件 `references/block-dangerous-git.sh` 查看危险操作拦截脚本。

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| main 上直接写代码 | worktree 或分支 |
| push 前不看代码 | 过一遍推前清单 |
| force push 到共享分支 | 避免 --force，用 --force-with-lease |
