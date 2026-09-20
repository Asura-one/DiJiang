---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '648c7f03-394d-483a-a023-036bcd6074f1'
  PropagateID: '648c7f03-394d-483a-a023-036bcd6074f1'
  ReservedCode1: 'ad573154-4fdf-4fdc-9283-689b30f631ad'
  ReservedCode2: 'ad573154-4fdf-4fdc-9283-689b30f631ad'
---

# DiJiang

本文件为 Claude 提供 DiJiang 项目上下文。

## 项目结构

- `skills/` — 所有 DiJiang skill（engineering/productivity 两个 bucket）
- `.dijiang/` — 项目本地状态（gitignored，由 `dj-setup` 初始化）
- `CONTEXT.md` — 领域术语表
- `docs/adr/` — 架构决策记录
- `docs/references/` — 跨技能参考文档

## Skill 使用

通过 `Call the Skill tool with "<skill-name>"` 调用 skill。

### 常用入口

1. **新项目**：`dj-setup` 初始化
2. **全局流程**：`dj-ask` 查看 DiJiang 有哪些 workflow / 该走什么流程
3. **新任务**：`dj-dispatch` 分流路由
4. **需求对齐**：`dj-grill` 逐轮拷问
5. **实现**：`dj-implement` 或 `dj-tdd`（改码护送走 `dj-regression-guard`）
6. **排查**：`dj-hunt`
7. **审查**：`dj-check`
8. **全量测试**：`dj-fullstack-testing`
9. **收尾**：`dijiang-finish-work`

### 任务状态

读取 `.dijiang/active_task.txt` 获取活跃任务指针，然后读取 `.dijiang/tasks/<name>/task.json` 获取任务状态。

| 状态 | 下一步 |
|------|--------|
| planning | `dj-grill` → `dj-output` → `dj-split` |
| in_progress | `dj-implement` / `dj-tdd` → `dj-regression-guard`（护送）→ `dj-check` |
| completed | `dijiang-finish-work` |
| paused | `dijiang-continue` |

## 核心工作流

1. **plan**: 读取 CONTEXT.md 和 .dijiang/spec/ 中相关规范。用 `dj-grill` 对齐需求。
2. **implement**: 按 PRD/design 实现代码。运行测试验证。
3. **check**: 用 `dj-check` 审查代码质量。
4. **archive**: 用 `dijiang-finish-work` 收尾、提交、归档。