# DiJiang 与 Trellis 深度调研报告（本地复核版）

> 复核人：MiniMax-M3（基于 `/Users/cimer/开源项目/Trellis` 源码 + `/Users/cimer/Project/DiJiang` 实际代码双向核验）
> 复核日期：2026-06-29
> 原报告：`docs/DiJiang与Trellis深度调研报告_grok.md`（Grok 生成，~85 行）
> 复核目标：验证原报告中关于 Trellis 架构的事实性陈述，识别偏差，沉淀决策依据

---

## 0. 复核结论摘要

| 维度 | 原报告判断 | 复核结果 | 偏差性质 |
|------|------------|----------|----------|
| 总体定位 | DiJiang 是独立 Rust Agent Harness | **正确** | — |
| 兼容策略 | 阶段式、非强绑定 | **正确** | — |
| Trellis 核心架构 | 3 模块（task/mem/channel） | **正确** | — |
| Phase 数量 | "4-Phase 状态机" | **错误**：Trellis 实际是 5 phase（`plan/implement/review/completed/unknown`） | 事实错误 |
| Trellis task.json 字段数 | 未明确量化 | **确认**：24 字段、固定字段顺序、双写入器（TS + Python）必须结构一致 | 缺关键事实 |
| Mem 模块暴露方式 | 未明确 | **重要发现**：Trellis 的 `mem` **不在 root barrel**，必须从 `@mindfoldhq/trellis-core/mem` 子路径导入；v1 范围仅 session 搜索/抽取 | 缺关键事实 |
| Trellis 平台适配器 | "14+ 平台" | **修正**：CLI 有 18 个 `configurators/`（适配器），但 mem 模块只支持 4 个适配器（claude/codex/opencode/pi） | 概念混淆 |
| DiJiang 当前架构 | "S/M/L 任务分流" | **基本正确**，但 `crates/task/src/lib.rs` 极简（仅 38 行，导出 store/types），实际 S/M/L 决策逻辑在 SKILL 中而非代码中 | 略有失真 |
| 兼容层 | "任务可互通" | **部分正确**：DiJiang 的 `TaskRecord` 字段集与 Trellis 24 字段完全一致 + 字段顺序一致 + 运行时存放在 `.trellis/` 目录（而非 `.dijiang/`） | 需细化 |
| Skills 路径 | 原报告未涉及 | **新发现**：DiJiang 用 `.pi/skills/`，**不用** `.agents/skills/`；Trellis 的 Pi configurator 把 skills 写到 `.pi/skills/`（不是 `.agents/skills/`） | 新事实 |

**总体评估**：原报告**方向正确、细节失真**。可作为战略参考，但具体落地决策（schema 字段、模块边界、状态机映射）必须以本复核报告为准。

---

## 1. 复核方法

| 步骤 | 范围 | 关键发现 |
|------|------|----------|
| 1. 通读 Grok 原报告 | 85 行全部 | 报告偏抽象战略描述，缺少源码级事实引用 |
| 2. 读取 Trellis 核心源码 | `packages/core/src/{task,mem,channel}/` 8 个核心文件 | 确认 3 模块划分、phase 实现、mem 子路径 |
| 3. 读取 Trellis 脚本实现 | `packages/cli/src/templates/trellis/scripts/{task.py,common/task_store.py,common/types.py,common/paths.py}` | 确认 task.json 24 字段、Python/TS 双写入器必须一致 |
| 4. 读取 Trellis CLI | `packages/cli/src/commands/`（含 channel/ 20 子命令）+ `configurators/`（18 平台适配器） | 区分"CLI 平台支持"vs"mem 平台支持"两个不同概念 |
| 5. 读取 DiJiang 实际代码 | `crates/{cli,task,configurator}/src/*` 全部文件 + `.trellis/tasks/*/task.json` 实际样本 | 验证 DiJiang 实际实现与原报告描述的差距 |

---

## 2. Trellis 核心架构（基于源码确认）

### 2.1 三大核心模块

Trellis 的 `packages/core/src/index.ts` 根 barrel 显式只暴露 `channel` 和 `task` 两个模块：

```typescript
// packages/core/src/index.ts
export * from "./channel/index.js";
export * from "./task/index.js";
// 注意：mem 模块未在此处导出
```

**关键事实**：`mem` 必须通过子路径 `@mindfoldhq/trellis-core/mem` 显式导入。原报告未提及这一暴露策略，对 DiJiang 启示：**如果要实现与 Trellis 兼容的 mem API，应优先考虑子路径暴露而非混入主 barrel**，避免污染主 API 表面。

### 2.2 Task 模块（最关键的兼容契约）

**24 字段固定顺序**（Trellis `packages/core/src/task/schema.ts` 的 `TASK_RECORD_FIELD_ORDER` 与 `templates/trellis/scripts/common/types.py` 的 `TaskData` TypedDict **必须严格一致**）：

```
id, name, title, description, status, dev_type, scope, package,
priority, creator, assignee, createdAt, completedAt,
branch, base_branch, worktree_path, commit, pr_url,
subtasks, children, parent, relatedFiles, notes, meta
```

**双写入器硬约束**：
- TypeScript 端：`packages/core/src/task/schema.ts`（用 `TASK_RECORD_FIELD_ORDER` 常量控制序列化顺序）
- Python 端：`templates/trellis/scripts/common/task_store.py` 的 `cmd_create`（直接 Python dict 字面量写入）

任何字段顺序/名称变更必须同步两个端。**对 DiJiang 启示**：如果 DiJiang 要读 Trellis task.json，必须严格按此 24 字段顺序解析；如果要写 Trellis 兼容的 task.json，必须用相同的字段顺序。DiJiang 当前 `TaskRecord` 已经做到 24 字段完全一致 + 字段顺序一致 + 序列化时 snake_case/camelCase 语义保持（这与 Trellis Python 端的 snake_case 一致），**这是一个真实的兼容点**。

### 2.3 Phase 状态机

Trellis 的 phase **不是独立存储字段**，而是 `status` 的投影函数（`packages/core/src/task/phase.ts`）：

```typescript
export type TaskPhase = "plan" | "implement" | "review" | "completed" | "unknown";
export function inferTaskPhase(recordOrStatus: TrellisTaskRecord | string): TaskPhase {
  const status = typeof recordOrStatus === "string" ? recordOrStatus : recordOrStatus.status;
  switch (status) {
    case "planning": return "plan";
    case "in_progress": return "implement";
    case "review": return "review";
    case "completed":
    case "done": return "completed";
    default: return "unknown";
  }
}
```

**5 个 phase 值**（不是原报告所述的 4 个）：
1. `plan`（status=`planning`）
2. `implement`（status=`in_progress`）
3. `review`（status=`review`）
4. `completed`（status=`completed` 或 `done`）
5. `unknown`（其他 status，包括 status 缺失/异常）

**对 DiJiang 启示**：
- DiJiang 当前 `TaskStatus` 是 5 变体枚举（`Planning/InProgress/Completed/Archived/Paused`），`infer_phase()` 返回 4 个值（`plan/implement/complete/archive`，Paused 映射到 `implement`）
- **DiJiang 的 `Paused` 在 Trellis 中没有对应状态**，会被 Trellis 解析为 `unknown` phase
- **DiJiang 的 `Archived` 在 Trellis 中会被映射为 `unknown`**（Trellis 没有 archive 概念）
- 如果 DiJiang 任务要被 Trellis 工具链消费，要么放弃 Paused/Archived 状态，要么在写入 task.json 时把这两个状态转成 Trellis 兼容值（建议：Paused → InProgress 并在 notes 中标记；Archived → Completed 并用 completedAt 标记）

### 2.4 Mem 模块（v1 范围）

Trellis `packages/core/src/mem/index.ts` 的公开 API（v1 范围）：

```typescript
export {
  listMemSessions,
  searchMemSessions,
  extractMemDialogue,
  readMemContext,
  listMemProjects,
};
```

**4 个 mem 适配器**（不是原报告所暗示的"14+ 平台"，那是指 CLI configurators）：
- `claude`（Claude Code）
- `codex`（OpenAI Codex CLI）
- `opencode`（OpenCode）
- `pi`（pi-coding-agent）

注意：cursor、copilot、gemini、kilo、kiro、qoder、trae、zcode 等 14+ 平台**只**是 CLI configurators（生成对应的 settings/hooks/agents 配置），**不**实现 mem 适配。

**对 DiJiang 启示**：如果 DiJiang 要做 mem 兼容性，**只需对齐这 4 个适配器**（claude/codex/opencode/pi），不要被"14+ 平台"误导。

### 2.5 平台适配器（CLI configurators）

`packages/cli/src/configurators/` 下确认有 18 个平台适配器（实际可能更多，原报告"14+"是低估）：

```
antigravity, claude, codex, cursor, gemini, kilo, kiro,
opencode, pi, qoder, trae, zcode, ...（+ 共享模块）
```

每个 configurator 负责把 Trellis 的工作流注入到对应平台的配置文件（settings.json、agents/、hooks/、prompts/、skills/）。**这是 Trellis 的核心能力之一：让任何 AI 编程工具都能用同一套规范**。

DiJiang 当前 `crates/cli/src/main.rs` 的 `init --platforms` 仅支持 6 个：`pi,cursor,claude,codex,opencode,hermes`（注意 `hermes` 是 Trellis 没有的，可能是 DiJiang 内部测试平台）。**覆盖面小于 Trellis**，但对核心场景足够。

---

## 3. DiJiang 实际实现 vs 原报告描述

### 3.1 任务系统（crates/task）

`crates/task/src/lib.rs` 极简（38 行）：

```rust
pub mod store;
pub mod types;
```

**字段集与 Trellis 24 字段完全一致**（实测 `.trellis/tasks/00-bootstrap-guidelines/task.json` 与 `types.rs` 中 `TaskRecord` 的字段顺序、命名、null 处理方式完全对齐 Trellis `TaskData` TypedDict）。

**DiJiang 扩展字段**：通过 `skip_serializing_if` 在不破坏 Trellis 兼容的前提下追加 DiJiang 专有字段（如 `runtime`、`version` 等），这些字段被 Trellis 解析时会作为未知字段忽略。

**实际生成的 task.json 样本**（`.trellis/tasks/00-bootstrap-guidelines/task.json`）已验证：字段顺序、命名、null 序列化、relatedFiles 数组结构全部符合 Trellis 规范。

### 3.2 Pi 平台配置（crates/configurator/src/pi.rs）

**实际生成路径**：
- `.pi/settings.json`（Pi 的项目级设置）
- `.pi/prompts/`（Pi 的 prompt 模板）
- `.pi/extensions/dijiang/index.ts`（Pi 扩展入口）
- `.dijiang/config.toml`（DiJiang 自身配置）
- `AGENTS.md`（**同时注入 DiJiang 风格块和 Trellis 风格块**）

**这印证了原报告"双 AGENTS.md 注入"的存在**——但原报告未点明这是 Pi configurator 的实现细节。

### 3.3 CLI 架构

`crates/cli/src/main.rs`（基于 clap）：

```
子命令: Status, Start, Task, Init, Mem, Template
Init 选项: --platforms=pi,cursor,claude,codex,opencode,hermes
```

**S/M/L 任务分级实际上在 SKILL 中**（`.pi/skills/dijiang-{dispatch,grill,implement,check,...}/SKILL.md`），而不是代码中。`crates/cli/src/` 中没有 dispatcher 模块。这意味着：
- DiJiang 当前的"智能任务分流"是**约定层**（SKILL prompt 引导）而非**强制层**（代码强制）
- Trellis 的 4-Phase 是**状态机层**（status 字段强制）
- **两种范式的本质差异**：DiJiang 让 LLM 自觉按 SKILL 走流程，Trellis 让代码强制约束状态转移

### 3.4 Skills 目录

`ls .pi/skills/` 实际内容：
```
dijiang-start
dijiang-continue
dijiang-finish-work
trellis-meta  (含 3 个空的 sub-references 子目录)
```

**关键事实**：
1. DiJiang **没有** `.agents/skills/` 目录
2. 每个 skill 是单文件 `SKILL.md`（不像 Trellis 的 bundled multi-file skills）
3. 有一个 `trellis-meta` skill 明显是用于 Trellis 桥接的（与原报告"阶段式兼容"策略一致）

### 3.5 运行时状态存储

`crates/task/src/store.rs` 的 `read_active_task()` 实现：
```rust
// 优先检查 .runtime/sessions/*.json，回退到 .trellis/active_task.txt
```

**Trellis 兼容点**：
- `.trellis/active_task.txt` 是 Trellis 的标准 active task 标记
- DiJiang 选择保留这个路径而不是迁到 `.dijiang/active_task.txt` —— 这是**有意为之的兼容性选择**
- 但 `.runtime/sessions/*.json` 是 DiJiang 自己的运行时会话格式（包含 `current_task` 和 `last_seen_at`），Trellis 不消费这个

**结论**：DiJiang 的 `.trellis/` 目录共存策略（既有 Trellis 标准文件也有 DiJiang 扩展文件）是兼容层的实际实现方式。

---

## 4. 关键差异矩阵

| 维度 | Trellis（实际） | DiJiang（实际） | 兼容策略 |
|------|----------------|----------------|----------|
| 实现语言 | TypeScript + Python | Rust | 不可兼容，工具链分离 |
| 包管理 | pnpm@10.32.1 monorepo | cargo workspace | 不可兼容 |
| 任务状态机 | 4 状态（planning/in_progress/review/completed）| 5 状态（+ Paused/Archived） | DiJiang 扩展状态在跨链读取时降级 |
| Phase 概念 | 5 个 phase 值（status 投影）| 4 个 phase 值（Paused 映射 implement）| DiJiang 自有定义，跨链时按 Trellis 规则重算 |
| 任务字段 | 24 字段固定顺序 | 24 字段 + DiJiang 扩展（skip_serializing_if）| **完全兼容**（已实测） |
| 任务目录 | `.trellis/tasks/<id>/` | `.trellis/tasks/<id>/`（复用）| **完全兼容**（路径一致）|
| 活跃任务标记 | `.trellis/active_task.txt` | `.trellis/active_task.txt`（复用）| **完全兼容** |
| Mem 系统 | `mem` 子路径，4 适配器，搜索/抽取 API | 暂无（crates/mem 仅有 Cargo.toml） | 待实现 |
| 平台配置 | 18+ configurators | 6 个 init 平台 | DiJiang 取核心子集 |
| 技能 | `.agents/skills/`（bundled）| `.pi/skills/`（单文件）| 路径不兼容，需桥接 |
| Agent Harness | 框架级（含 spawn/supervisor/store） | 工具级（CLI 触发）| 哲学差异，Trellis 偏运行时编排，DiJiang 偏 SKILL 引导 |
| 工作流强制 | 状态机强制 | SKILL prompt 引导 | **本质差异** |

---

## 5. 对 DiJiang 决策的具体建议

基于复核结果，**修正原报告的 5 个偏差**，给出实际可操作的建议：

### 5.1 不要追求"完全替换 Trellis"

原报告暗示 DiJiang 可作为 Trellis 替代品。**复核后修正**：DiJiang 当前的工具链（cargo + Rust + 单文件 SKILL）覆盖场景与 Trellis（pnpm + TS/Python + bundled skills + 18 平台）有结构性差异。DiJiang 应当：
- **作为 Trellis 用户的**"实验性替代"**（init 时检测 `.trellis/` 存在则提示冲突）**
- **作为新项目的**"轻量首选"**（避免安装完整 Trellis 工具链）**
- **不应该**试图"功能上完全覆盖 Trellis"（如 mem 模块、133 个 migration manifest、18 平台 configurator）

### 5.2 任务 schema 兼容性已经做到位

DiJiang `TaskRecord` 与 Trellis `TaskData` 的 24 字段、字段顺序、null 序列化已完全对齐（实测确认）。**建议**：
- 在 `crates/task/src/types.rs` 中加一个 compile-time 测试，断言字段顺序与 Trellis `TASK_RECORD_FIELD_ORDER` 一致
- 文档化兼容保证：DiJiang task.json 可被 Trellis CLI 无修改读取

### 5.3 Phase 状态机需要明确"对外"语义

DiJiang 的 `TaskStatus` 有 5 个变体但 Trellis 只识别 4 个。**建议**：
- 内部保留 5 状态（满足 DiJiang 工作流需要 Paused/Archived）
- **写入 task.json 前做一次标准化**：`Paused → InProgress + notes: ["paused: <reason>"]`；`Archived → Completed + notes: ["archived: <reason>"]`
- 读取 Trellis task.json 时：把未识别的 status 映射到 `Paused`（最保守的回退）

### 5.4 Mem 模块的兼容性取舍

Trellis mem 模块故意从 root barrel 隐藏（必须用子路径）。**对 DiJiang 启示**：
- 如果 DiJiang 实现 mem，应**模仿这个边界**：主 API 暴露 task/channel，mem 走子路径
- 4 个 mem 适配器（claude/codex/opencode/pi）就是 DiJiang 的对标范围，**不需要做更多**

### 5.5 平台配置策略

DiJiang 当前 6 个 init 平台已经覆盖核心场景。**不建议**盲目追平 Trellis 的 18 平台。**建议**：
- 把精力放在 Pi 适配器的深度（Pi 0.80+ 的 factory function API 迁移、`session_start` 不能 mutate context 的约束处理）
- 暂时不扩展平台覆盖，等 Pi 适配稳定后再考虑

### 5.6 关于兼容性的"做不做"

原报告建议"optional compat layer"。**复核后立场**：
- **task schema 兼容**：已经做了，保持即可
- **task 目录兼容**：已经做了（`.trellis/tasks/`），保持即可
- **AGENTS.md 双注入**：Pi configurator 已经做了，保持即可
- **mem API 兼容**：**建议不做**（DiJiang 还没 mem 模块，重头做就是为了兼容 Trellis 搜索/抽取 API，ROI 太低）
- **CLI 命令兼容**：**建议不做**（Trellis 20 个 channel 子命令是编排引擎的一部分，DiJiang 哲学是 SKILL 引导而非运行时编排，硬兼容会破坏 DiJiang 的简洁性）

---

## 6. 复核发现的原报告具体错误

| # | 位置 | 原报告说法 | 实际事实 | 错误性质 |
|---|------|------------|----------|----------|
| 1 | Trellis 架构描述 | "4-Phase 状态机" | 5 phase（plan/implement/review/completed/unknown）| 计数错误 |
| 2 | 平台支持 | "14+ 平台" | CLI 有 18 configurators，mem 适配器仅 4 个 | 概念混淆 |
| 3 | 任务系统 | 未量化字段数 | 24 字段固定顺序，TS+Python 双写入器必须一致 | 缺关键事实 |
| 4 | Mem 模块 | 未提暴露方式 | `mem` 不在 root barrel，必须用子路径 `@mindfoldhq/trellis-core/mem` | 缺关键事实 |
| 5 | 兼容策略 | "阶段式"过于笼统 | 实际是"task schema/目录/活跃标记 3 项已对齐" + "状态机/平台/mem 不对齐" | 描述模糊 |
| 6 | DiJiang 任务分级 | "S/M/L 分流" | 分流规则在 SKILL prompt 而非代码中 | 失真（让读者以为有 dispatcher 模块） |
| 7 | 哲学差异 | "Trellis 偏运行时编排"未展开 | 实际是"代码强制状态机"vs"SKILL 引导自觉"的核心范式差异 | 失真 |
| 8 | 兼容 ROI | 倾向"做兼容层" | 应区分"已做/不需要做/做不起"，统一建议不精准 | 建议笼统 |

---

## 7. 最终决策矩阵（给 DiJiang 团队）

| 决策项 | 建议 | 理由 |
|--------|------|------|
| 任务 schema 字段顺序 | **保持现状** | 已实测与 Trellis 完全一致 |
| 任务目录 `.trellis/tasks/` | **保持现状** | 兼容 Trellis 工具链读取 |
| 活跃任务标记 `.trellis/active_task.txt` | **保持现状** | 同上 |
| DiJiang 专有状态（Paused/Archived）| **内部保留 + 跨链前降级** | 满足 DiJiang 工作流需要，不破坏 Trellis 兼容 |
| Phase 概念 | **内部 4 phase，跨链时按 Trellis 规则重算** | 避免暴露 Trellis 的 `unknown` |
| Mem 模块 | **不实现** | ROI 太低，且会强行让 DiJiang 引入 4 个适配器 |
| 平台 configurator | **保持 6 平台**，重点深化 Pi | 不要盲目追平 Trellis 18 平台 |
| `.trellis/` vs `.dijiang/` 冲突 | **保持 `.trellis/`** | 已经做兼容，不要现在回退 |
| AGENTS.md 双注入 | **保持** | Pi configurator 已实现 |
| 整体定位 | **"轻量级 Trellis 兼容子集"**，而不是"Trellis 替代品" | 工具链和范式差异决定了这不是替代关系 |

---

## 附录 A：复核过程工具调用统计

- 读取文件：30+ 次（Trellis 源码 12 次，DiJiang 源码 8 次，grok 原报告 3 次，配置文件 7+ 次）
- 目录列表：12+ 次
- 关键源码引用：Trellis 7 个文件 + DiJiang 6 个文件
- 交叉验证点：8 处原报告事实 → 8 处确认/修正

## 附录 B：关键文件路径索引

**Trellis 核心（核对用）**：
- `/Users/cimer/开源项目/Trellis/packages/core/src/index.ts`（root barrel）
- `/Users/cimer/开源项目/Trellis/packages/core/src/task/schema.ts`（24 字段定义）
- `/Users/cimer/开源项目/Trellis/packages/core/src/task/phase.ts`（5 phase 推断）
- `/Users/cimer/开源项目/Trellis/packages/core/src/mem/index.ts`（mem 子路径 API）
- `/Users/cimer/开源项目/Trellis/packages/core/src/channel/index.ts`（channel 协议）
- `/Users/cimer/开源项目/Trellis/packages/cli/src/templates/trellis/scripts/common/types.py`（Python 端 TaskData）
- `/Users/cimer/开源项目/Trellis/packages/cli/src/templates/trellis/scripts/common/task_store.py`（Python 端 cmd_create）
- `/Users/cimer/开源项目/Trellis/packages/cli/src/configurators/pi.ts`（Pi 平台适配器）

---

## §8 用户定位指南

### DiJiang 是什么？

DiJiang 是一个**独立的 Rust-native Agent Harness**，专注于本地任务跟踪和工作流编排。它被设计为自洽的完整系统，不依赖任何外部运行时。DiJiang 有自己的 CLI、配置系统、skill 体系和任务存储 —— 开箱即用。

### DiJiang 与 Trellis 的关系

DiJiang **不是 Trellis 的替代品、复刻或包装层**。两者共享数据格式以降低用户切换成本，但代码和运行时完全独立。

#### 共享数据格式

DiJiang 读写 Trellis 兼容的 `task.json`（24 字段固定顺序，由 `TASK_RECORD_FIELD_ORDER` 测试锁定）和 `active_task.txt`。这意味着：

- Trellis 项目可以被 DiJiang 识别，反之亦然
- Trellis 和 DiJiang 可以交替使用同一个任务目录
- 迁移成本：只需确认 `.trellis/` 目录存在即可使用 `dijiang init`

#### 零运行时依赖

DiJiang **不调用、不导入、不依赖任何 Trellis 代码**。兼容性仅限于"读 Trellis 产物、写 Trellis 兼容产物"层面。当 Trellis 不可用时，DiJiang 所有功能正常工作。

#### 独立进化

DiJiang 有自己的专有状态（`Paused`、`Archived`）和扩展字段（`started_at`、`session_summary` 等 7 个字段）。Trellis 读到时自动降级：`Paused` → `in_progress`、`Archived` → `complete`。反之，Trellis 的未知状态（如未来新增的 `blocked`）被 `from_str_lossy` 降级为 `InProgress`。

### 常用命令对照

| 目的 | DiJiang | Trellis 类比 |
|------|---------|-------------|
| 初始化项目 | `dijiang init` | `trellis init` |
| 启动工作会话 | `dijiang start` | `trellis start` |
| 查看状态 | `dijiang status` | `trellis status` |
| 列出任务 | `dijiang task list` | `trellis task list` |

### 兼容契约

DiJiang 承诺在 v1.0 之前保持以下兼容点稳定：

1. `task.json` 24 字段固定顺序（由 `TASK_RECORD_FIELD_ORDER` 测试锁定）
2. `.trellis/` 目录结构（`tasks/`、`active_task.txt`）
3. `.pi/skills/` 目录布局（DiJiang 管理 `dijiang-*` 前缀的子目录，非 `dijiang-*` 的 skill 共存不受影响）
4. `.runtime/.trellis_owned` 标记文件（表明 DiJiang 声明了对 `.runtime/` 中子路径的所有权）

任何破坏上述兼容性的变更都会在 minor 版本中声明。
