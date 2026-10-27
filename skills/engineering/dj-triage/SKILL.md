---
name: dj-triage
description: "将 issue 和外部 PR 通过分诊角色状态机流转：分类、验证、必要时拷问、编写 agent-ready briefs。适配本地文件任务存储。 Use when issues arrive—bug reports, feature requests, external PRs—and need classification before dispatch. 触发词：分诊、triage、issue 处理、新 issue、问题分类、工单流转。"
disable-model-invocation: true
---

# Triage 分诊

将本地任务队列中的 issue/任务通过小型状态机分诊。任务存储在 `.dijiang/tasks/` 或用户指定的 issue 目录中。

Issue 跟踪器同步的完整机制、固化模板（bug/feat/结题评论）、标签机制、Forgejo API 要点与平台陷阱 → `references/issue-sync.md`。

## 分诊角色

两个**类别**角色：`bug`（某处坏了）、`enhancement`（新功能或改进）。

五个**状态**角色：`needs-triage`（待评估）、`needs-info`（等报告者补信息）、`ready-for-agent`（完全明确 agent 可执行）、`ready-for-human`（需人工）、`wontfix`（不处理）。

每个任务恰带一个类别 + 一个状态角色。冲突时先标记问维护者。流转：未标记任务先到 `needs-triage`，再转其余状态；`needs-info` 报告者回复后回 `needs-triage`。维护者可随时覆盖；异常流转先问。

## 调用

维护者描述意图并执行："看看 task-xxx"、"把 task-xxx 移到 ready-for-agent"、"有哪些任务 agent 可以直接做"等。

## 分诊单个任务

1. **收集上下文**：读任务完整内容（正文/评论/标签/作者/日期），解析先前分诊 notes 避免重复问。代码库两个检查：(a) **冗余**——按领域概念搜是否已实现；(b) **先前拒绝**——读 `.out-of-scope/`（或对应知识库）看相似请求。
2. **推荐**：给类别和状态推荐及理由 + 简短代码库摘要。等维护者指示。
3. **验证主张**：拷问前先验证任务核心主张——bug 按报告者步骤复现，确认后报告。验证通过使 agent brief 更强。
4. **拷问（如需）**：请求需充实时调 `dj-grill` 和 `dj-domain-modeling` 逐轮拷问。
5. **应用结果**：
   - `ready-for-agent`: 写 agent brief（任务说明 + 上下文 + 验收标准）
   - `ready-for-human`: 同结构，注明为何不能委派
   - `needs-info`: 贴分诊 notes
   - `wontfix`: 关闭任务按因注释——已实现指位置 / 拒绝（bug）礼貌解释后关 / 拒绝（enhancement）记录理由后关

## 同步到 issue 跟踪器（可选）

若项目在本地任务外有真实跟踪器（Forgejo/Gitea/GitHub 等），分诊结果**双向同步**，本地任务为唯一权威源，不得只更新一侧：结果落地 → 跟踪器创建/更新对应 issue；`wontfix` → 按因评论后关闭；状态变化 → 补评论保持一致。

**格式统一（必须）**：标题 `<前缀>-<编号> [<类型>] [P?] <一句话摘要>`，正文固定章节骨架 + 尾注本地工单路径，**不得自创格式**（模板即统一格式，无需查看线上存量）。

**类型必标**：与类别角色一一对应——`[bug]` 对应骨架「现象→根因→影响→修复建议→关联」；`[feat]` 对应「需求→方案建议→影响/收益→验收标准→关联」。

**标签必挂**：类型标签 + 按需领域标签；只用仓库已有标签，缺的先建（约定色见 references）。

**关闭正规流程**：三重证据核实（跟踪器状态 + 本地提交 + 代码实证 grep，不信推测）→ 结题评论（commit 引用 + 验收对照 + 边界如实声明）→ PATCH closed → GET 复核 → 本地同步。模板全文与 API/平台细节（私有仓库假 404、Windows curl JSON 临时文件、token 同命令内赋值使用）见 `references/issue-sync.md`。

## Needs-info 模板

```markdown
## 分诊 Notes

**我们已确认：**

- 要点 1

**我们仍需要你提供（@报告者）：**

- 问题 1
```

## 快速状态覆盖

维护者说"把 task-xxx 移到 ready-for-agent"时信任并直接应用：先确认要做什么（角色变化/评论/关闭），然后执行，跳过拷问。无拷问直接到 `ready-for-agent` 时，询问是否要写 agent brief。

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