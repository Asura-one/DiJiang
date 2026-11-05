# Loop Run State 模板

> 摘要：loop 单轮运行态的状态文件模板——区别于 Task State（任务进度）；只存下一轮要读的决策依据，叙事过程进 trace。

```markdown
# <loop-name> Run State

- run_id:
- started_at:
- stage: <采集|分析|产出|gate>
- attempts: <本目标第几次尝试>
- budget: { token_used, token_limit, elapsed, limit }
- latest_gate: { name, result: verified|not_run|inconclusive, evidence_path }
- next_action: <下一轮第一件事，一句话>
- blockers: <升级中的问题；无则空>
- stop: { hit_budget: false, gate_failed_times: 0, escalate_to: null }
```

## 字段纪律

- **state 是决策依据不是日志仓库**：删掉任何字段不影响下一轮决策的，不该在 state 里——过程叙事进 trace（只追加）
- 每轮末尾原子写入（先写临时文件再 rename）；崩溃后从 state 恢复，不从零
- Task State（`.dijiang/tasks/`）与本模板是两个东西：前者管任务进度，后者管运行轮次——术语见 CONTEXT.md

来源：琅嬛 templates/STATE.md 改写 + 「Loop State File 设计模式」字段纪律
