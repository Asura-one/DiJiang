# Finish Work Automation Design

## Problem

`dijiang finish-work` 的 skill/workflow 文案声明它负责 version、commit、push、merge 和 worktree cleanup，但 CLI 实现只做验证、dirty 检查、任务归档和 journal 记录。用户选择将 CLI 实现补齐为真实收尾命令，并要求确认文档同步是否包含在 finish-work 中。

## Decision

实现显式自动化，而不是默认静默执行 git 动作。

`dijiang finish-work` 新增参数：

- `--docs-sync <evidence>`：文档/spec/changelog 同步证据；有 diff 或 `--commit` 时必填。
- `--version-impact <major|minor|patch|none>`：版本影响决策，默认 `none`。
- `--commit`：更新可发布版本元数据、归档任务、写 journal 后，提交当前任务 diff。
- `--commit-message <message>`：提交消息；未提供时根据任务和 summary 生成。
- `--push`：提交后 push 任务分支。
- `--integrate`：提交后在主分支 worktree 中 `--no-ff` 合并任务分支，清理任务 worktree，并删除已合并分支。
- `--main-branch <branch>` / `--remote <name>`：集成目标与远端配置。

## 版本语义

`--version-impact major|minor|patch` 会更新根 `Cargo.toml` 的 `[workspace.package] version`。如果项目没有可发布版本元数据，则不修改版本文件，只记录版本影响决策。当前实现覆盖 GNU 风格三段版本号：`Major.Minor.Revision`。

| 决策 | 行为 |
|---|---|
| `major` | `X.Y.Z -> X+1.0.0` |
| `minor` | `X.Y.Z -> X.Y+1.0` |
| `patch` | `X.Y.Z -> X.Y.Z+1` |
| `none` | 不修改版本文件 |
## 文档同步语义

finish-work 现在包含文档同步门禁，但不自动替用户推断并修改文档内容。

含义：

1. 实现阶段或 `dj-output` 先完成 task artifact、docs、spec、changelog 的实际同步。
2. `finish-work --docs-sync "<evidence>"` 记录同步证据。
3. 如果不需要同步，必须显式写 `--docs-sync "none: <reason>"`。
4. 缺少 `--docs-sync` 时，存在 diff 或使用 `--commit` 会阻塞 finish-work。

这样可以防止“代码提交了但文档/spec 没检查”的收尾漏洞，同时避免 CLI 在不了解需求语义时自动改文档。

## Safety Rules

- `--push` / `--integrate` 必须和 `--commit` 同时使用。
- `--commit` 不能和 `--allow-dirty` 同时使用。
- `--integrate` 不允许在主分支上执行。
- `--integrate` 在主分支 worktree 中合并和删除任务 worktree，不在任务 worktree 中删除自己。
- 默认不 push、不 merge、不清理 worktree。

## Workflow Projection Sync

`dj-*` 原子能力边界调整后，同步了所有会投影给 agent 的入口：

- README workflow：删除已移除的 `dijiang review` 兼容入口描述，补充 `dj-review` 是轻量只读审查 lens，质量闸门仍是 `dj-check`。
- `.dijiang/workflow.md` 模板和当前投影：`completed` 阶段改为新 `finish-work` 门禁参数，skill taxonomy 增加 `dj-review` 边界。
- AGENTS 模板和当前投影：CLI command 和 workflow routing 改为 `--verification` + `--docs-sync` + `--version-impact`，并补 `dj-review` 与 `dj-check` 的职责区分。
- `.pi/prompts/dijiang-finish-work.md` 模板和当前投影：finish prompt 增加 docs/spec sync、version decision、memory quality gate、显式 commit/push/integrate 语义。
- `dijiang-start` / `dijiang-continue`：保留 session wrapper 的路线选择职责，强路由措辞改为 `follow-up` / `blocking` 输出，不让 wrapper 声称直接调用原子 skill。
- `dijiang-finish-work`：失败处理改为输出 blocker/follow-up，不再写“回到实现”式跨 skill 强编排。

## Validation

- `cargo fmt --check` passed.
- `cargo check -p dijiang` passed.
- `cargo test -p dijiang --test e2e test_e2e_finish_work_commit -- --nocapture` passed.
- `cargo test -p dijiang --test e2e` passed after updating the dirty-worktree test for the new docs-sync gate.
- `rg -n 'dijiang review|review 只是|兼容保留' README.md crates/configurator/templates crates/cli/src/main.rs; test $? -eq 1` passed.
- `rg -n 'finish-work --verification "\\.\\.\\."(?! --docs-sync)|`dijiang finish-work --verification "\\.\\.\\."`|completed` → `dijiang finish-work --verification "\\.\\.\\."`' README.md AGENTS.md .dijiang/workflow.md .pi/prompts crates/configurator/templates .dijiang/tasks/dijiang-git-worktree-version-commit-push-merge-w/design.md --pcre2; test $? -eq 1` passed.
- Markdown table cells that mention version impact use `<major/minor/patch/none>` instead of a bare-pipe enum, so tables render with stable columns.
- `rg -n '<major\\|minor\\|patch\\|none>' README.md AGENTS.md .dijiang/workflow.md crates/configurator/templates/config; test $? -eq 1` passed; no-match is the expected success condition for this drift scan.
- `rg -n 'Route to|route to|return to|delegate to|Load `dj-|调用|回到|切换|路由到|返回到' crates/configurator/templates/skills/dijiang-*; test $? -eq 1` passed.
