# 变更日志

## [0.13.6] — 2026-07-23

### 新增

- `finish-work` 硬门禁：`version-impact ≠ none` 时强制根 `CHANGELOG.md` 含目标版本 Keep a Changelog 条目（中英 section）
- 版本权威读取顺序：Cargo workspace → package.json → VERSION；`none` 时禁止相对 HEAD 漂移
- 模板：`check-version.sh` 对齐 workspace/VERSION；version-management / finish-work skill / meta/changelog 约定

### 变更

- Cargo workspace bump 后自动同步根 `VERSION`（若存在）
- e2e 剥离宿主 `DIJIANG_CONTEXT_ID`，避免 session 路径非 hermetic

## [0.13.5] — 2026-07-23

### 变更

- 统一 crate 版本到 Cargo workspace `0.13.5`（ADR 004：workspace 为版本权威面）
- 各 crate 使用 `version.workspace = true`；根 `VERSION` 与 workspace 同号

### 新增

- （回填汇总）0.10.0 之后至 0.13.5：finish-work 路由/门禁、task worktree、dispatch/route gate、mem/session 闭环、configurator update 与模板投影等演进

## [0.13.0] — 2026-07-20

### 变更

- （粗粒度回填）CLI/task/workflow 平台能力持续迭代至 0.13 线；细节见对应任务归档与 git 历史

## [0.12.0] — 2026-07-16

### 变更

- （粗粒度回填）中间次要版本区间汇总；完整逐 commit 说明不在此展开

## [0.11.0] — 2026-07-13

### 变更

- （粗粒度回填）0.10.0 之后功能与修复区间汇总

## [0.10.0] — 2026-07-11

### 新增

- P0: Python 脚本辅助层（.dijiang/scripts/ — task.py, get_context.py, add_session.py, common/{paths,types,io,config,tasks}）
- P0: workflow-state 面包屑（6 个 [workflow-state:...] 标签嵌入 workflow.md）
- P1: JSONL 上下文清单（common/context.py — seed/add/list/validate-context）
- P1: 任务层级（add-subtask/tree — parent/children/subtasks 关系）
- P1: 会话身份识别（common/session.py — 检测 6 平台: pi/claude/cursor/codex/hermes/terminal）
- P2: 可互换 workflow 模板（standard + tdd 模板，common/workflow_templates.py）
- P2: Agent 定义文件（agents/implement.md + check.md — YAML frontmatter, 上下文加载顺序）
- P2: Spec 注册表同步（common/spec_sync.py — hash 漂移检测, registry.jsonl）
- P3: 跨会话记忆（common/memory.py — 按 keyword/task/date/platform/event 检索 session 日志）
- P3: 安全提交模式（safe_commit.py — 分阶段 add/commit, auto-commit 配置控制）

### 变更

- Rust CLI main.rs 拆分为 `commands/` 模块（4402→800 行，15 个命令模块）
- 代码规范：从 Trellis 吸收 10 个关键亮点，落地 P0-P3

## [0.9.0] — 2026-07-08

### 变更

- Python 脚本导入路径（运行从 .dijiang/scripts/）
- Spec 注册表初始结构和 scansory 流程

### 新增

- .dijiang/scripts/ 模块初始化
- 版本号从 0.6.2 → 0.9.0

## [0.6.2] — 2026-07-03

### 新增

- CLI: `dijiang finish-work` 自动化 — 验证、文档同步、版本影响声明
- CLI: `dijiang doc-sync check` 子命令，从 git diff 检测哪些长期文档需要更新
- CLI: `dijiang spec-sync check/record`，追踪 spec 文件 checksum 变更
- CLI: Route Gate Phase 1 — 对 active task 的 dispatch 施加工况阶段约束
- CLI: Git Gate Phase 2 — worktree 就绪/供应/阻塞检测
- CLI: finish-work Phase 4 的 cleanup gate（integrate, push, worktree 清理）
- CLI: `dijiang update` 项目刷新机制（hash 比较 + GitHub 下载）
- Mem: `dijiang mem findings/learn/archive` 子命令
- Mem: 五层记忆架构及双速演化
- Channel: `dijiang channel spawn/list/send/execute` 子命令
- Channel: 带超时的并行执行
- Task: `dijiang task start/status/archive/prune` 子命令
- Skills: 所有 `dj-*` skill 文件归置于 `.dijiang/`
- Templates: session 包装器与规范路径对齐
- CI: 所有 CLI 输出本地化为中文
- Spec: 架构决策记录模板（ADR）
- Spec: backend/frontend/meta 编码规范

### 变更

- Context: 规范工作流投影同步到 workflow.md 和 AGENTS.md
- Dispatch: 模糊请求通过 `dj-grill` 路由，不再直接实现
- Finish-work: 支持 `--no-active-task` 模式
- Task: 路径从遗留 `.trellis/` 迁移到 `.dijiang/`
- Skills: 边界明确；应用 Darwin 优化

### 修复

- CLI: phase jump bug — DispatchDecision `task_status` 硬化为 Planning/Align
- CLI: 从主仓库自动清理任务 worktree
- CLI: finish-work 支持中文 commit message
- Dispatch: 缩小启动和异常路由范围
- Finish-work: 从外部 worktree 发现项目根目录
- Finish-work: 显示 worktree 残留决策门
- Workflow: 过期 active task 检测和 TDD 防漂移
- Workflow: 代码 dispatch 路由时创建 worktree
- Skills: 重复 skill 检测和清理
- Skills: `dj-output` 路径从 `.trellis/` 更新为 `.dijiang/`
- Extension: session 事件实时刷新状态栏和 widget

### 文档

- AGENTS.md 添加范围纪律规范
- 定义规范工作流模型
- Workflow: worktree 任务完成后要求合并回 main checkout
- Workflow: 将 push 与本地集成分开处理
- 验证循环和记忆生命周期 spec 合并

## [0.6.1] — 2026-06-30

### 新增

- CLI: `dijiang update` 命令（项目更新）
- CLI: 模板更新的 hash 比较机制
- CLI: `dijiang update` 的 GitHub 下载支持
- 运行时 workflow 状态注入（`dijiang workflow-state --json`）
- Workflow 状态 session 日志和记忆注入
- 对等工作流 session 面
- 项目级更新工作流

### 变更

- 所有 CLI 输出本地化为中文
- 生成的 Makefile 使用中文本地化目标
- Configurator: 改进平台运行时 hook 可见性
- Init: 冲突检测和 skills 路径共存

### 修复

- `dijiang update` 直接写入模板文件（不再仅为建议）
- 本地更新从当前目录模板文件读取
- 测试夹具与新更新机制对齐
- 重复生成的 skills 报告为警告（强制移除/刷新）
- CLI 命令边界明确

### 文档

- 公开 workflow 入口点对齐
- 规范工作流模型正式化
- Review 和 channel 命令加入 workflow 模板

## [0.6.0] — 2026-06-29

### 新增

- Rust CLI: 初始实现，支持多平台 configurator
- Trellis 兼容性状态映射，active task 双写
- `dijiang init` 带强制覆盖和冲突检测
- `dijiang status` 命令，显示详细状态
- 五层记忆架构（working → episodic → semantic → procedural → meta）
- `dijiang mem findings/learn/archive`（同步：`mem sync`，列表：`mem list`）
- 带 session skills 集成的 mem evolve/finetune 命令
- Agent channel 系统：`channel spawn/list/send/execute`，超时和并行
- Thompson sampling 内置策略（`mem tactic/record`）
- 对抗式和第一性原理审查命令
- CLI 完整工作流 E2E 集成测试（Phase 5）
- 交互式 Init 模板引擎（Phase 0）
- 4 个工作空间 crate: cli, task, mem, configurator

### 变更

- 脱离 Trellis 独立 —— 自有 `.dijiang/` 状态目录
- 任务状态路径从 `.trellis/` 迁移到 `.dijiang/`
- Skills 和 templates 重构至 `.pi/skills/` 和 `.pi/prompts/`

### 文档

- README 更新：定位指南和规范工作流
- 调研文档：DiJiang 与 Trellis 深度对比报告
- 优化计划和迁移分析已记录

### Pre-0.6 时代（2026-06-22 — 2026-06-28）

初始 skill 文件开发阶段。关键里程碑：
- Skill 优化和命名冲突解决
- Dispatch 分类器：双层架构，支持混合任务
- MUSE 五层记忆架构设计
- 证据-结论绑定原则（hunt, check, dispatch）
- dj-grill 追问策略增强
- dj-hunt 代码定位改进
- Karpathy 代码准则合并
- 范围纪律规范制定
