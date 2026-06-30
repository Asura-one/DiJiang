
## 2026-06-30 16:45
用户反馈：DiJiang 目前不像 Trellis；自然语言输入后没有自动判断并创建任务，表现像裸 agent 执行。初步诊断：workflow-state/session hook 只注入状态，不驱动任务创建；dj-dispatch 是 skill 文档而非强制入口。

## 2026-06-30 18:06
DiJiang runtime dispatch/worktree workflow task: configurator lib、dijiang-task、dispatch e2e、finish-work e2e 通过；全量 e2e 中 test_e2e_update_refreshes_existing_platform_hooks 因 .pi/skills/dj-design/SKILL.md 被判定为受管文件冲突失败，需后续单独处理 update hash/conflict 逻辑。

## 2026-06-30 21:45
Pi 新项目自然语言未触发工作流的根因：项目扩展模板缺少 before_agent_start dispatch hook，仅注入 workflow-state；修复为在非 slash 自然语言输入时调用 dijiang dispatch，并用 e2e 断言初始化模板包含 dispatch。
