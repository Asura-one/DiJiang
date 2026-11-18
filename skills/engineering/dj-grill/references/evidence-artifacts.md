# dj-grill：门禁证据示例与 Artifact Rules 细节

本文件承载 `meta.grilling` 最小门禁证据的 JSON 结构示例与 Artifact Rules 的任务文档维护细则。SKILL.md 保留写入时机与触发条件；结束访谈写入门禁证据或维护任务文档时读本节。

## 最小门禁证据示例

结束时写入 `{task_dir}/task.json` 的最小门禁证据：

```json
{
  "meta": {
    "grilling": {
      "mode": "grill-me",
      "startedAt": "2026-07-30T08:00:00Z",
      "questions": [{
        "prompt": "最高信息量的决策题",
        "recommendation": "推荐答案",
        "answer": "用户回答"
      }],
      "confirmedAt": "2026-07-30T08:05:00Z",
      "confirmation": "用户明确确认共享理解的原话"
    }
  }
}
```

## Artifact Rules

grilling 过程中同步维护以下任务文档：

| 文档 | 位置 | 职责 |
|------|------|------|
| `prd.md` | `{task_dir}/prd.md` | 需求文档，包含目标/范围/方案/约束/验收/风险 |
| `design.md` | `{task_dir}/design.md` | 技术设计（复杂任务需要） |
| `implement.md` | `{task_dir}/implement.md` | 执行计划（复杂任务需要） |

### prd.md 更新规则

- 每次用户回答后，更新 `{task_dir}/prd.md` 中的对应维度
- 不确定的内容留 `TBD` 标记
- grilling 结束前做一次 PRD 完整性检查

### `grill-with-doc` 文档契约

`grill-with-doc` 在上述任务文档之外，复用 `dj-domain-modeling` 的文档格式，留下与原版 `grill-with-docs` 等价的纸面记录：

- 术语一经澄清，立即更新 `CONTEXT.md`；只记录规范术语和定义，不混入实现方案或需求细节。
- 只有决策难以逆转、缺少背景会令人意外、并且确实比较过替代方案时，才在 `{task_dir}/adr/NNN-title.md` 新建 ADR。ADR 必须记录状态、上下文、选择与放弃的方案、影响。
- 可逆、没有真实权衡的日常决定只写入 `prd.md` 或 `design.md`；不为凑文档创建 ADR。

### PRD Convergence Pass

结束前检查 `prd.md` 是否记录了会影响实现的目标、范围、方案、约束、验收与风险。已由用户材料确定的维度直接记录；只有缺失且会阻塞实现时才补问。
