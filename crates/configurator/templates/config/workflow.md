# 开发工作流

---

## 核心原则

1. **先计划，再写代码** — 需求不清时，先对齐范围再实现。
2. **Spec 注入，不靠记忆** — 指南通过 hook/skill 注入，不从记忆中回想。
3. **持久化决策** — task artifacts、findings、lessons 和 handoffs 写入 `.dijiang/`。
4. **一个规范 workflow** — CLI、skills、AGENTS、prompts 和 agents 都是这个模型的投影。
5. **验证循环优先** — 把目标拆成可证明的命题，先建立 pass/fail 反馈回路，再让实现、审查和记忆围绕它收敛。
6. **决策可持久追溯** — 重要取舍写 ADR，记录 why、状态和替代方案；设计文档描述当前形态并引用 ADR。
7. **复利式学习** — AI 造成或发现的问题必须沉淀到 prompt、skill、spec 或 memory，让下一轮工作少犯同类错。
8. **记忆有质量门禁** — 长期记忆必须有 source、scope、confidence、freshness、conflict、actionability；不满足就留在 task artifact。
9. **Git 隔离优先** — 所有会修改代码的任务，修改前都必须创建专用 worktree/branch；任务结束时先做版本决策，再按权限完成提交、push、合并和 worktree 清理。

## DiJiang 规范工作流

DiJiang 使用 `dijiang` CLI 管理项目状态，使用 `dj-*` skills 执行具体能力。`review` 不是规范任务状态；质量验证由 `dj-check` 处理。

| 任务状态 | Workflow 阶段 | 推荐入口 | 输出 |
|-------------|----------------|-------------------|--------|
| none | dispatch | `dijiang start <name>` or `dj-dispatch` | 当前任务和路由决策 |
| `planning` | align | `dj-grill`, optionally `dj-output` | `prd.md`，可选 `design.md` / `implement.md` |
| `in_progress` | implement | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-script` / `dj-design` | 可工作的代码、测试和验证记录 |
| `in_progress` | check | `dj-check` | 已验证 diff 和后续修复 |
| `completed` | finish | `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | 版本决策、文档/spec 同步证据、范围一致的提交、journal、归档任务、清理当前 session active task |
| `archived` | closed | 只读，或用 `dijiang start <task>` 重启 | 已归档任务无 active work |
| `paused` | resume | `dijiang-continue` | 恢复上下文，然后回到 `planning` 或 `in_progress` |

## Runtime Route Gate

当前已有一层 runtime hard gate 管 active task 的 workflow route。它不是 skill prose 的建议，而是 CLI/task runtime 的真实约束。

- `planning` active task 只能放行 `dj-grill` 或 `dj-output`；实现、排查、检查类请求会被 redirect 到 `dj-grill`。

- `paused` active task 会 redirect 到 `dijiang-continue`。

- `archived` active task 会 block 到 `dijiang-start`。

- `completed` active task 默认面向 `dj-check` 或 `dijiang-finish-work`。

- 新建任务保留 classifier 分流，不强行套用 active-task route gate。

当前 gate 的事实源在 `crates/task/src/route_gate.rs`，可视化注入在 `crates/task/src/workflow_state.rs`，CLI 消费点在 `crates/cli/src/main.rs`。

## Runtime Git Gate

当前已有一层最小 runtime Git Gate 管 active task 的实现类 dispatch。它会先判断 task worktree metadata、当前 runtime 所在 worktree、以及是否需要 provision，再决定 `ready / provisioned / blocked`。

- `dj-implement`、`dj-tdd`、`dj-hunt`、`dj-script`、`dj-design` 这类实现路线会触发 Git Gate readiness 检查。

- 缺少 task worktree metadata 时，Git Gate 会进入 `blocked`，并在满足 git baseline 前提时由 CLI 自动 provision task worktree。

- task worktree 已存在，但当前 runtime 仍在主 checkout 或错误 worktree时，Git Gate 会保持 `blocked`，不再只给 prose 提示。

- 不需要立即写代码的路线，例如 `dj-grill` 或 `dj-output`，不会提前 provision worktree。

当前 Git Gate 的事实源在 `crates/task/src/git_gate.rs`，可视化注入在 `crates/task/src/workflow_state.rs`，CLI 统一消费点在 `crates/cli/src/main.rs::ensure_task_worktree(...)`。

## Progressive Skill Loading

当前 Progressive Skill Loading 已部分落地：`dispatch` 与 `workflow-state` 两个 agent-facing runtime 入口都会先暴露 capsule-scoped skill manifests，再按 route 目标延迟展开单个或少量顺序/分叉 skill body；Pi/Codex/OpenCode/Hermes 的 agent prompt 也已统一改成优先消费这套 runtime context，并补上了 risk/capsule 驱动的最小展开阈值。另一个兼容优先的执行期通路是 `dijiang skill-body`，可在不改变现有注入形态的前提下按 skill 名按需取 body。

这还不等于 Phase 3 全链路完成。当前尚未覆盖的部分只剩：默认预注 body 的全链路切换。

Phase 4 已开始最小闭环：当前先覆盖 `finish-work --integrate`、`finish-work --push` 与 merge 后 cleanup（worktree remove / branch delete）这几条高风险路径，要求显式 approval 后才允许继续 merge / push / cleanup。它还不是完整 capability system；后续仍要扩到更广泛的高风险动作。

## Skill 分类

| 类别 | Skills | 边界 |
|----------|--------|----------|
| 路由 | `dj-dispatch` | 分类和路由；不直接实现 |
| 对齐 | `dj-grill` | 需求对齐；不写代码 |
| 实现 | `dj-implement`, `dj-tdd`, `dj-hunt`, `dj-prototype`, `dj-script`, `dj-design` | 写代码或调查根因
| 复刻 | `dj-remix` | 系统化复刻站点/App并做差异化改造 |
| 实现 | `dj-implement`, `dj-tdd`, `dj-hunt`, `dj-prototype`, `dj-script`, `dj-design` | 写代码或调查根因 |
| 质量门禁 | `dj-check` | 验证 diff 质量、完整性、安全性和回归 |
| 审查视角 | `dj-review` | 轻量只读 review；不运行测试、不改代码、不替代 `dj-check` |
| 分析报告 | `dj-audit`, `dj-debt`, `dj-health`, `dj-pattern` | 产出报告；不是默认交付门禁 |
| 推理增强 | `dj-reason` | 复杂判断、系统透镜和认知校准；只分析，不改变 workflow state |
| 风格叠加 | `dj-ponytail`, `dj-karpathy` | 给其他 workflow 路径增加约束 |
| 写作润色 | `dj-write` | 润色文本；不负责工程文档生命周期 |
| 会话交接 | `dj-handoff` | 准备 handoff；不替代 finish-work journal |
| 会话包装器 | `dijiang-start`, `dijiang-continue`, `dijiang-finish-work` | 加载上下文、路由和关闭会话；skill 执行不同于 prompt checklist 注入 |

## Verification & Compound Loop

DiJiang 的默认交付循环是 `Plan → Verify → Work → Check → Compound`。

1. **Plan** — 明确用户目标、约束、完成标准和不做什么；重大取舍写 ADR，普通任务写 task artifacts。
2. **Verify** — 为目标选择最小可执行反馈回路：测试、CLI fixture、HTTP 脚本、浏览器/OCR、trace 重放或人工可复核清单。
3. **Work** — 只围绕反馈回路实现，保持改动可审查；遇到不一致先保护源事实，不擅自“纠正”旧系统术语、字段或界面文案。
4. **Check** — 对照需求、反馈回路、引用点和风险面审查。高风险改动可拆成多个 reviewer lens：correctness、security、performance、architecture、docs。
5. **Compound** — 把新发现写回 `.dijiang/`：bug 根因进入 `dj-hunt` 记录，长期约束进入 spec，重大决策进入 ADR；只有通过记忆质量门禁的经验才进入 memory。

反馈回路必须能回答“这个目标是否已经达成”。如果只能说明“代码看起来合理”，它还不是验证。

## Code Task TDD Contract

所有会修改代码、行为、配置、模板或脚本的任务默认遵守 TDD 约束。TDD 在 DiJiang 中不是形式化地堆测试，而是先固定目标行为和回归边界，再实现最小改动。

编程任务进入实现前必须写清：

```text
Behavior/Invariant: <要保护或新增的行为命题>
RED/Repro evidence: <先失败的测试、复现命令、fixture、trace 或人工可复核步骤>
GREEN command: <实现后必须变绿的最小命令或检查>
Regression scope: <可能受影响的调用方、兄弟路径、全量/相关测试范围>
Exception: <none，或无法自动化/纯机械变更/环境不可用的具体原因和替代检查>
```

规则：

1. 新功能或行为变化优先走 `dj-tdd`：先 RED，再 GREEN，再 REFACTOR，再 RECORD。
2. Bug 修复优先走 `dj-hunt`：先定位根因并保留 RED/Repro evidence，再修复到 GREEN。
3. `dj-implement` 不绕过 TDD contract；它只在需求已清楚、反馈回路已定义时执行实现。
4. `dj-check` 必须把 RED/Repro、GREEN、Regression、Exception 作为编程 diff 的交付门禁；缺失证据时不能给通过结论。
5. `dijiang-finish-work` 的 verification 必须包含 TDD evidence，或说明本次变更为什么不适用。

## Git Worktree 生命周期

所有会修改代码的工作，在修改任何文件前都必须使用隔离 worktree。主 checkout 保持纯净，只在任务分支完成后用于集成。

1. **修改前** — 从目标 base branch 创建任务分支和 worktree。分支名使用 `<type>/<task-slug>`，worktree 路径使用 `../<repo>-<task-slug>`。如果已经在主 checkout 中，先停止编辑，创建或切换到任务 worktree。

2. **实现中** — 所有修改只留在任务 worktree。不要因为某个逻辑单元完成就提交；实现、检查、文档/spec 同步和版本决策都完成后再进入 finish-work。

3. **文档同步** — 代码、行为、CLI、配置或模板改变后，先同步相关 task artifact、spec、docs 或 changelog；若无需更新，记录 `docs-sync: none` 和原因。

4. **版本决策** — 任务结束时判断变更属于 `major`、`minor`、`patch` 或 `none`。只有项目存在可发布的 package/version 元数据，且变更需要发布时才更新版本文件。

5. **提交内容** — `dijiang finish-work --commit` 只提交当前任务的实际 diff；提交前必须提供 `--verification`、`--docs-sync` 和 `--version-impact`。commit message 描述行为变化，不堆文件名。`/dijiang-finish-work` 只是 Pi prompt checklist，`/skill:dijiang-finish-work` 是 agent workflow，真实归档/提交状态只由 `dijiang finish-work ...` CLI 修改。

6. **Push 与集成** — `dijiang finish-work --integrate` 是本地集成动作：在主分支 worktree 中 `--no-ff` 合并任务分支、清理任务 worktree 并删除已合并分支。`--push` 是可选发布动作；远端明显不可达、凭证缺失或策略不允许时，不阻塞本地 merge 和 worktree 清理，必须报告 push 阻塞原因。

7. **主仓库落地** — 任务分支完成但未能执行 `--integrate` 时，必须在主 checkout 明确执行 `git merge <task-branch>` 或记录未合并原因。只要任务分支已合并到主分支，就应删除任务 worktree 和已合并本地分支；只有未合并、存在冲突、范围未确认或需要人工保留证据时才保留。任何面向本机安装、发布、演示或后续开发的命令，都必须基于已合并后的主 checkout，避免从旧代码执行 `make install`、构建或更新。

这套 Git 规则已经有一部分下沉成 runtime gate。当前 Phase 2 只覆盖 dispatch / active-task implementation route 的 readiness、provision 和 blocked reason；还没有覆盖跨 worktree 的 `.dijiang` discovery，也还没有接到 `finish-work`。这两个点应被视为后续工作，而不是当前默认能力。

## Project Structure

```
.dijiang/            # DiJiang project state
├── tasks/           # Task directories (task.json, prd.md, design.md, …)
├── spec/            # Coding guidelines by package/layer
├── workspace/       # Developer journals
├── workflow.md      # This file
└── config.toml      # DiJiang configuration

.pi/                 # Pi platform configuration
├── settings.json    # Platform settings
├── skills/          # Project-level skills
├── agents/          # Sub-agent definitions
└── prompts/         # Prompt templates
```

.trellis/ may be read only as a legacy compatibility fallback. New DiJiang templates should use `.dijiang/` as the primary path.

## CLI 命令

| 命令 | 说明 |
|---------|-------------|
| `dijiang init [name]` | 初始化 DiJiang 项目状态和平台配置 |
| `dijiang status` | 显示项目和当前任务状态 |
| `dijiang status --compat` | 显示兼容性诊断 |
| `dijiang start <name>` | 创建并激活一个工作会话 |
| `dijiang dispatch <prompt>` | 从自然语言请求创建或复用 active task，并输出路由上下文 |
| `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | 在验证、文档/spec 同步证据、版本决策、范围一致的提交/发布决策、journal 记录后完成当前工作并归档 |
| `dijiang task list` | 列出所有任务 |
| `dijiang task current` | 显示 active task |
| `dijiang task start <name>` | 用低层任务语义创建或激活任务记录 |
| `dijiang task status <name> <status>` | 更新任务状态 |
| `dijiang task archive <name>` | 归档任务 |
| `dijiang task prune --days N` | 删除早于 N 天的已归档任务 |
| `dijiang mem list` | 列出平台会话 |
| `dijiang mem sync` | 同步平台会话到 `~/.dijiang/mem/` |
| `dijiang mem findings --finding "..."` | 追加项目发现 |
| `dijiang mem learn --lesson "..."` | 记录项目经验 |
| `dijiang mem correction --correction "..." --lesson "..." --actionability "..."` | 记录带记忆质量元数据的用户纠正 |
| `dijiang mem archive` | 归档当前 memory session |
| `dijiang mem tactic --name N --description D` | 添加全局 tactic |
| `dijiang mem tactics --select N` | 列出或选择使用 Thompson sampling 的 tactics |
| `dijiang mem record --tactic T --outcome success --context C` | 记录 tactic 结果 |
| `dijiang mem pattern --name N --description D` | 添加项目 pattern 或标准操作流程 |
| `dijiang mem patterns` | 列出项目 patterns |
| `dijiang mem stats` | 显示 memory 统计 |
| `dijiang mem backup` | 将项目 memory 备份到全局存储 |
| `dijiang mem evolve` | 分析 session memory 并提取 tactics |
| `dijiang mem finetune` | 运行较慢的 memory fine-tuning 循环 |
| `dijiang template list` | 列出内置和缓存模板 |
| `dijiang template pull <source>` | 从 `gh:owner/repo` 或 URL 拉取模板 |
| `dijiang template validate <path>` | 验证模板 manifest |
| `dijiang skills` | 列出可用 `dj-*` skills |
| `dijiang skills --sync` | 同步项目 `dj-*` skills |
| `dijiang workflow-state --json` | 输出供 hooks/agents 使用的 workflow 状态 |
| `dijiang migrate` | 将 legacy `.trellis/` 状态迁移到 `.dijiang/` |
| `dijiang channel spawn <agent>` | 创建 agent channel |
| `dijiang channel list` | 列出 active channels |
| `dijiang channel send <id> <message>` | 向 channel 发送消息 |
| `dijiang channel execute <id>` | 执行 channel 中的 agent |
| `dijiang channel execute-all` | 并行执行所有 active channels |
| `dijiang channel status <id>` | 检查 channel 状态 |
| `dijiang channel stop <id>` | 停止 channel |
| `dijiang update` | 更新 DiJiang 管理的 skills、agents、prompts、hooks 和 workflow 投影 |
| `dijiang update --from-github` | Refresh global skills from GitHub before updating the project |

## Routing Rules

| Request type | Use |
|--------------|-----|
| New task or unclear request | `dj-dispatch` |
| Requirements alignment; vague feature/optimization or vague bug/fix requests | `dj-grill` |
| Specific feature implementation with concrete object/scope | `dj-implement` or `dj-tdd` |
| Bug or regression | `dj-hunt` |
| Code review / quality gate | `dj-check` |
| Lightweight read-only review | `dj-review` |
| Whole-codebase audit | `dj-audit` |
| Technical debt assessment | `dj-debt` |
| Codebase health report | `dj-health` |
| Documentation / specs | `dj-output` |
| Handoff between sessions | `dj-handoff` |
| Minimal focused changes | `dj-ponytail` |
| Prototype | `dj-prototype` |
| UI design | `dj-design` |
| Script / tool | `dj-script` |
| Pattern research | `dj-pattern` |
| Reasoning / system lens | `dj-reason` |
| Writing polish | `dj-write` |
| Long code discussion | `dj-karpathy` |
| Session findings or lessons | `dijiang mem findings` / `dijiang mem learn` |
