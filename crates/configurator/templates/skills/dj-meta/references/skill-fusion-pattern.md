# dj-* Skill Fusion Pattern: Lessons from mattpocock/skills

## The Core Insight

mattpocock/skills 的核心优势不在于内容多，而在于**砍掉了什么**。

每个 SKILL.md 只保留 "agent 执行时必须逐条照做的最小指令集"。信任 agent 有能力处理边缘情况，不需要把所有 "如果 XX 失败怎么办" 都写进去。

## Applied in Phase 1 (2026-07-08)

### Before (Problem)
dj-* 技能承担了太多角色：
- 核心指令
- 输入/输出规格
- 失败处理参考
- 培训材料（反例教育）
- 模板参考

结果：平均 7KB，最大 17KB。Agent 读技能时注意力被非核心内容稀释。

### After (Solution)
每个 SKILL.md 只保留：
- 职责（一句话）
- 工作流步骤
- 核心规则（3-5 条）
- 精简反例（3-4 条）
- 验证命令

移到了 `references/`：
- 输入/输出规格表 → `references/io.md`
- 失败处理表 → `references/failure-handling.md`
- 深度原则 → `references/principles.md`

结果：平均 2.1KB，总大小从 149KB → 47KB（-71%，含 3 个新技能）

## The Pipeline Pattern (Phase 2)

完整的开发前流水线：

```
dj-grill ──→ dj-prd ──→ dj-split ──→ dj-implement / dj-tdd
   │                        │
   ├ glossary               └ 独立可执行的任务
   │ 术语收集、更新               列表
   │ .dijiang/glossary.md
   │
   └ 需求摘要 + 术语确认
```

## Quality Pattern (Phase 3)

- **并行审查**: `dj-review` 使用 `delegate_task` 同时审查 spec 匹配度和代码质量
- **元技能**: `dj-meta` 记录设计原则和创建模板
- **ADR**: `.dijiang/decisions/` 记录跨技能设计决策

## Directory Boundary Rule

Critical lesson: skill files and project files must follow strict boundaries.

```
~/.hermes/skills/<skill>/    ← skill-level
  SKILL.md                     core instructions
  references/                  reference docs

.dijiang/                     ← project-level
  glossary.md                   created by dj-grill on first use
  decisions/                    created by dijiaang adr on first use
  prd/                          created by dj-prd on first use
```

## Decision Tree vs Fixed Checklist

The biggest improvement from comparing dj-grill with grill-me:

| Fixed checklist | Decision tree |
|---|---|
| Steps 1→2→3→4→5 in order | Define N dimensions, chase the most uncertain |
| Agent feels "done" after checklist walkthrough | Agent is done when all branches are clear |
| Misses domain-specific blind spots | Adapts to what's actually unclear |
| Feels mechanical | Feels like a senior engineer probing |

Apply decision-tree pattern to any investigation/audit/review skill.

## grill-me Comparison

Why grill-me works better than the initial dj-grill:
1. **Thin entry**: 3 lines with `disable-model-invocation: true`. User decides to engage.
2. **Branching strategy**: Walks every branch of the decision tree, not a linear checklist.
3. **Adaptive order**: Always pick the most uncertain branch next.
4. **Termination**: "shared understanding" — not "all questions answered".

## Volume Guidelines

| Metric | Target | Hard Limit |
|--------|--------|------------|
| SKILL.md size | 800-2500B | 3000B |
| Skill count | 20-25 class-level | -- |
| 反例 entries | 3-4 | 5 |
| References per skill | 2-3 | -- |

## Pitfalls

- TDD Contract markers must be preserved in relevant skills
- New skills must be synced to template directory
- Don't put skill files under `.dijiang/` -- that's project-level
- Don't hardcode project data paths in skills -- create on first use
