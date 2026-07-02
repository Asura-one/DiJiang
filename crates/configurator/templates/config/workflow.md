# Development Workflow

---

## Core Principles

1. **Plan before code** — align scope before implementation when requirements are unclear.
2. **Specs injected, not remembered** — guidelines are injected via hook/skill, not recalled from memory.
3. **Persist decisions** — task artifacts, findings, lessons, and handoffs are written to `.dijiang/`.
4. **One canonical workflow** — CLI, skills, AGENTS, prompts, and agents are projections of this model.
5. **Verification loop first** — 把目标拆成可证明的命题，先建立 pass/fail 反馈回路，再让实现、审查和记忆围绕它收敛。
6. **Decisions are durable** — 重要取舍写 ADR，记录 why、状态和替代方案；设计文档描述当前形态并引用 ADR。
7. **Compound learning** — AI 造成或发现的问题必须沉淀到 prompt、skill、spec 或 memory，让下一轮工作少犯同类错。
8. **Memory has a quality gate** — 长期记忆必须有 source、scope、confidence、freshness、conflict、actionability；不满足就留在 task artifact。
9. **Git 隔离优先** — 所有会修改代码的任务，修改前都必须创建专用 worktree/branch；任务结束时先做版本决策，再按权限完成提交、push、合并和 worktree 清理。

## DiJiang Canonical Workflow

DiJiang uses `dijiang` CLI for project state and `dj-*` skills for execution capability. `review` is not a canonical task status; quality verification is handled by `dj-check`.

| Task status | Workflow phase | Recommended entry | Output |
|-------------|----------------|-------------------|--------|
| none | dispatch | `dijiang start <name>` or `dj-dispatch` | Active task and routing decision |
| `planning` | align | `dj-grill`, optionally `dj-output` | `prd.md`, optionally `design.md` / `implement.md` |
| `in_progress` | implement | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-script` / `dj-design` | Working code, tests, verification notes |
| `in_progress` | check | `dj-check` | Verified diff and follow-up fixes |
| `completed` | finish | `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | 版本决策、文档/spec 同步证据、范围一致的提交、journal、归档任务、清理当前 session active task |
| `archived` | closed | Read-only, or restart with `dijiang start <task>` | No active work on archived task |
| `paused` | resume | `dijiang-continue` | Context restored, then return to `planning` or `in_progress` |

## Skill Taxonomy

| Category | Skills | Boundary |
|----------|--------|----------|
| Routing | `dj-dispatch` | Classify and route; do not implement directly |
| Alignment | `dj-grill` | Requirements alignment; do not write code |
| Planning docs | `dj-output` | PRD/design/implementation docs and code-doc alignment |
| Implementation | `dj-implement`, `dj-tdd`, `dj-hunt`, `dj-prototype`, `dj-script`, `dj-design` | Write code or investigate root causes |
| Quality gate | `dj-check` | Verify diff quality, completeness, safety, and regressions |
| Review lens | `dj-review` | Lightweight read-only review; does not run tests, edit code, or replace `dj-check` |
| Analysis reports | `dj-audit`, `dj-debt`, `dj-health`, `dj-pattern` | Produce reports; not a default delivery gate |
| Style overlays | `dj-ponytail`, `dj-karpathy` | Add constraints to another workflow path |
| Writing polish | `dj-write` | Polish prose; does not own engineering docs lifecycle |
| Session transfer | `dj-handoff` | Prepare handoff; does not replace finish-work journal |
| Session wrappers | `dijiang-start`, `dijiang-continue`, `dijiang-finish-work` | Load context, route, and close sessions; skill execution is distinct from prompt checklist injection |

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
6. **Push 与集成** — `dijiang finish-work --push --integrate` 是显式动作：push 任务分支、在主分支 worktree 中 `--no-ff` 合并、清理任务 worktree 并删除已合并分支。如果 push/merge 不可执行，保留分支和 worktree，并报告具体阻塞。

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

## CLI Commands

| Command | Description |
|---------|-------------|
| `dijiang init [name]` | Initialize DiJiang project state and platform configuration |
| `dijiang status` | Show project and active task status |
| `dijiang status --compat` | Show compatibility diagnostics |
| `dijiang start <name>` | Create and activate a work session |
| `dijiang dispatch <prompt>` | Create or reuse an active task from a natural-language request and emit route context |
| `dijiang finish-work --verification "..." --docs-sync "..." --version-impact <major/minor/patch/none>` | 在验证、文档/spec 同步证据、版本决策、范围一致的提交/发布决策、journal 记录后完成当前工作并归档 |
| `dijiang task list` | List all tasks |
| `dijiang task current` | Show active task |
| `dijiang task start <name>` | Create or activate a task record with low-level task semantics |
| `dijiang task status <name> <status>` | Update task status |
| `dijiang task archive <name>` | Archive a task |
| `dijiang task prune --days N` | Delete archived tasks older than N days |
| `dijiang mem list` | List platform sessions |
| `dijiang mem sync` | Sync platform sessions to `~/.dijiang/mem/` |
| `dijiang mem findings --finding "..."` | Append project finding |
| `dijiang mem learn --lesson "..."` | Record project lesson |
| `dijiang mem correction --correction "..." --lesson "..." --actionability "..."` | Record a user correction with memory quality metadata |
| `dijiang mem archive` | Archive current memory session |
| `dijiang mem tactic --name N --description D` | Add global tactic |
| `dijiang mem tactics --select N` | List or select tactics with Thompson sampling |
| `dijiang mem record --tactic T --outcome success --context C` | Record tactic outcome |
| `dijiang mem pattern --name N --description D` | Add a project pattern or standard operating procedure |
| `dijiang mem patterns` | List project patterns |
| `dijiang mem stats` | Show memory statistics |
| `dijiang mem backup` | Back up project memory to global storage |
| `dijiang mem evolve` | Analyze session memory and extract tactics |
| `dijiang mem finetune` | Run the slow memory fine-tuning loop |
| `dijiang template list` | List built-in and cached templates |
| `dijiang template pull <source>` | Pull template from `gh:owner/repo` or URL |
| `dijiang template validate <path>` | Validate a template manifest |
| `dijiang skills` | List available `dj-*` skills |
| `dijiang skills --sync` | Sync project `dj-*` skills |
| `dijiang workflow-state --json` | Output workflow state for hooks/agents |
| `dijiang migrate` | Migrate legacy `.trellis/` state to `.dijiang/` |
| `dijiang channel spawn <agent>` | Spawn an agent channel |
| `dijiang channel list` | List active channels |
| `dijiang channel send <id> <message>` | Send message to a channel |
| `dijiang channel execute <id>` | Execute an agent in a channel |
| `dijiang channel execute-all` | Execute all active channels in parallel |
| `dijiang channel status <id>` | Check channel status |
| `dijiang channel stop <id>` | Stop a channel |
| `dijiang update` | Update managed DiJiang skills, agents, prompts, hooks, and workflow projections |
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
| Writing polish | `dj-write` |
| Long code discussion | `dj-karpathy` |
| Session findings or lessons | `dijiang mem findings` / `dijiang mem learn` |
