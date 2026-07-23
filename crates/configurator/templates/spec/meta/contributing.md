# 贡献工作流

## 目的
确保 DiJiang 的工作流行为、运行时门禁和文档在项目自身变更时保持同步。

## 规则
- 影响任务路由的代码和工作流变更，必须在同一个变更中同时更新 `README.md`、`.dijiang/workflow.md` 和 `crates/configurator/templates/config/workflow.md`。
- 运行时强制的工作流行为优先于 skill 文本。如果 CLI 或任务运行时实现了硬性门禁，文档必须将该门禁描述为事实来源。
- `planning` 任务可以暴露 `dj-output`，但活跃 `planning` 任务上的实现导向请求必须重定向到 `dj-grill`，直到任务向前推进。
- `paused` 的活跃任务必须先通过 `dijiang-continue` 恢复，然后才能继续实现或检查。
- `archived` 任务为只读状态。继续工作需要 `dijiang start <task>` 来重新激活任务。
- 新任务分类和活跃任务路由门禁是独立的关注点。文档不得暗示每个新请求都被强制通过活跃任务路由门禁。
- 当某个工作流阶段仅设计完成但尚未实现时，必须明确标注为后续阶段，而非描述为当前行为。
- 当某个阶段仅交付了部分运行时门禁时，文档必须同时描述已强制的边界和仍为手动的边界。
- 如果某个工作流或 Git 规则仍仅为 prompt 级别，需明确说明。如果是运行时强制的，需指明强制模块或 CLI 路径。

## 示例
- Route Gate 第一阶段：`crates/task/src/route_gate.rs` 决定活跃任务的 `allow`、`redirect` 或 `block`，CLI dispatch 输出 `action`、`reason` 和 `nextAction`。
- Git Gate 第二阶段：`crates/task/src/git_gate.rs` 评估 worktree 就绪状态，`crates/cli/src/main.rs::ensure_task_worktree(...)` 消费该决策，并为面向实现的活跃任务 dispatch 报告 `ready / provisioned / blocked`。
- 跨 worktree discovery：`crates/cli/src/util.rs::resolve_dijiang_dir` 供 CLI（含 `finish-work`）定位项目 `.dijiang/`。
- Phase 4 capability：`finish-work --integrate` / `--push` / cleanup 走 `evaluate_capability` 显式批准门禁。
- 受 Zleap 启发的设计：渐进式披露和审批策略可以在设计文档中作为后续阶段出现，而 README 和 workflow 只声明已交付的活跃运行时门禁。

## 反模式
- 在 Route Gate 已经落地后，仍将 `dj-grill` 强制描述为仅存在于 skill 文本中。
- 把跨 worktree 的 `.dijiang` discovery 或 `finish-work` 的 capability approval 说成 Phase 2 Git Gate 的同一条路径；discovery 在 `resolve_dijiang_dir`，finish 高风险操作在 `evaluate_capability`（Phase 4）。
- 当 CLI 有意为新工作保留分类器行为时，为新任务和活跃任务使用同一条路由表。
