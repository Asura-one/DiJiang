---
name: dj-triage
description: "将 issue 和外部 PR 通过分诊角色状态机流转：分类、验证、必要时拷问、编写 agent-ready briefs。适配本地文件任务存储。"
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
