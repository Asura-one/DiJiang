# DiJiang Reason

这是 `/skill:dj-reason` 的轻量入口。`dj-reason` 是 DiJiang 的手动推理增强 skill，用于复杂判断、需求澄清、架构取舍、排障复盘、系统透镜分析和质量审查前的认知校准。

使用方式：

1. 加载 `/skill:dj-reason`。
2. 按 skill 指令读取上下文、区分事实/推断/假设、选择认知路径。
3. 在问题涉及工作流、组织行为、长期副作用或反复出现的流程问题时，启用系统透镜。

边界：

- 此 prompt 只做入口提示，不承载完整推理流程。
- `dj-reason` 只分析，不写代码、不改变 DiJiang workflow state、不替代 `dj-dispatch`、`dj-grill`、`dj-check` 或任何 CLI 状态转换。
- 如果当前任务受 route gate 约束，仍以 workflow-state 允许的 `dj-*` 路线为准。
