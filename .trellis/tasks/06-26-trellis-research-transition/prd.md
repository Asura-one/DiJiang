# Trellis 深度调研与 DiJiang 转型计划

## Goal

吸收 mindfold-ai/Trellis 的工程化设计，构建 **DiJiang 自有基础设施 `.dijiang/` + `dijiang` CLI**，达到与 Trellis 同等能力水平。`.dijiang/` 是 DiJiang 的主权目录，`.trellis/` 格式兼容供 Trellis CLI 读取，互不冲突。


## Context

DiJiang **拥有自身完整的 `.dijiang/` 基础设施**（项目级）和 `~/.dijiang/` 全局存储，
以及 `.pi/` 平台配置（skills/、prompt-templates/）。
`.trellis/` 不是 DiJiang 的依赖项——DiJiang 可以<u>读写兼容</u> `.trellis/tasks/` 的任务数据，
但自有格式在 `.dijiang/` 中（TOML vs JSON）。

三者关系：
- **`.dijiang/`** — DiJiang 主权目录（配置、workspace、hooks、spec）
- **`~/.dijiang/`** — 全局用户数据（mem 会话、全局配置、缓存）
- **`.trellis/`** — 兼容 Trellis 格式（task.json、spec），DiJiang 可读写但不依赖
- **`.pi/`** — Pi 平台运行时载体（skills、prompt-templates）

DiJiang **拥有自身完整的 `.trellis/` 基础设施**（scripts/、spec/、workspace/），以及 `.pi/` 平台配置（skills/、prompt-templates/）。`.trellis/` 和 `.pi/` 都是 DiJiang 自己的资产，不是 Trellis 框架的附属品。DiJiang 与 Trellis 是平行关系：Trellis 的设计模式可借鉴，DiJiang 独立实现。


DiJiang 的独特资产:
- 9 个 dj-* 自定义技能（dispatch, grill, tdd, hunt, check, implement, output, design, muse）
- Go 实现的全链路记忆系统 `dj-muse/`（单平台，待升级为多平台 adapter）
- S/M/L 三级任务分级 + 智能路由
- 深度对齐工作流（grill → output → implement/tdd → hunt ↔ check）

Trellis (mindfold-ai/Trellis) 是 AGPL-3.0 许可的开源工程框架，具有:
- TypeScript monorepo 架构（trellis-core + trellis CLI）
- 18+ AI 编码平台 configurator（Pi, Cursor, Claude, Codex, Gemini 等）
- Hooks 系统（class-1 push 自动注入 / class-2 pull 手动加载）
- Channel 模块（事件溯源 worker 管理）
- Mem 模块（跨会话记忆 + multi-platform adapter）
- Task 模块（24 字段结构化 task record + phase 投影）
- Template 系统（平台无关上下文注入模板）
- Skill/Command 系统（自动触发技能 + 用户可调用命令）

## Key Findings（调研结论）

### 1. Trellis 的核心架构优势

| 维度 | Trellis 实现 | DiJiang 现状 | 差距 |
|------|-------------|-------------|------|
| 平台支持 | 18+ 平台 configurator | 仅 Pi（但 dj-* 技能可跨平台复用） | dijiang CLI 需实现多平台 configurator |
| 注入机制 | Hooks auto-inject + pull-based prelude | 无 hooks，依赖手动上下文 | 需实现 hooks 系统（先 Pi class-1） |
| 任务模型 | 24 字段结构化 TrellisTaskRecord | ~23 字段（命名差异），task.py 已接近 | 字段名对齐（creator→source, parent→parentTask）+ 补缺字段 | P1 |
| Worker 管理 | Event sourcing channel 系统 | 无 | 可选，非优先 |
| 记忆系统 | adapter 模式，多平台对话/项目聚合 | Go 单平台 | **需要尽早实现多平台 adapter，优先级提升到 P0** |
| CLI 工具 | trellis init/start/check 等 | 仅有 task.py（Python）+ 无统一 CLI | 需构建 dijiang CLI（Rust cargo workspace） |
| 代码组织 | TS monorepo (pnpm workspace) | dj-* 技能为 .md 文件 + Go + Python | 需统一为 Rust cargo workspace |
| 规范注入 | 平台模板系统 | 基础上下文拼接 | 模板化（Trellis 框架层职责） |

### 2. DiJiang 的独特价值（不可丢失）

| 能力 | 说明 | 策略 |
|------|------|------|
| dj-* 技能体系 | dispatch/griell/tdd/hunt/check 等 9 个技能 | **保留并增强**，作为 DiJiang 差异化竞争力 |
| 任务分级系统 | S/M/L 三级自动判断 + 路由 | **保留**，比 Trellis 单一 phase 更精细 |
| 中文友好 | 中文输出、中文技能文档 | **保留**，本土化优势 |
| Go 全链路记忆 | dj-muse 的 session_summary 等 | **保留或迁移到 adapter 模式** |
| 深度对齐工作流 | grill → output → implement/tdd → hunt ↔ check | **保留**，Trellis 无等价物 |

### 3. 平台分类对比

Trellis 将 AI 平台分为两类:
- **Class-1 (hasHooks=true)**: Cursor, Claude, Gemini, Pi, Windsurf 等 — 可 push 注入
- **Class-2 (hasHooks=false)**: Codex, ZCode, Reasonix 等 — 需 pull 加载

DiJiang 可从中受益: 更多平台 = 更大用户群。

## Transition Strategy（转型策略）

### 阶段 0: `.trellis/` 基础设施完善（小修）

**实际状态：已基本就绪。** `task.py` 当前支持 ~23 字段，spec/ 已有 backend(6篇) / frontend(7篇) / guides(3篇) / meta(1篇)，workspace/ 已有 journal 格式。

剩余工作：
- 字段命名对齐（creator→developer/source, parent→parentTask）
- 补缺字段（`startedAt`, `archivedAt`, `acceptanceCriteria`, `keyDeliverables` 等）
- task.json 字段顺序对齐
- workspace journal 格式确认

验收：`trellis list` / `trellis status` 正确输出。（兼容，非依赖）
### 阶段 1: dijiang CLI — Rust 原生实现

**目标**: 使用 Rust 构建 `dijiang` CLI，单二进制分发，零运行时依赖。

同时完成 Mem 多平台 adapter。

⚠️ **技术栈统一决策：全量 Rust。**
Python `.trellis/scripts/`（7.6k 行）和 Go `dj-muse/`（7.2k 行）全部迁移到 Rust。
TypeScript 原型 `packages/` 废弃。

具体:
- Rust cargo workspace 结构（`crates/core`, `crates/cli`, `crates/task`, `crates/mem`, `crates/configurator`）
- **Mem adapter 多平台实现（优先级最高）**
  - Rust trait 定义 MemAdapter
  - Pi MemAdapter（文件系统扫描 `.pi/sessions/`）
  - Claude/Codex/Cursor MemAdapter
  - `dijiang mem list` 跨平台聚合
- Configurator 体系（先 Pi，其他后续扩展）
- Hooks 系统（Pi 平台 class-1 auto-inject）

分发:
- `cargo install dijiang`
- GitHub Releases 预编译二进制 (Linux/macOS/Windows)
- 无 Node.js / Python 运行时依赖

⚠️ **Mem 多平台不再是阶段 3 的任务——它从阶段 1 起就必须具备。**

具体:
- Monorepo 结构（dijiang-core + dijiang CLI）
- **Mem adapter 多平台实现（优先级最高）**
  - 定义 MemAdapter 接口
  - Pi MemAdapter（复用 dj-muse Go 逻辑，通过 adapter 包装）
  - Claude/Codex/Cursor MemAdapter（读取各平台 session 文件）
  - `dijiang mem list` 跨平台聚合项目统计
- Configurator 体系（先支持 Pi，其他平台后续扩展）
- Hooks 系统（Pi 平台 class-1 auto-inject）

### 阶段 2: 多平台 Configurator 扩展

**目标**: 从 Pi 扩展到 Cursor + Codex + Claude，使 dj-* 技能在更多平台可用。

具体:
- 实现 Cursor/Claude/Codex configurator
- 实现对应 hooks 适配
- 测试跨平台一致性
- Mem adapter 已在阶段 1 就绪，此阶段直接使用

### 阶段 3: dj-* 技能增强与生态

**目标**: 强化 DiJiang 差异化竞争力，新增技能、优化现有流程。

**已完成（超前交付）**:
- ✅ dj-dispatch: 自动激活（session:start）+ phase 映射
- ✅ dj-grill: Phase 标记 + 自动恢复
- ✅ dj-output: TemplateContext 模型 + Spec 更新合约（7 章节）
- ✅ dj-hunt: spec 晋升合约模板（7 章节）
- ✅ dj-muse: 多平台 Session 聚合（vNext 文档）

**后续**:
- 新增 dj-audit（全仓审计）、dj-pattern（模式识别）、dj-review（代码评审）
- 增强 dj-grill 的提问策略
- 增强 dj-hunt 的代码定位能力
- 文档和社区建设

## Acceptance Criteria

- [x] 完成 Trellis 源码架构深度分析文档（design.md）
- [x] 完成 DiJiang vs Trellis 详细对比矩阵
- [x] 完成转型路线图（含阶段、里程碑、风险）
- [x] 5 个 dj-* SKILL.md 优化（dd-dispatch/grill/output/hunt/muse）
- [ ] 完成优先级重评估（本节已完成）
- [ ] PRD 获得评审通过

## Notes

- Trellis 为 AGPL-3.0，DiJiang 转型涉及协议兼容性需确认
- DiJiang 的 dj-* 技能是核心差异化资产，转型过程中不可丢失
- Go 全链路记忆系统（dj-muse）需迁移到 Rust
- Python 脚本（task.py 等）需迁移到 Rust
- For complex tasks, add `design.md` for technical design and `implement.md` for execution planning before `task.py start`.

## 技术栈统一决策（2026-06-28）

**核心变更: Python + Go + TypeScript → 全量 Rust**

| 序号 | 迁移源 | 行数 | 目标 Rust crate | 说明 |
|------|--------|------|----------------|------|
| 1 | `.trellis/scripts/` (Python) | 7,622 | `crates/task` | task CRUD、jsonl 上下文、git context |
| 2 | `dj-muse/` (Go) | 7,208 | `crates/mem` | 会话管理、多平台 adapter |
| 3 | `packages/` (TypeScript) | 554 | `crates/{cli,core}` | CLI 入口、核心类型（已在 TS 原型中定义好） |

迁移策略: 按 crate 分步进行，先迁移 task（复用现有 task.json schema），再迁移 mem（优先 adapter 接口），最后 CLI 整合。

## 优先级重评估（2026-06-28）

基于实际状态重新排序：

```
P0 ── `dijiang` CLI 构建（dijiang-core + dijiang-cli monorepo）
       `dijiang init / start / status / mem list`
P1 ── task.py schema 对齐（字段命名 + 补缺，小修）
       Mem 多平台 adapter（Claude/Codex/Cursor）
P2 ── 多平台 Configurator（Cursor/Codex/Claude）
P3 ── 后续技能增强（dj-review, dj-audit 等）
```

关键转变：
- `.trellis/` 不再是被动对齐的框架层，而是 DiJiang 自有的基础设施
- `dijiang` CLI 是 DiJiang 的身份标识，`trellis` CLI 兼容是附加价值
- Phase 0 已基本完成（80%+），不再是阻塞前置
