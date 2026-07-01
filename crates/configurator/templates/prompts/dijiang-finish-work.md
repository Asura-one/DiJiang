## Finish Work

Complete your current task and prepare for `dijiang finish-work`. This prompt is a checklist only; use `/skill:dijiang-finish-work` for the agent workflow and `dijiang finish-work ...` for the CLI state transition.

Steps:
1. Run relevant checks or tests
2. Verify type checks pass when applicable
3. Run `dj-check` if code changed
4. Sync task artifacts, docs, spec, or changelog; record `docs-sync: none` with reason if no update is needed
5. Decide version impact: `major`, `minor`, `patch`, or `none`
6. Record findings or lessons with `dijiang mem findings` / `dijiang mem learn` when they pass the memory quality gate
7. Finish with `dijiang finish-work --verification "..." --docs-sync "..." --version-impact none --commit` when a scoped commit is needed
8. Add `--push` and `--integrate` only when push/merge/worktree cleanup is explicitly allowed
