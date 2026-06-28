# Journal - tiezhu (Part 1)

> AI development session journal
> Started: 2026-06-22

---


## 2026-06-28 — Phase 1 P0: `dijiang` CLI 构建完成

### 交付物
- **pnpm monorepo** — root workspace + `packages/core/` + `packages/cli/`
- **`@dijiang/core`** — 类型定义 (TaskRecord, SessionRecord, DiJiangPlatformMeta, MemAdapter 接口) + ConfiguratorRegistry
- **`@dijiang/cli`** — 4 个命令全部正常工作:
  - `dijiang init` — 生成 `.trellis/` + `.pi/` 目录
  - `dijiang start` — 查看/启动任务
  - `dijiang status` — 项目状态概览
  - `dijiang mem list` — Pi 平台 session 扫描

### 验证
- ✅ `node packages/cli/bin/dijiang.js --help` 输出正确
- ✅ `dijiang init` 在空目录生成完整结构
- ✅ `dijiang status` 显示活跃任务和平台配置
- ✅ `dijiang start <task>` 切换当前任务
- ✅ `dijiang mem list` 正常扫描 Pi session

### 下一步
- 实现 Pi Configurator (Pi 平台适配器)
- 实现 Pi MemAdapter (包装现有 session 发现逻辑)
- task.py schema 对齐 (24 字段)

## 2026-06-28 — 技术栈统一决策：全量 Rust

### 决策
Python + Go + TypeScript → **全量 Rust**（cargo workspace）

### 迁移范围
| 来源 | 行数 | 目标 crate |
|------|------|-----------|
| `.trellis/scripts/` (Python) | 7,622 | `crates/task` |
| `dj-muse/` (Go) | 7,208 | `crates/mem` |
| `packages/` (TypeScript) | 554 | 废弃（原型） |

### 已更新文档
- PRD: Phase 1 重写为 Rust，新增技术栈统一决策章节
- design.md: ADR-1 重写为 Rust，风险表更新
- implement.md: 总体策略重写为 Rust cargo workspace

### 下一步
- 搭建 Rust cargo workspace（Cargo.toml + crates/ 目录）
### 2026-06-28: 多平台 Memory Adapter（Hermes + OpenCode）

**HermesAdapter** — 实现 `~/.hermes/sessions/` 三源扫描：
- `sessions.json` 索引（快速路径）
- `session_YYYMMDD_HHMMSS_<id>.json` 元数据文件（含 messages 数组 → 提取 first user message 为 task）
- `YYYMMDD_HHMMSS_<id>.jsonl` JSONL 事件日志（fallback）
- 去重逻辑：三源交叉使用 `session_id` 匹配
- 项目分组：`hermes/{platform}`（cli / telegram / curator）
- 实机数据验证：`hermes/cli` 267 sessions，`hermes/telegram` 5，`hermes/curator` 2

**OpenCodeAdapter** — 空桩（SQLite 后端依赖推迟），占位注册

**修复的 bug**：
- UTC-8 截断：`&text[..117]` → `.chars().take(117).collect()`
- 扫描守卫：`if sessions.is_empty()` 导致 index 存在时忽略 300+ session 文件
- 重复注册：CLI 中 PiMemAdapter 注册了两遍

**总测试**：16/16（mem 16），全仓 19/19
**总测试**：16/16（mem 16），全仓 25/25
**总 session 数**：1074 条（5 providers: pi + claude + codex + hermes + opencode）

### 2026-06-28: `dijiang start` 命令

`crates/cli/src/main.rs` — 新增 `Start` 命令：

```
$ dijiang start fix-login-bug "修复登录 bug"
  ✓ Task 'fix-login-bug' created
    Title: 修复登录 bug
    Status: planning → in_progress
  ✓ Session started

  Project: DiJiang
  Active:  .trellis/tasks/fix-login-bug

  Task summary:
    Title:  修复登录 bug
    State:  in_progress
    Phase:  implement
```

**行为**：
- `dijiang start <name> [title]` — 创建或激活任务
- 已存在的任务：状态更新为 in_progress，保留已有 `startedAt`
- 新任务：创建完整 `task.json`，写入 `startedAt`
- 写入 `active_task.txt` 设为活跃任务

**修复的结构问题**：enum Commands 分支不完整、match block 缺少 Mem/TaskCommands 分支、fn main 缺少闭合 `}`、冗余 `Status` 变体

**总测试**：25/25（configurator 4 + mem 16 + task 5）

### 2026-06-28: Phase 0 字段对齐 — TaskRecord 匹配 Trellis `TrellisTaskRecord`

**关键发现**：design.md 写的 24 字段方案和 Trellis 实际代码不一致。
实际 Trellis `TrellisTaskRecord` 是 Python `task.py` 的格式（`creator` 不是 `developer/source`，`parent` 不是 `parentTask`，无 `startedAt`/`acceptanceCriteria` 等字段）。

**改动**：
- 重写 `crates/task/src/types.rs` 的 `TaskRecord`，精确对齐 Trellis 实际 24 字段 + 字段顺序
- Trellis 标准 Optional 字段始终序列化（即 null 也写出）
- DiJiang 扩展字段（`startedAt`, `archivedAt`, `acceptanceCriteria` 等）用 `#[serde(skip_serializing_if = "Option::is_none")]` 避免干扰标准格式
- 删除已无用的 `as_string` 自定义反序列化器（`subtasks` 改为 `Vec<String>`）
- 更新 `store.rs` 的 `create_task()` 使用新字段名

**验证**：
- `dijiang start` 创建的 task.json 输出 24 Trellis 字段 + 1 DiJiang 扩展
- 字段顺序和 `TASK_RECORD_FIELD_ORDER` 一致
- 现有 Python 格式的 task.json 可正常读取（`dijiang status`、`dijiang task list`）
- 25/25 测试全绿

### 2026-06-28: Phase 2 — 多平台 Configurator（Cursor/Claude/Codex）

新增 3 个 Configurator：

**CursorConfigurator** (`crates/configurator/src/cursor.rs`)
- `.cursor/rules/dijiang.mdc` — Cursor 规则文件，描述 DiJiang 项目结构和工作流
- `.cursor/hooks.json` — session:start 钩子配置
- class-1 (hasHooks=true)

**ClaudeConfigurator** (`crates/configurator/src/claude.rs`)
- `CLAUDE.md` — 项目概述，含任务工作流说明
- `.claude/settings.json` — 注册 3 个 slash command（status, task-list, start）
- class-1 (hasHooks=true)

**CodexConfigurator** (`crates/configurator/src/codex.rs`)
- `.codex/agents/trellis-implement.toml` — 实现子代理定义
- `.codex/agents/trellis-check.toml` — 审查子代理定义
- `.codex/hooks/inject-workflow-state.py` — 钩子脚本
- `.codex/hooks.json` — UserPromptSubmit 钩子配置
- `.codex/config.toml` — 启用 hooks
- class-2 (hasHooks=false)

**`init_project` 整合**：`init.rs` 自动运行全部 6 个 configurator（Pi + Cursor + Claude + Codex + OpenCode + Hermes）

**新增 OpenCode & Hermes Configurator** (`crates/configurator/src/{opencode,hermes}.rs`)

**OpenCodeConfigurator** (class-2)
- `.opencode/agents/trellis-{implement,check}.md` — 子代理定义
- `.opencode/plugins/session-start.js` — 会话插件（chat.message 拦截）
- `.opencode/lib/trellis-context.js` + `session-utils.js` — 工具库
- `.opencode/package.json` — `@opencode-ai/plugin` 依赖

**HermesConfigurator** (class-1)
- `.hermes/agents/trellis-{implement,check}.md` — 子代理定义
- `.hermes/hooks.json` — session:start 钩子

**验证**：`dijiang init` 在空目录生成 24 个文件，覆盖 6 个平台
- Pi (4), Cursor (2), Claude (2), Codex (4), OpenCode (6), Hermes (3) + .dijiang/ + AGENTS.md
**测试**：30/30 全绿（configurator 5 → +1 test）

### 2026-06-28: 遗留代码清理（P0）

**删除的遗留代码**：
- `.trellis/scripts/` — 47 个 Python 文件（~7,600 行，已由 `crates/task` 覆盖）
- `dj-muse/` — 54 个 Go 文件（~7,200 行，已由 `crates/mem` 覆盖）
- `.dijiang/scripts/` — 4 个 Python 残留文件
- `.codex/hooks/inject-workflow-state.py` — 旧 Python 钩子（已替换为 .sh）
- `crates/core/` — 空存根 crate（已从 workspace 移除）

**更新的引用文件**（Python→`dijiang` CLI）：
- `crates/configurator/src/pi.rs` — extension 模板、prompt 模板、agents_md
- `crates/configurator/src/codex.rs` — 内联钩子脚本（Python→shell）、hooks.json
- `.pi/extensions/trellis/index.ts` — session:start 钩子使用 `dijiang task current`
- `.cursor/hooks.json` — 使用 `dijiang status`
- `.codex/hooks.json` — 使用 `.codex/hooks/inject-workflow-state.sh`
- `.pi/agents/trellis-{implement,check,research}.md` — 指令中使用 `dijiang task current`
- `AGENTS.md` — 移除 Python 回退描述


---

## 2026-06-28 — Phase 3: dj-* 技能增强 + 遗留清理完成

### 交付物

- **`dj-pattern`（新增，141 行）** — 重复模式检测 + git 历史修复分析 + YAGNI 验证的抽象建议 + 一致性检查
- **`dj-review`（新增，148 行）** — A-E 五维审查（正确性/安全/可维护/一致性/可评审性）+ 严重度分级 + 判定结果
- **`dj-grill` 增强（213 行，+94%）** — 自适应深度控制 + 推荐答案构造法 + 代码库感知建议 + 追问生成模式 + Gap 检测
- **`dj-hunt` 增强（413 行，+52%）** — 7 种代码定位策略：错误驱动 → 语义搜索 → 调用链 → Git 历史 → 分层交叉 → 特征搜索 → 失败回退

### 遗留清理

- 已删除 8 个孤立 `trellis-*` skills（引用已删的 Python 脚本）
- 已删除 3 个 `trellis-*` agents（引用已删的 Python 脚本）
- 保留 `trellis-meta`（参考文档库，非执行技能）
- 确认 `dijiang-*` 生成文件中零 `python3` 引用

### 验证

- `cargo test` 27/27 全绿
- `.pi/` 结构：3 dijiang-* skills + 3 agents + 1 extension + 2 prompts
- `dj-*` 用户级 skills：19 个（含新增 dj-pattern、dj-review）

#### 未完成（需后续任务）
1. 跨平台验证（Phase 2.4）— 需要在 Claude/Cursor/Codex 上实测
2. CI/CD + Release pipeline（Phase 3.3）— GitHub Actions + cargo publish/cargo-dist
3. PRD 评审确认
