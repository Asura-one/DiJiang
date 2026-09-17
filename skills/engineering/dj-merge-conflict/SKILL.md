---
name: dj-merge-conflict
description: "解决进行中的 git merge/rebase 冲突。Use when you need to resolve an in-progress git merge/rebase conflict."
---

# 解决合并冲突

1. **查看当前状态**。检查 merge/rebase 的历史和冲突文件。

```bash
git status
git log --oneline -10
git diff --name-only --diff-filter=U
```

2. **为每个冲突找到主要来源**。深入理解每处变更为什么发生、原始意图是什么。读取 commit message、PR、原始 issue/ticket。

3. **逐个 hunk 解决**。尽可能保留双方意图。无法兼容时，选择与 merge 目标一致的一方并说明权衡。**不要**发明新行为。总是解决，从不 `--abort`。

4. **发现项目的自动化检查并运行**：通常是 typecheck、tests、format。修复 merge 破坏的任何东西。

## 规则

| 规则 | 说明 |
|------|------|
| 总是 resolve | 从不 `--abort`；abort 丢失双方工作 |
| 保留意图 | 两边作者的原意都应尽量保留 |
| 不发明行为 | 冲突解决不是写新代码 |
| 记录权衡 | 选择一方时说明为什么 |

## 反模式

| 不要 | 改为这样做 |
|------|-----------|
| `git merge --abort` | 逐 hunk 解决，从不放弃 |
| 直接接受一边删除另一边的代码 | 先理解两边意图再决定 |
| 复制粘贴新逻辑解决冲突 | 保留现有行为，最小改动 |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '47c9620d-bf72-449e-a637-f81b11ebdf05'
  PropagateID: '47c9620d-bf72-449e-a637-f81b11ebdf05'
  ReservedCode1: 'e594bf10-df03-4d5c-83d8-f0ddc57c3332'
  ReservedCode2: 'e594bf10-df03-4d5c-83d8-f0ddc57c3332'
