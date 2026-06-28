# 实施计划

## 总体策略

四阶段交付，每阶段独立可验证。

**技术栈决策: Rust cargo workspace**。全部代码（Python 7.6k + Go 7.2k + TS 0.5k）迁移到 Rust。

优先级：
```
P0 ── `dijiang` CLI（Rust cargo workspace）
       crates: core, cli, task, mem, configurator
       `dijiang init / start / status / mem list`
P1 ── task 模块迁移（Python task.py 重写为 crates/task）
       mem 模块迁移（Go dj-muse 重写为 crates/mem）
       task.json schema 对齐
P2 ── 多平台 Configurator（Pi → Cursor → Claude → Codex）
P3 ── dj-* 技能增强与生态
```

关键转变：
- `.trellis/` 是 DiJiang 自有的基础设施，`dijiang` CLI 是主工具
- Phase 0 已基本完成（80%+），不再是阻塞前置
- 5 个 SKILL.md 优化已提前完成（Phase 3 超前交付）
```

关键转变：
- `.trellis/` 是 DiJiang 自有的基础设施，`dijiang` CLI 是主工具
- Phase 0 已基本完成（80%+），不再是阻塞前置
- 5 个 SKILL.md 优化已提前完成（Phase 3 超前交付）
每阶段按 **grill → output → implement/tdd → hunt ↔ check** 流程推进。

---

## Phase 0: `.trellis/` 基础设施完善（小修）

**实际状态：已基本就绪。** `task.py` 当前支持 ~23 字段，
spec/ 已有 backend(6篇) / frontend(7篇) / guides(3篇) / meta(1篇)，
workspace/ 已有 journal 格式。

**剩余任务**:
### 0.1 字段命名对齐
- creator→developer/source
- parent→parentTask
- 补缺字段: startedAt, archivedAt, acceptanceCriteria, keyDeliverables, sessionId, estimatedEffort, actualEffort, reviewStatus, reviewComments, tags

### 0.2 字段顺序对齐
- 对齐 TASK_RECORD_FIELD_ORDER

### 0.3 workspace journal 格式确认
- 确保格式与 Trellis CLI 兼容

### 0.4 验证
- `trellis list` / `trellis status` 正确输出
---

## Phase 1: `dijiang` CLI — Rust 原生（P0 — DiJiang 身份标识）

**目标**: 使用 Rust 构建 `dijiang` CLI，单二进制分发，零运行时依赖。

⚠️ `dijiang` CLI 是 DiJiang 在终端上的身份标识。同时完成 Mem 多平台 adapter。

**子任务**:
### 1.1 工作空间初始化
- 建立 Rust cargo workspace
- `crates/core/` — 核心类型 + trait 定义
- `crates/cli/` — CLI 入口（clap）
- `crates/task/` — task CRUD（迁移 Python task.py）
- `crates/mem/` — 记忆系统（迁移 Go dj-muse）
- `crates/configurator/` — 平台适配器
- `Cargo.toml` workspace 配置

### 1.2 Mem 多平台 Adapter（最高优先级）
- 定义 `MemAdapter` 接口
- **Pi MemAdapter**: 扫描 `~/.dijiang/mem/` 读取 session 数据
- **Claude MemAdapter**: 读取 Claude session 文件
- **Codex MemAdapter**: 读取 `.codex/sessions/`
- **Cursor MemAdapter**: 读取 Cursor session 数据
- `dijiang mem list` — 跨平台聚合项目统计
- 验证: `dijiang mem list` 显示 >=2 个平台的 session

### 1.3 Configurator 体系（先 Pi）
- 定义 DiJiangPlatformMeta（扩展 hasDJSkills）
- ConfiguratorRegistry
- Pi Configurator: 完整实现（生成 `.pi/skills/dj-*/`）

### 1.4 CLI 命令
- `dijiang init` — 初始化项目（创建 `.dijiang/`，`--trellis` 额外创建 `.trellis/`）
- `dijiang start` — 启动会话
- `dijiang status` — 项目状态
- `dijiang mem list` — 跨平台记忆列表

### 1.5 验证
- `dijiang init` 在空目录生成完整 `.dijiang/` + `.pi/`
- `dijiang start` 正确激活任务
- `dijiang mem list` 聚合多种平台 session 数据

---

## Phase 2: 多平台 Configurator 扩展

**目标**: Configurator 从 Pi 扩展到 Cursor + Codex + Claude，使 dj-* 技能在更多平台可用。

注意：Mem adapter 已在阶段 1 就绪，此阶段直接使用。

**子任务**:

### 2.1 Cursor Configurator
- `.cursor/rules/` 写入
- hasHooks=true → 自动注入
- 注入 trellis-start + dj-* 技能列表

### 2.2 Codex Configurator
- `.codex/` 目录写入
- hasHooks=false → pull 模式
- 注册 `/trellis:start` 命令手动加载

### 2.3 Claude Configurator
- `CLAUDE.md` + `.claude/` 写入
- hasHooks=true → 自动注入

### 2.4 跨平台验证
- Pi: 全功能 (dj-* + trellis-*)
- Cursor: trellis-* 基础功能 + dj-* 技能列表
- Codex: trellis-* 基础功能
- Claude: trellis-* 基础功能

---

## Phase 3: dj-* 技能增强与生态

**目标**: 强化 DiJiang 差异化竞争力。

**子任务**:

### 3.1 新增技能
- dj-audit: 全仓代码审计
- dj-pattern: 模式识别（从历史修复中学习）
- dj-review: 代码评审辅助

### 3.2 现有技能增强
- dj-grill: 增强多轮追问策略
- dj-hunt: 增强代码定位（利用 mem 跨平台数据）

### 3.3 工程化与社区
- CI/CD (GitHub Actions)
- 单元测试 + 集成测试
- npm 发布流程
- 文档站点

---

## 已完成的 Phase 3 工作（超前交付）

以下 SKILL.md 优化已在 Phase 1 之前完成：

| 技能 | 优化内容 | 行数 |
|------|---------|------|
| `dj-dispatch` | 自动激活（session:start）+ phase 映射 | ~40 行 |
| `dj-grill` | Phase 标记 + 自动恢复 | ~40 行 |
| `dj-output` | TemplateContext 模型 + Spec 更新合约（7 章节） | ~100 行 |
| `dj-hunt` | spec 晋升合约模板（7 章节） | ~60 行 |
| `dj-muse` | 多平台 Session 聚合（vNext 文档） | ~45 行 |
---

## Phase 2: 技能系统统一

**目标**: dj-* 技能集成到统一系统，可通过 `/dj-dispatch` 在 Pi 中触发。

**子任务**:

### 2.1 Skill Spec 设计
- 扩展 Trellis skill frontmatter
- 新增 `dj`, `tier`, `category` 字段
- 定义自动触发 vs 用户调用机制

### 2.2 Skill 迁移
- 9 个 trellis-* 技能对齐 Trellis 标准
- 9 个 dj-* 技能迁移到新格式

### 2.3 验证
- `/dj-dispatch` 在 Pi 中正确路由
- `/dj-grill` 正确触发提问
- 所有 dj-* 技能均可用

---

## Phase 3: 多平台扩展

**目标**: 同一 DiJiang 项目在 Pi/Cursor/Codex 中可用。

**子任务**:

### 3.1 Cursor Configurator
- `.cursor/rules/` 写入
- Hooks 集成

### 3.2 Codex Configurator
- `.codex/` 目录写入
- Pull-based 命令注册

### 3.3 跨平台验证
- Pi: 全功能 (dj-* + trellis-*)
- Cursor: trellis-* 基础功能
- Codex: trellis-* 基础功能

---

```
Phase 0（小修，非阻塞）

  └→ Phase 1（CLI — Rust P0）

        └→ Phase 2（Configurator — P2）

              └→ Phase 3（技能增强 — P3）
```

Phase 0 已基本完成（80%+），不再阻塞后续阶段。
Phase 1 是 DiJiang 的身份标识和技术栈统一的起点。
迁移顺序：task（Python）→ mem（Go）→ configurator（新写）。

Phase 0 已基本完成（80%+），不再阻塞后续阶段。
Phase 1 是所有工作的载体和 DiJiang 的身份标识。

---

## 验证标准

每个子任务完成后:
1. 运行对应的验证命令
2. 检查输出是否符合预期
3. 记录到 check.jsonl

## 回滚策略

- 每 Phase 完成前创建 git tag
- 如发现 Protocol 不兼容，回退到 Phase 0 重新对齐
