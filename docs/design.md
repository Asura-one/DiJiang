# 设计决策

## 1. 单体 CLI 二进制

**决策**：所有 CLI 命令处理器集中在单一 `crates/cli/src/main.rs`（约 4000 行）。

**理由**：CLI crate 是集成层 —— 它编排对 `task`、`mem`、`configurator` 的调用。拆分为多个文件只会引入模块边界样板代码，不会降低复杂度，因为所有处理器共享状态（git 仓库路径、项目根目录、任务存储）和互相 dispatch 逻辑（route gate → git gate → skill 选择）。拆分后增加 `mod` 声明和可见性管理，但不提升可测试性 —— 集成点在调用二进制的 CLI 测试夹具中，不在内部单元测试。

**权衡**：单个命令实现的可发现性降低。通过 Clap 的自然子命令分组和大多数处理器遵循一致模式（解析参数 → 调库函数 → 格式化输出）来缓解。

**候选方案**：每个子命令组一个文件（`cmd_status.rs`、`cmd_task.rs` 等）。弃用原因是共享的 dispatch 和门禁编排在 `main.rs` 中 —— 移出处理器要么被迫跨文件开放可见性，要么重复公共模式。

## 2. 门禁式工况约束

**决策**：在三个门禁层面实施工况阶段转移：

- **Route Gate**（`task/src/route_gate.rs`）：Capsule 检测（`active_task.route_decision()`）—— dispatch 时将任务状态映射为 `allow`、`redirect` 或 `block`。
- **Git Gate**（`task/src/git_gate.rs`）：Worktree 就绪评估 —— 代码修改路由前返回 `ready`、`provisioned` 或 `blocked`。
- **Capability Gate**（`task/src/capability_gate.rs`）：finish-work 阶段破坏性操作（integrate、push、cleanup）的批准检查。

**理由**：纯提示级别的工况约束是脆弱的 —— agent 可以跳过步骤或误解意图。运行时门禁让错误转移在结构上不可行。三者分离隔离了关注点：路由有效性独立于 git 隔离，后者又独立于安全批准。

**第一性原理推导**：

- 问题：Agent 提示经常跳过工作流步骤，产生损坏的中间状态。
- 硬约束：CLI 是唯一的入口点，所有 dispatch 都经过它。
- 推论：在 CLI 层面施加约束，打破提示 ↔agent 反馈循环。

**设计选择**：门禁返回结构化 JSON（`action`、`reason`、`nextAction`），而非打印警告。Agent 可以编程方式消费该结果，同时获得门禁决策和机器可读的上下文对象。

## 3. 五层记忆架构

**决策**：将项目记忆组织为 5 层：Working → Episodic → Semantic → Procedural → Meta。

| 层         | 持久化                  | 保留期           | 访问模式   |
| ---------- | ----------------------- | ---------------- | ---------- |
| Working    | Session 范围            | Session 生命周期 | 始终可用   |
| Episodic   | 每个 session 一个 JSONL | 归档前           | 上下文注入 |
| Semantic   | 项目级 JSONL            | 永久             | 检索增强   |
| Procedural | Skill/tactic 注册表     | 永久             | 意图匹配   |
| Meta       | 平台特定                | 永久             | 备份/同步  |

**理由**：扁平记忆存储混淆了临时 session 状态和持久项目知识。分层结构让系统从原始 session 事件逐步提炼为持久模式和策略，同时保持工作集足够小以便注入提示。

**候选方案**：单一结构化文件，每条记录带 TTL。弃用原因：检索逻辑复杂（需要按年龄、来源、重要性过滤），且演化路径（session → pattern → tactic）在扁平存储中不易自然表达。

## 4. 跨平台记忆适配器

**决策**：在 trait `PlatformMemory`（`mem/src/adapter.rs`）后抽象平台记忆，每个平台有独立实现（Pi、Claude、Codex、Cursor、OpenCode、Hermes）。

**理由**：DiJiang 运行在多个 agent 平台中，各平台记忆 API 不同（认证、schema、访问模式不同）。Trait 抽象让 CLI 统一调用记忆操作，同时每个适配器处理平台特定序列化和传输。

**设计细节**：

- `registry.rs` 发现可用适配器（本地 JSONL 始终可用，平台适配器按需启用）。
- 备份静默同步本地 JSONL → 平台记忆，不打断 agent 工作上下文。
- 不做跨平台兼容性假设 —— 每个适配器独立将 DiJiang 记忆模型映射到平台 schema。

## 5. 任务 Worktree 隔离

**决策**：代码修改任务自动供应隔离 git worktree。主 checkout 保持干净。

**流程**：

```
dispatch(code route) → evaluate_worktree_readiness() → ensure_task_worktree()
  → git worktree add ../dijiang-<task-name> <branch>
    → finish-work auto-cleanup: 删除 worktree + 分支
```

**理由**：Agent 的代码编辑经常留下垃圾（过期文件、部分合并、测试产物）。隔离 worktree 保证无论编辑过程多混乱，主 checkout 始终保持干净。finish-work 中的 cleanup gate 要求显式批准后才能删除 worktree。

**硬约束**：`dijiang` 通过 `.dijiang/` 哨兵文件发现项目根目录。从主 checkout 启动的 worktree 仍需找到项目根目录。解决方法：worktree 创建时显式传入 `--project-root`。

## 6. 工作空间级版本号

**决策**：4 个 crate 共享同一工作空间版本（`v0.6.2`）。

**理由**：这些 crate 从不独立发布 —— 它们作为单一 CLI 二进制发布。独立版本号只会增加困惑（`dijiang-mem v0.3.1` 单独看有什么意义？）和 Cargo workspace 开销，没有任何运维收益。版本号的唯一消费者是 `dijiang --version` 和 `dijiang update`。

**例外**：ADR 系统独立于 crate 版本记录决策。ADR 按内容版本化（由 `dijiang spec-sync` 的 SHA256 链接），不按发布版本。

## 7. Doc-Sync / Spec-Sync

**决策**：两个独立的同步机制：

- **Doc-sync**（`task/src/doc_sync/`）：将 git diff 映射到 11 种长期文档类型，带置信度分数（0.4–0.9）。只读检查模式。
- **Spec-sync**（`task/src/spec_sync/`）：`.dijiang/spec/` 文件的 SHA256 checksum 追踪。支持 `check`（检测变更）和 `record`（更新数据库）。

**理由**：描述代码行为的文档所需的检测逻辑，与约束 agent 行为的 spec 文件不同。Doc-sync 变更意味着"更新此文档"；spec-sync 变更意味着"agent 指令变了 —— 重新评估工作流。"

**设计选择**：两者在 Phase 1 中均为只读（仅检测，不修改）。Phase 2 将与 `dj-write` 集成以实现自动文档更新。

## 8. Channel 系统（Sub-Agent 执行）

**决策**：将并行工作隔离到独立 agent "channel"中，每个 channel 有独立的 session 上下文、超时和结果收集。

**设计细节**：

- `channel spawn <agent>` —— 创建新 agent，带独立工作目录和提示上下文。
- `channel execute-all` —— 并行运行所有活跃 channel，可配置超时。
- Channel 输出为 JSON 结构，供父 agent 编程方式消费。
- 超时防止失控子 agent 阻塞整个工作流。

**理由**：试图通过建立多个独立对话来实现并行化的 agent（例如"你来安装依赖，同时你来格式化这个文件"）会产生协调错误。Channel 提供结构化的并行化模型和显式结果处理。

## 9. Trellis 向后兼容

**决策**：任务状态映射支持从 Trellis 状态的有损转换。`.trellis/` 作为遗留读回退保留。

**映射**：

```
Trellis         → DiJiang
todo              in_progress（Trellis 的 "已计划但未开始" 无直接对应）
in_progress       in_progress
done              completed
dropped           archived（与 "未完成即结束" 合并）
```

**理由**：项目从 Trellis 迁移到 DiJiang 时处于中途。活跃任务需要继续工作，无需手动重建。有损映射是可接受的 —— 这些状态用于路由门禁，不用于精确状态追踪：一个 `in_progress` 任务无论来源如何都保持 in_progress。

## 10. 渐进式 Skill 加载

**决策**：Skill body 不预先注入 agent 提示。仅先暴露清单（名称 + 描述）；路由引擎选定目标 skill 后才加载完整 body。

**实现**：`skill_manifest.rs` 维护所有已知 `dj-*` skill 的注册表，含元数据。dispatch 时，将路由目标的 manifest 解析为文件路径，`dijiang skill-body <name>` 懒加载完整文本。

**理由**：预注入所有 skill 文本会塞满提示上下文。Dispatch 引擎恰好选中一条路由（加上可能的重定向），因此只需要目标 skill 的 body。

## 11. Pi Extension（扩展优先于 CLI 直接调用）

**决策**：Pi Extension（`.pi/extensions/dijiang/index.ts`）在 agent 生命周期事件中自动注入 DiJiang 路由上下文。agent 应消费注入的 `<dijiang-workflow-state>` 和 `<dijiang-route>`，而非主动频繁调用 `dijiang dispatch`。

**实现**：扩展注册 6 个 Pi lifecycle hooks：

- `before_agent_start` / `user_prompt_submit`：调用 `dijiang workflow-state` 和 `dijiang dispatch --json --hook-event`，刷新 UI 并将路由上下文注入 agent 提示
  | `tool_call`：向 bash 命令注入 `DIJIANG_CONTEXT_ID`
  | `tool_result`：失败路由到 `dj-hunt`，通过验证且有脏 diff 路由到 `dj-output`
  | `session_start` / `session_shutdown`：刷新状态栏和 widget

**理由**：agent 主动调用 dispatch 有两个问题：1）时机不可控 —— agent 可能在错误的状态片段中调用 dispatch，路由到错误方向；2）增加延迟 —— 每次 dispatch 需要 CLI 启动和 gate 评估。Extension hook 在正确的生命周期事件中自动触发，时机确定且与 UI 刷新绑定。

**候选方案**：agent 在收到每个输入时自动调用 `dijiang dispatch`。弃用原因：agent 无法可靠判断当前是否是分类时机（可能是在 review 途中想查个资料，但 dispatch 误以为是一个新实现请求）。Extension 的 `user_prompt_submit` hook 由 Pi 平台稳定触发。

## 12. 路由上下文注入（自动注入而非主动请求）

**决策**：扩展在 agent 启动时自动注入 `<dijiang-workflow-state>` 块，包含最新工作流状态、Skill Manifests 和目标 skill 路由。agent 在所有后续操作中消费该注入。

**注入的上下文格式**：

```
<dijiang-workflow-state>
会话：pi
活跃任务：<task-id>
标题：<task-title>
状态：<in_progress|planning|...>
指引：<text>
……
</dijiang-workflow-state>

<dijiang-route>
路线: <skill-name>
原因: <trigger-reason>
下一步: <action-description>
</dijiang-route>

**同时注入的 Skill Manifests**：
```

<dijiang-target-skill role="primary" name="dj-implement">
summary: ...
</dijiang-target-skill>

<dijiang-skill-manifests>
- dj-implement | 特性代码实现 | phases=in_progress | risk=medium
- dj-tdd | 测试驱动开发 | phases=in_progress | risk=medium
- ...
</dijiang-skill-manifests>

**理由**：agent 的 prompt token 有限，频繁读取文件或调用 CLI 浪费上下文。一次性注入完整路由上下文让 agent 立即知道当前状态、可用 skill 和目标 skill，不需要在实现过程中反复查询。

**关键区别**：agent 在扩展注入了 `<dijiang-route>` 后不应再自行调用 `dijiang dispatch`。只在主动请求（如用户手动输入 `/dijiang` 命令）需要重新分类时，agent 才调用 dispatch。

## 13. 子 Agent 职责分离

**决策**：将质量、实现、调研三种职责分离到三个独立子 agent（`.pi/agents/`），各自加载对应 skill 清单。

| 子 agent          | 加载的 skill 清单                                          |
| ----------------- | ---------------------------------------------------------- |
| dijiang-check     | dj-check, dj-audit, dj-debt, dj-health                     |
| dijiang-implement | dj-implement, dj-tdd, dj-prototype, dj-ponytail, dj-script |
| dijiang-research  | dj-hunt, dj-dispatch, dj-pattern                           |

**理由**：单一 agent 加载全部 skil 会超出 prompt 上下文。分离后每个子 agent 只需知道所属领域的 skill。同时职责边界更清晰 —— check agent 不会误入实现路径。

**设计细节**：

- 子 agent prompt 首行要求读取 `dijiang workflow-state --json`，不假设预注上下文
- `<dijiang-target-skill>` 覆盖默认 skill 映射，子 agent 在没有注入时使用内置 fallback
- 子 agent 输出 JSON 给父 agent 消费，不直接修改项目状态

## 权衡汇总

## 权衡汇总

| 决策           | 收益                            | 牺牲                            |
| -------------- | ------------------------------- | ------------------------------- | --------------------------- |
| 单一 main.rs   | 简单集成，无模块样板            | 约 4000 行，导航困难            |
| 门禁式约束     | 结构性正确性保证                | Dispatch 路径的运行时复杂度     |
| 五层记忆       | 从 session 到 tactic 的清晰演化 | 更多文件描述符（JSONL）         |
| 跨平台适配器   | 统一 API，平台灵活性            | 每个平台的维护负担              |
| Worktree 隔离  | 主 checkout 保持干净            | 任务期间磁盘用量约 2 倍         |
| 工作空间级版本 | 简单                            | 无逐 crate 变更日志             |
| Doc/Spec-sync  | 精准、正确更新                  | 两个机制而非一个                |
| Channel 系统   | 结构化并行                      | Agent 生命周期复杂度            |
| t7J            | 渐进式 skill 加载               | 高效上下文使用                  | 首次使用的懒加载延迟        |
| 4Ud            | Pi Extension 优先               | 自动时机，UI 同步，去重         | 扩展层维护成本              |
| KeL            | 路由上下文注入                  | 减少 agent 主动查寻，节省 token | 注入信息可能滞后于 CLI 状态 |
| Dhe            | 子 Agent 职责分离               | prompt 空间节省，职责清晰       | 跨 agent 协调复杂度         |
