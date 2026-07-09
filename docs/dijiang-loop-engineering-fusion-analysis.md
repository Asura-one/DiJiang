# DiJiang × Loop Engineering 融合分析报告

> 调研日期：2026-07-07
> 目标：全面分析 cobusgreyling/loop-engineering 的核心架构与设计理念，评估与 DiJiang 的适配性，输出融合方案。

---

## 一、Loop Engineering 项目全景

### 1.1 核心定位

Loop Engineering 是一个**理念框架 + 工具链**项目，核心口号："Stop prompting. Design the loop. Get a score." 它将 AI 编程代理的使用模式从"人工手动写提示"转向"设计自动化控制系统来编排代理"。

### 1.2 五大构建模块 + 记忆

| 原语 | 职责 | DiJiang 对应物 |
|------|------|----------------|
| **Automations/Scheduling** | 以一定节奏触发发现 + 分诊 | `dijiang channel execute-all` / WorkBuddy automation |
| **Worktrees** | 安全的并行执行，每个代理隔离工作目录 | Git Gate + `git_gate.rs` + task worktree 生命周期 |
| **Skills** | 持久化的项目知识（SKILL.md） | `dj-*` skills + `.pi/skills/` |
| **Plugins & Connectors (MCP)** | 接入真实工具（Linear/Jira/Slack/GitHub） | WorkBuddy connectors + `dijiang-mcp` (规划中) |
| **Sub-agents (Maker/Checker)** | 实施者/验证者分离 | `dj-check` (Checker) vs `dj-implement` (Maker) |
| **Memory/State** | 持久化主干（STATE.md / JSON） | `.dijiang/` task artifacts + `dijiang mem` 层级记忆 |

### 1.3 七个生产模式

| 模式 | 节奏 | 风险 | Week 1 | Token 成本 |
|------|------|------|---------|------------|
| Daily Triage | 1d-2h | low | L1 报告 | 低 |
| PR Babysitter | 5m-15m | medium | L1 监控 | 高 |
| CI Sweeper | 5m-15m | medium | L2 谨慎 | 非常高 |
| Dependency Sweeper | 6h-1d | medium | L2 仅补丁 | 中 |
| Changelog Drafter | 1d或按标签 | low | L1 草稿 | 低 |
| Post-Merge Cleanup | 1d-6h | low | L1 非高峰 | 低 |
| Issue Triage | 2h-1d | low | L1 仅建议 | 低 |

### 1.4 L1 → L2 → L3 分阶段推出策略

- **L1 — Report**：只输出报告，不做任何修改
- **L2 — Assisted**：小范围自动修复 + 验证者
- **L3 — Unattended**：无人值守运行（需 budget + denylist + run log + human gates）

### 1.5 核心概念

- **意图债务 (Intent Debt)**：每次 session 代理从零开始，Skills 是偿还意图债务的方式
- **理解债务 (Comprehension Debt)**：快速循环产出你没写的代码 → 理解债务增长
- **认知投降 (Cognitive Surrender)**：让循环跑而自己不再有意见 → 最危险的陷阱
- **编排税 (Orchestration Tax)**：并行代理的人力协调成本
- **Harness vs Loop**：Harness = 单 session 设置；Loop = harness + schedule + state + verification

---

## 二、Loop Engineering 核心实现深度分析

### 2.1 loop-context：断路器 + 状态记忆管理器

这是 loop-engineering 最精炼的实现，12KB 纯 TypeScript，零依赖，确定性逻辑。

**核心数据结构**：

```typescript
interface Ledger { goal: string; attempts: Attempt[]; }
interface Attempt { iteration: number; action: string; outcome: 'success'|'failure'|'noop'; error?: string; tokensUsed?: number; repeated?: number; }
interface CircuitBreakerConfig { maxIterations: number; stagnationThreshold: number; noProgressThreshold: number; tokenBudget?: number; }
```

**核心算法**：

1. **errorSignature**：将错误/堆栈跟踪规范化为稳定签名，忽略行号、地址、时间戳、端口、临时路径等易变细节。这使得"同一错误"可在跨迭代识别。
2. **checkCircuitBreaker**：按优先级检测四种触发条件——停滞(stagnation, 同一错误连续 N 次) > 无进展(no-progress, 连续 N 次失败) > token 预算 > 迭代上限。返回 `BreakerDecision { shouldContinue, escalate, trigger, reason }`。
3. **pruneLedger**：保留最近窗口内的尝试，修剪堆栈跟踪行数，折叠连续相同失败为单条带 repeat count。
4. **summarizeAttempts**：确定性事实汇总——总尝试、成功/失败数、distinct error groups（按频率排序）、已尝试 actions。
5. **buildContextInjection**：生成紧凑上下文块注入下次 prompt——目标 + 进度 + 已尝试列表 + 失败模式 + 最近错误(pruned) + 断路器状态。

**关键设计决策**：
- 所有逻辑是确定性的，无需 LLM 调用 → 成本极低
- Ledger 是 append-only JSON → 可持久化、可审查
- CLI 退出码：0=继续, 2=升级, 1=错误 → 可与任何调度系统集成
- 断路器优先级：最具体最便宜的先检测（停滞 > 无进展 > 预算 > 上限）

### 2.2 loop-audit：循环就绪评分

评分体系基于 20+ 信号的加权求和（满分 100）：

| 信号 | 权重 | DiJiang 对应 |
|------|------|--------------|
| State file | 18 | `.dijiang/tasks/` task.json |
| Triage skill | 14 | `dj-grill` / `dj-dispatch` |
| Verifier skill | 14 | `dj-check` |
| Skills (2+) | 14 | `dj-*` family |
| Agents.md | 9 | `AGENTS.md` (已有) |
| Loop config | 9 | `.dijiang/workflow.md` (已有) |
| GitHub workflows | 4 | CI (可引入) |
| Budget doc | 3 | 尚无 |
| Run log | 3 | 尚无 |
| Constraints | 4 | 尚无 |

**L1/L2/L3 阈值**：L1=38, L2=58, L3=78。L3 需额外条件：verifier + state + cost observability + real loop activity。

### 2.3 loop-mcp-server：MCP 资源与工具暴露

7 个 resources (registry, config, budget, run-log, safety, pattern template, skill template, state template) + 8 个 tools (list/get/recommend/estimate_cost)。

关键特性：
- Agent 按需查询而非 prompt stuffing → 节省 token
- `loop_recommend_pattern`：基于关键词匹配的简单推荐算法
- `loop_estimate_cost`：基于 cadence + level + cost 模型的 token 估算

### 2.4 loop-init / loop-cost / loop-sync

- **loop-init**：脚手架，根据 pattern + tool 生成 STATE.md + LOOP.md + budget + skills
- **loop-cost**：基于 registry.yaml 中每个 pattern 的 noop/report/action token 估算 + cadence 换算日频次
- **loop-sync**：检测 STATE.md 与 LOOP.md 之间的漂移

---

## 三、DiJiang 当前架构概览

### 3.1 核心架构

```
dijiang CLI (Rust) ──┬── task lifecycle (task.json, prd.md, design.md, implement.md)
                      ├── workflow state injection (workflow_state.rs)
                      ├── route gate (route_gate.rs) — 状态机约束
                      ├── git gate (git_gate.rs) — worktree 约束
                      ├── mem (memory.rs, types.rs) — L1-L5 层级记忆
                      └── channel — agent 并行执行

dj-* skills ────────── 原子能力（dispatch, grill, implement, tdd, hunt, check, ...）

dijiang-* skills ───── session 包装器（start, continue, finish-work）

/dijiang-* prompts ──── Pi prompt checklist
```

### 3.2 记忆层级

| 层级 | DiJiang | Loop Engineering |
|------|---------|-----------------|
| L1 — 工作记忆 | task.json + workflow_state | STATE.md |
| L2 — 情景记忆 | findings + learnings + corrections | Ledger (attempts) |
| L3 — 语义记忆 | tactics (Thompson sampling) | loop-context errorSignature + distinctErrors |
| L4 — 程序记忆 | patterns (SOPs) | pattern registry |
| L5 — 元记忆 | evolution + stats + baseline | run log + budget + audit score |

### 3.3 Learning Loop（刚刚实现的闭环）

```
写回：finish-work → SessionClosure → mem evolve → tactics/patterns
读回：workflow_state → load_learned_memory → additional_context → agent 上下文
```

### 3.4 DiJiang 独有的优势

- **Runtime Gate 系统**：route_gate + git_gate 是 CLI 层级的硬约束，不是 prose 建议
- **Bayesian 策略选择**：tactic 的 Thompson sampling，不是简单计数
- **SessionClosure loop_signal**：finish-work 会显式写入 `next_tactic` + `next_pattern` + `loop_signal`，是闭环的写回锚点
- **Workflow 投影一致性**：CLI、skills、AGENTS、prompts 都是同一 workflow 的投影

---

## 四、适配性分析

### 4.1 架构兼容性

| 维度 | 评估 | 说明 |
|------|------|------|
| **哲学一致性** | ★★★★★ | 两者都主张"设计循环而非手动提示"、"验证分离"、"持久化状态"。核心理念高度一致。 |
| **原语映射** | ★★★★☆ | 五大原语几乎一一对应。唯一缺口：DiJiang 没有 scheduling/automation 原语的内建支持（靠 WorkBuddy 外部 automation）。 |
| **数据模型** | ★★★☆☆ | loop-engineering 用 STATE.md + JSON ledger；DiJiang 用 task.json + findings/learnings JSON。数据格式不同但语义可映射。 |
| **执行模式** | ★★★☆☆ | loop-engineering 是"定时循环驱动"；DiJiang 是"任务状态机驱动"。前者是 cron 式，后者是 state machine 式。融合需要折衷。 |

### 4.2 技术栈契合度

| 维度 | 评估 | 说明 |
|------|------|------|
| **语言** | ★★☆☆☆ | loop-engineering 全栈 TypeScript/Node；DiJiang 全栈 Rust。直接引入代码不可能，但**算法和模式**可移植。 |
| **包格式** | ★★☆☆☆ | loop-engineering 用 npm 包（npx 一键用）；DiJiang 用 cargo bin。分发模式完全不同。 |
| **MCP 协议** | ★★★★☆ | 两者都支持 MCP。loop-mcp-server 的 resource/tool 设计可直接映射为 DiJiang 的 MCP connector。 |

### 4.3 功能互补性

| Loop Engineering 功能 | DiJiang 当前状态 | 互补价值 |
|----------------------|-----------------|---------|
| **断路器 (Circuit Breaker)** | **无** | ★★★★★ — DiJiang 的 `dj-hunt` 和 `dj-implement` 都可能陷入无限修复循环。断路器是关键的缺失安全层。 |
| **错误签名 (errorSignature)** | **无** | ★★★★★ — DiJiang 的 findings/learnings 是人工文本，无自动错误归组。errorSignature 可自动聚类重复故障。 |
| **Ledger (Attempt Tracking)** | **弱** | ★★★★☆ — DiJiang 只有 task.json 的状态字段，没有每次尝试的迭代记录。Ledger 可让 `dj-hunt` 记录每次诊断尝试。 |
| **Pruning (上下文修剪)** | **无** | ★★★★☆ — DiJiang 的 workflow_state 注入全量记忆，无窗口化修剪。长 session 时上下文膨胀。 |
| **Loop Audit (评分)** | **无** | ★★★☆☆ — DiJiang 有 `dj-health` 但没有"循环就绪"评分。Audit 可量化项目自动化成熟度。 |
| **Token Cost 估算** | **无** | ★★★☆☆ — 对预算管理有用，但 DiJiang 当前主要在 WorkBuddy 内运行，token 成本由平台管控。 |
| **Pattern Registry** | **弱** | ★★★☆☆ — DiJiang 的 patterns 是项目级 JSON，没有 machine-readable index + cost + risk 元数据。 |
| **Multi-Loop Coordination** | **弱** | ★★☆☆☆ — DiJiang 的 channel 系统可并行但无碰撞检测。多 loop 场景尚少。 |
| **Safety/Denylist** | **部分** | ★★☆☆☆ — DiJiang 的 scope discipline 是 prose 约束，无 runtime enforceable denylist。 |

### 4.4 潜在冲突点

| 冲突 | 严重性 | 说明 |
|------|--------|------|
| **执行模式冲突** | ★★★☆☆ | loop-engineering 的"定时循环"与 DiJiang 的"任务状态机"是不同驱动模型。强行引入 cron 式调度会破坏 DiJiang 的状态一致性。 |
| **State 格式冲突** | ★★★☆☆ | STATE.md 是 Markdown；DiJiang 的 task.json 是结构化 JSON。两个系统不能共享同一 state 文件。 |
| **Skills 定义冲突** | ★★☆☆☆ | loop-engineering 的 Skills 是 SKILL.md + 可选脚本；DiJiang 的 `dj-*` skills 是完整的 workflow + spec 约束。粒度和深度不同。 |
| **MCP 定位冲突** | ★★☆☆☆ | loop-mcp-server 是"按需查询"式；DiJiang 的 workflow_state 是"注入式"。两种 context 供给模式的哲学不同。 |
| **语言栈冲突** | ★★☆☆☆ | TypeScript vs Rust。不能直接引入 npm 包。所有实现必须用 Rust 重写。 |

---

## 五、融合方案

### 5.1 直接引入（移植为 Rust 实现）

#### A. 断路器 (Circuit Breaker) → `dijiang-circuit-breaker`

**最高价值、最急迫的引入。**

```
位置：crates/task/src/circuit_breaker.rs（新增）
依赖：无（纯确定性逻辑，零外部依赖）

数据结构：
- CircuitBreakerConfig { max_iterations, stagnation_threshold, no_progress_threshold, token_budget }
- BreakerDecision { should_continue, escalate, trigger, reason }
- ErrorSignature: fn error_signature(error: &str) -> String

集成点：
1. dj-hunt: 每次诊断尝试后 check_circuit_breaker → 超过 stagnation_threshold 时在 workflow_state 中注入 STOP 信号
2. dj-implement / dj-tdd: 实现循环中记录 attempt → 达到上限时自动 redirect 到 dj-check 或 escalate
3. finish-work: 将 attempt ledger 写入 SessionClosure → mem evolve 可消费
4. workflow_state: 新增 circuit_breaker_status 字段（JSON 可序列化）

算法移植要点：
- errorSignature 的正则替换逻辑直接移植（ISO timestamp → <ts>, hex addr → <addr>, path → basename, :line:col → strip, digits → #）
- checkCircuitBreaker 的优先级检测链直接移植（stagnation > no-progress > token-budget > max-iterations）
- pruneLedger 的窗口化 + 折叠逻辑直接移植

测试：
- 单元测试：error_signature 稳定性、breaker 四种触发、prune 窗口化
- e2e 测试：dj-hunt 陷入循环 → breaker 触发 → workflow_state 显示 escalate
```

#### B. Attempt Ledger → 扩展 `SessionClosure`

```
位置：crates/mem/src/types.rs
改动：
- SessionClosure 增加 attempts: Vec<Attempt> 字段
- Attempt { iteration, action, outcome: Success|Failure|Noop, error: Option<String>, tokens_used: Option<u64> }

集成点：
1. dj-hunt 记录每次诊断 attempt（action="run-test", outcome=Failure, error=...）
2. dj-implement 记录每次实现 attempt
3. finish-work 将 attempts 写入 SessionClosure
4. mem evolve 可消费 attempts 做更精确的模式检测（替代当前的 finding 50字符截取法）

改动量：中等，需改 SessionClosure 结构 + finish-work 写入逻辑
```

#### C. 上下文修剪 → 扩展 `workflow_state`

```
位置：crates/task/src/workflow_state.rs
改动：
- load_recent_memory 增加 pruning 参数（window=5, max_trace_lines=8）
- findings/learnings/corrections 按时间窗口截取
- 错误信息按 errorSignature 归组，折叠重复
- learned_memory 的 tactic 列表按 win_rate 排序后只取 top-5（已有）

集成点：
1. workflow_state.additional_context() 注入 pruned context
2. workflow-state --json 的 memory 字段带 pruned 标记

改动量：小，在现有 format_memory / format_learned_memory 内增加修剪逻辑
```

### 5.2 改造适配（理念引入 + DiJiang 化改造）

#### D. Loop Audit → `dijiang audit`

```
理念引入：循环就绪评分
DiJiang 化改造：

不移植 loop-audit 的 TypeScript 代码。而是定义 DiJiang 的评分维度：

| 信号 | 权重 | 检测方式 |
|------|------|---------|
| 有 .dijiang/ 目录 | 10 | file_exists |
| 有 active task | 18 | task current |
| 有 route gate | 14 | route_gate.rs 已编译 |
| 有 git gate | 14 | git_gate.rs 已编译 |
| 有 2+ dj-* skills | 14 | dijiang skills |
| 有 dj-check (verifier) | 14 | skill list |
| 有 AGENTS.md | 9 | file_exists |
| 有 workflow.md | 9 | file_exists |
| 有 tactic 记录 | 6 | mem stats |
| 有 pattern 记录 | 6 | mem stats |
| 有 circuit breaker | 6 | (新增后) |
| 有 run log | 3 | (新增后) |
| 有 budget | 3 | (新增后) |

CLI 命令：dijiang audit [--suggest] [--badge]
输出：Loop Readiness Score (0-100) + L0/L1/L2/L3 等级 + 建议

实现路径：
1. 新增 crates/cli/src/cmd_audit.rs
2. 评分逻辑纯 Rust，无外部依赖
3. 可选 badge 生成（SVG）
```

#### E. Pattern Registry → 扩展 `dijiang mem pattern`

```
理念引入：machine-readable pattern index with cost/risk metadata
DiJiang 化改造：

当前 dijiang mem pattern 只记录 { name, description, steps, tags, project }。
扩展为：

Pattern {
  name: String,
  description: String,
  steps: Vec<String>,
  tags: Vec<String>,
  project: Option<String>,
  // 新增
  cadence: Option<String>,        // "1d", "15m" 等
  risk: Option<String>,           // "low", "medium", "high"
  week_one_mode: Option<String>,  // "L1", "L2"
  token_cost: Option<String>,     // "low", "medium", "high"
  human_gates: Vec<String>,       // 需要人工审批的场景
  phases: Vec<String>,            // discover, triage, fix, verify, notify
}

CLI 扩展：
- dijiang mem pattern --registry → 输出 YAML/JSON 索引
- dijiang mem recommend --use-case "watch CI" → 推荐 pattern

这不需要移植 loop-engineering 的 registry.yaml，而是让 DiJiang 自己的 pattern 系统进化出同等元数据。
```

#### F. Token Cost 估算 → `dijiang cost`

```
理念引入：预估每日 token 开销
DiJiang 化改造：

DiJiang 运行在 WorkBuddy 内，token 成本由平台管控。但仍然可以：
1. 基于 workflow_state 的 injection_count + skill 数量估算单次 session 的 token
2. 基于 automation cadence 估算日开销
3. 在 workflow_state 中增加 token_budget 字段

CLI 命令：dijiang cost [--pattern <name>] [--level L1/L2/L3]
实现：纯 Rust 估算逻辑，从 workflow_state 读取参数
```

### 5.3 不适合引入及原因

#### G. STATE.md 文件格式 — **不引入**

```
原因：
1. DiJiang 用结构化 JSON (task.json) 而非 Markdown，已有更严谨的数据模型
2. STATE.md 的 "一个文件存所有状态" 设计在多 loop 时会冲突（loop-engineering 自己也承认这点）
3. DiJiang 的 .dijiang/tasks/ 目录结构比单文件更可扩展
4. 引入 STATE.md 会要求 agent 同时维护两套状态 → 双写问题

替代：保持 DiJiang 的 task.json + workflow_state JSON，让 loop-engineering 的
理念通过 Rust 数据结构表达，而非引入其文件格式。
```

#### H. npm 包分发模式 — **不引入**

```
原因：
1. DiJiang 是 Rust 项目，用 cargo bin 分发
2. npx 一键用的便利性在 DiJiang 场景下由 WorkBuddy 平台提供
3. 引入 npm 包会要求项目同时维护 Node + Rust 两套工具链

替代：所有实现用 Rust 重写，通过 cargo install dijiang 或 WorkBuddy 内置分发。
```

#### I. loop-sync (漂移检测) — **不引入**

```
原因：
1. loop-sync 检测 STATE.md 与 LOOP.md 的漂移，但 DiJiang 不用 STATE.md
2. DiJiang 的 workflow.md 是规范投影（AGENTS.md 引用它），不是运行时状态
3. 漂移检测在 DiJiang 场景下应由 route_gate/git_gate 在 runtime 层级检测

替代：route_gate 已经是 runtime 层级的状态机约束，比文本漂移检测更可靠。
```

#### J. 多 Loop 碰撞检测 — **暂不引入**

```
原因：
1. DiJiang 的 channel 系统已支持并行执行，但当前使用场景主要是单任务顺序流
2. 多 loop 碰撞是 L3 无人值守阶段的问题，DiJiang 当前还在 L1-L2
3. 引入碰撞检测需要 "acting_on" 锁定机制 → 需要持久化锁 → 复杂度不合当前阶段

替代：在到达 L3 阶段后再引入，基于 circuit_breaker 的 escalate 机制 + channel 的 stop 命令。
```

#### K. Sub-agent Maker/Checker 分离 — **不引入其实现，理念已内建**

```
原因：
1. DiJiang 已有 dj-implement (Maker) + dj-check (Checker) 的分离
2. loop-engineering 的实现依赖特定 agent 平台的 sub-agent API（Grok isolation, Claude Code agents）
3. DiJiang 的 route_gate 已在 runtime 层级约束 Maker 和 Checker 不能混淆

替代：保持现有 dj-check 独立验证模式，不需引入 loop-engineering 的平台特定 sub-agent 机制。
```

---

## 六、实施路线图

### Phase 1：安全层（最高优先级）✅ 已完成

| 步骤 | 内容 | 预估 | 状态 |
|------|------|------|------|
| 1.1 | `circuit_breaker.rs`：error_signature + check_circuit_breaker + prune_ledger | 2-3 天 | ✅ `crates/task/src/circuit_breaker.rs` |
| 1.2 | `Attempt` 类型扩展 SessionClosure | 1 天 | ✅ `crates/mem/src/types.rs` |
| 1.3 | workflow_state 上下文修剪 | 1 天 | ✅ windowed/top-5 pruning |
| 1.4 | e2e 测试：hunt 循环 → breaker 触发 | 1 天 | ✅ 2 e2e tests |

### Phase 2：评分与可观测性 ✅ 已完成

| 步骤 | 内容 | 预估 | 状态 |
|------|------|------|------|
| 2.1 | `dijiang audit` 命令 + 评分逻辑 | 2 天 | ✅ `dijiang audit [--suggest] [--badge]` |
| 2.2 | Pattern 元数据扩展 (cadence, risk, human_gates) | 1 天 | ✅ 6 new fields on Pattern |
| 2.3 | Run log 结构化 (loop-run-log.json) | 1 天 | ✅ audit detects loop-run-log.json |

### Phase 3：MCP 与自动化 ✅ 已完成

| 步骤 | 内容 | 预估 | 状态 |
|------|------|------|------|
| 3.1 | DiJiang MCP server（暴露 workflow_state + patterns + tactics 为 MCP resources） | 3 天 | ✅ `crates/mcp-server/` — 4 resources + 5 tools |
| 3.2 | `dijiang cost` 估算命令 | 1 天 | ✅ `dijiang cost [--pattern] [--level]` |
| 3.3 | `dijiang mem recommend` pattern 推荐 | 1 天 | ✅ `dijiang mem recommend --use-case [--registry]` |

### Phase 4：L3 准备（远期）⏳ 尚未实施

| 步骤 | 内容 | 预估 |
|------|------|------|
| 4.1 | Budget 文件 + kill switch | 2 天 |
| 4.2 | Path denylist runtime enforcement | 2 天 |
| 4.3 | 多 loop 碰撞检测 | 3 天 |

---

## 七、可复用设计模式提炼

从 loop-engineering 中提炼的**不依赖语言栈**的核心设计模式：

### 7.1 断路器优先级链模式

```
检测顺序：最具体最便宜 → 最通用最昂贵
stagnation (同错误 N 次) > no-progress (连续失败 N 次) > token-budget (花费上限) > max-iterations (迭代上限)

原则：当多个条件同时满足时，报告最可操作的触发原因。
```

### 7.2 错误签名归一化模式

```
将运行时错误转化为稳定标识符：
- 时间戳 → <ts>
- 内存地址 → <addr>
- 文件路径 → basename
- 行号 → 删除
- 数字 → #

用途：跨迭代识别"本质相同的错误"，避免在表面不同的错误上重复浪费。
```

### 7.3 窗口化 + 折叠修剪模式

```
上下文注入前：
1. 只保留最近 N 次尝试（窗口化）
2. 连续相同失败的尝试折叠为单条 + repeat count
3. 堆栈跟踪截断到前 M 行 + "… X more lines pruned"

原则：代理需要知道"做了什么 + 最近的错误"，不需要完整历史。
```

### 7.4 L1 → L2 → L3 渐进推出模式

```
L1 = 只报告（验证 triage 准确性）
L2 = 小范围自动操作 + 验证者
L3 = 无人值守（需 budget + denylist + run log + human gates + proven L2 历史）

原则：永远不要跳过 L1。自动化成熟度是渐进赢得的，不是一次性设计的。
```

### 7.5 按需查询 vs 全量注入模式

```
loop-mcp-server 让 agent 按需查询 pattern/skill/state
vs DiJiang workflow_state 全量注入到 context

折衷：高优先级信息（task status, circuit breaker）注入；低优先级信息（pattern docs, cost estimates）按需查询。
```

### 7.6 Maker/Checker 必分离原则

```
实现者不能评判自己的工作。
- 不同 agent
- 不同 instructions（Verifier 默认立场：REJECT）
- 更强模型 on verifier（L3 场景）

DiJiang 已内建：dj-implement ≠ dj-check。
```

---

## 八、总结

| 引入类别 | 数量 | 价值 |
|----------|------|------|
| **直接引入**（Rust 重写） | 3 | 断路器 + Ledger + 修剪 = 安全核心 |
| **改造适配** | 3 | Audit + Pattern 元数据 + Cost 估算 = 可观测性 |
| **不引入** | 5 | STATE.md/npm/loop-sync/碰撞检测/sub-agent实现 = 架构不合或已有替代 |

**最高 ROI 的单点引入**：`circuit_breaker.rs`。它用约 500 行 Rust 就能移植 loop-engineering 最精炼的安全机制，填补 DiJiang 当前最大的空白——防止代理陷入无限循环。

**融合后的 DiJiang 架构一致性保持**：所有引入都走 DiJiang 的 CLI → task/mem crate → workflow_state 通道，不引入新的文件格式、外部包依赖或执行模式。loop-engineering 的精华以 Rust 数据结构和算法的形式内化，而非以 npm 包的形式外挂。

---

## 九、实施状态更新（2026-07-08）

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 1：安全层 | ✅ **已完成** | circuit_breaker.rs + SessionClosure + pruning + e2e |
| Phase 2：评分与可观测性 | ✅ **已完成** | audit + pattern 元数据 + run log |
| Phase 3：MCP 与自动化 | ✅ **已完成** | MCP server + cost + recommend |
| Phase 4：L3 准备 | ⏳ **远期** | Budget / denylist / collision detection 尚未实施 |

**实施总结**：该融合方案中计划实施的 10 个子项（Phase 1×4 + Phase 2×3 + Phase 3×3）已全部完成，对应代码变更约 2000+ 行 Rust，覆盖 3 个原有 crate 和 1 个新 crate (`crates/mcp-server/`)。Phase 4 的 3 个子项标注为"远期"，未纳入本次实施范围。
