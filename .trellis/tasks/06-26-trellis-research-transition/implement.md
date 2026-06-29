# DiJiang 工程化转型 — 实施路线图

> **状态**: 执行中  
> **最新更新**: 2026-06-29  
> **核心决策**: Rust 实现 · 混合模板（嵌入 + 远程）· dj-* 技能生态复用

---

## Phase 0：基础设施完善 ✅ 已完成

交付于 2026-06-29。

### 0.1 模板系统

| 组件 | 文件 | 说明 |
|------|------|------|
| 模板引擎 | `crates/configurator/src/templates.rs` | `rust-embed` 编译时嵌入 + `{{key}}` 变量替换 |
| 项目 skill | `templates/skills/dijiang-*` | 3 个薄层 skill，加载上下文后委托给 dj-* |
| 子代理 | `templates/agents/dijiang-*` | implement/check/research 定义 |
| Pi 扩展 | `templates/extensions/dijiang/index.ts` | session_start 注入 active task + spec |
| 工作流 | `templates/config/workflow.md` | 完整工作流指南 |
| AGENTS.md | `templates/config/agents.md` | 兼容 DIJIANG block 替换 |

### 0.2 交互式 Init

- `dialoguer` 三步骤：项目名 → 开发者名（git config 自动检测）→ 平台多选
- 静默模式：`--yes`, `--platforms pi,cursor`, `--developer tiezhu`
- 平台：Pi / Cursor / Claude Code / Codex CLI / OpenCode / Hermes

### 0.3 Crate 框架

- `PlatformKind` 枚举 + `InitConfig` 结构体（`types.rs`）
- `init_project_with_platforms(cwd, name, dev, &[PlatformKind])`
- 32 个单元测试全部通过

### 验证标准

- [x] `cargo build --release` 通过
- [x] `cargo test` 全部通过（32/32）
- [x] `dijiang init` 交互式模式工作
- [x] `dijiang init --yes` 非交互模式工作
- [x] 所有模板从嵌入文件加载，无 inline 字符串

---

## Phase 1：Rust CLI 核心深化

### 1.1 子命令补全

当前 CLI 子命令：

```
dijiang status        ✓ 工作
dijiang start         ✓ 工作
dijiang task list     ✓ 工作
dijiang task current  ✓ 工作
dijiang task status   ✓ 工作
dijiang init          ✓ 增强（交互式 + 平台选择）
dijiang mem list      ✓ 工作
dijiang mem sync      ✗ 待实现
```

**待实现**:
- `dijiang task archive` — 归档已完成任务
- `dijiang task prune` — 清理旧会话数据
- `dijiang mem sync` — 跨平台 session 聚合

### 1.2 错误处理

- 统一的 `DijiangError` 类型（取代散落的 `anyhow::Result`）
- 用户友好的错误消息（中文/英文，含建议操作）
- 退出码标准化

### 1.3 Init 增强

- `--template <url>` 远程模板注册表支持
- 预置模板列表（`dijiang init --list-templates`）
- 初始化后 `git init` + 初始 commit 建议

### 验证标准

- [ ] 所有子命令有 `--help`
- [ ] 统一的错误类型和退出码
- [ ] 远程模板可下载和引用

---

## Phase 2：多平台 Configurator 深化

### 2.1 平台适配器

当前平台 Configurator 状态：

| 平台 | 状态 | 深度 |
|------|------|------|
| Pi | ✓ 完整 | 写入 skills/agents/extensions/prompts/settings |
| Cursor | ✓ 基础 | 写入 rules/hooks.json |
| Claude | ✓ 基础 | 写入 CLAUDE.md/agent |
| Codex | ✓ 基础 | 写入 agents/hooks |
| OpenCode | ✓ 基础 | 写入 opencode 配置 |
| Hermes | ✓ 基础 | 写入 agents/hooks.json |

**深化方向**:
- 所有平台支持 hooks 注入（推/拉模式自动注入上下文）
- 平台特定的 settings 模板从嵌入文件加载
- 平台检测（`--auto-detect` 检测用户安装了哪些平台）

### 2.2 Configurator 注册表

```rust
trait PlatformConfigurator {
    fn platform(&self) -> PlatformKind;
    fn configure(&self, cwd: &Path) -> Result<()>;
    fn hooks(&self) -> Vec<Hook>;  // 推/拉 hooks
    fn priority(&self) -> u8;      // 优先级排序
}
```

- 注册表模式：`Registry::register(Box<dyn PlatformConfigurator>)`
- 支持外部 configurator（通过 trait 对象）

### 2.3 Configurator API 拓展

Configurator trait 新增能力：
- 模板变量注入：每个 configurator 可提供平台特定的变量
- 文件冲突处理：备份/覆盖/跳过策略
- 差异化生成：允许按平台生成不同文件集

### 验证标准

- [ ] 所有 6 个平台 Configurator 深度一致
- [ ] hooks 工作（auto-injection 测试）
- [ ] 平台检测功能可用
- [ ] Configurator 注册表可扩展

---

## Phase 3：Template System 完善

### 3.1 远程模板注册表

```bash
dijiang init --registry gh:owner/repo  # 从 GitHub 加载模板
dijiang template list                   # 列出可用模板
dijiang template pull <name>            # 拉取最新模板
```

- 模板格式：TOML/YAML 清单 + 目录树
- 变量替换增强：支持条件判断（`{{#if platform == "pi"}}`）
- 模板版本管理：语义化版本，向上兼容

### 3.2 模板 CLI 子命令

```bash
dijiang template list       # 列出本地 + 远程模板
dijiang template pull       # 从注册表拉取
dijiang template validate   # 校验模板格式
```

### 3.3 模板内容扩展

- 支持 spec 模板（code guidelines 骨架）
- 支持 task 模板（task.json + 空 prd/design/implement.md）
- 支持 CI/CD 模板（GitHub Actions / GitLab CI）

### 验证标准

- [ ] 远程模板可拉取和初始化
- [ ] 模板清单格式稳定
- [ ] 模板 CLI 子命令可用

---

## Phase 4：dj-* 技能与 Workflow 对齐

### 4.1 技能引用审计

确保所有生成的 skill 和 agent 文件正确引用现有的 dj-* 技能：

| 生成文件 | 引用的 dj-* 技能 | 状态 |
|----------|-------------------|------|
| `dijiang-start/SKILL.md` | dj-dispatch, dj-implement, dj-hunt, dj-check 等 | ✓ |
| `dijiang-continue/SKILL.md` | dj-grill, dj-implement, dj-hunt, dj-check, dj-output, dj-tdd | ✓ |
| `dijiang-finish-work/SKILL.md` | dj-check | ✓ |
| `dijiang-implement.md` | dj-implement, dj-tdd, dj-prototype, dj-ponytail, dj-script, dj-karpathy | ✓ |
| `dijiang-check.md` | dj-check, dj-audit, dj-debt, dj-health | ✓ |
| `dijiang-research.md` | dj-hunt, dj-dispatch | ✓ |

### 4.2 Workflow 文档对齐

- workflow.md 列出所有 dj-* 命令 + 使用场景
- AGENTS.md 的 DIJIANG 块列出常用命令
- 技能路由表保持最新

### 4.3 响应式技能

- 可选：当检测到某些 dj-* 技能缺失时给出建议
- 可选：`dijiang skill doctor` 检查技能完整性

### 验证标准

- [ ] 所有引用一致，指向正确的 dj-* 技能名
- [ ] workflow.md 路由表完整
- [ ] AGENTS.md 常用命令列表完整

---

## 架构总览

```
dijiang CLI (crates/cli)
├── init       → 交互式/非交互初始化
├── start      → 创建并激活任务
├── status     → 项目状态概览
├── task       → 任务 CRUD + 查询
├── mem        → 内存会话管理
├── template   → (Phase 3) 模板管理

dijiang-configurator (crates/configurator)
├── templates/     → 嵌入模板文件 (rust-embed)
├── templates.rs   → 模板引擎 (render + substitute)
├── types.rs       → PlatformKind, InitConfig, DijiangConfig
├── init.rs        → 初始化逻辑 + 平台分发
├── pi.rs, cursor.rs, claude.rs, ... → 平台适配器

dijiang-task (crates/task)
├── types.rs   → TaskRecord, TaskStatus
├── store.rs   → 文件存储 + CRUD

dijiang-mem (crates/mem)
├── adapters/  → Pi, Claude, Codex, Hermes, OpenCode
├── registry   → 跨平台聚合
```

## 依赖关系

```
Phase 0 ─────────────────────────────▶ ✅ 完成
   |
   ├──▶ Phase 1 (CLI 核心) ──▶ Phase 3 (Template 远程)
   │
   └──▶ Phase 2 (多平台 Configurator) ──▶ Phase 4 (技能对齐)
```

- Phase 1 和 Phase 2 可以并行
- Phase 3 依赖 Phase 1（需要 template 子命令框架）
- Phase 4 依赖 Phase 2（需要多平台 Configurator 稳定）
