# DiJiang → Trellis 改造分析报告

> 日期：2026-06-26
> 目标：将 DiJiang 改造为 Trellis 级别的工程工具

---

## 一、Trellis 核心架构解析

### 1.1 系统定位

Trellis 是一个 **AI 辅助开发的工程化管理系统**，核心理念：
- **Plan before code** — 先规划再编码
- **Specs injected, not remembered** — 规范通过 hook/skill 注入，不依赖记忆
- **Persist everything** — 研究、决策、教训全部持久化到文件
- **Incremental development** — 一次一个任务
- **Capture learnings** — 每个任务后回顾并写回规范

### 1.2 五大子系统

| 子系统 | 职责 | 核心文件 |
|--------|------|----------|
| **Developer Identity** | 开发者身份管理 | `.developer`, `workspace/<name>/` |
| **Spec System** | 编码规范（按 package/layer 组织） | `spec/<package>/<layer>/index.md` |
| **Task System** | 任务生命周期管理 | `tasks/{MM-DD-name}/task.json`, `prd.md`, `design.md` |
| **Workspace System** | 跨 session 追踪 | `workspace/<dev>/journal-N.md` |
| **Context System** | 运行时上下文注入 | `get_context.py`, JSONL manifests |

### 1.3 三阶段工作流

```
Phase 1: Plan    → 分类、获取任务创建许可、编写规划产物
Phase 2: Execute → 仅在 task status=in_progress 后实现
Phase 3: Finish  → 验证、更新规范、提交、收尾
```

### 1.4 Task 生命周期

```
planning → in_progress → completed → archived
   ↑            ↓
   └── (回退) ──┘
```

### 1.5 规划产物体系

| 产物 | 用途 | 必要性 |
|------|------|--------|
| `prd.md` | 需求、约束、验收标准 | ✅ 必须 |
| `design.md` | 技术设计（复杂任务） | 复杂任务必须 |
| `implement.md` | 执行计划（复杂任务） | 复杂任务必须 |
| `implement.jsonl` | 子 agent 上下文清单 | 推荐 |
| `check.jsonl` | 检查 agent 上下文清单 | 推荐 |
| `research/` | 调研产物 | 可选 |

---

## 二、DiJiang 现状分析

### 2.1 系统定位

DiJiang 是一个 **AI 编码 Skill 集合**，核心理念：
- 三层合一：流程骨架 + 编码风格 + 具体习惯
- Skill 可组合、可单独使用
- Runtime-neutral（不绑定特定 agent）

### 2.2 现有 Skill 清单

| 类别 | Skill | 功能 |
|------|-------|------|
| **核心流程** | dj-dispatch | 任务分类路由 |
| | dj-grill | 需求对齐 |
| | dj-output | 文档创建 |
| | dj-implement | 代码实现 |
| | dj-tdd | 测试驱动 |
| | dj-hunt | Bug 排查 |
| | dj-check | 代码审查 |
| **辅助** | dj-ponytail | 极简编码 |
| | dj-write | 文字润色 |
| | dj-design | UI 设计 |
| | dj-absorb | 吸收融合 |
| | dj-prototype | 废品验证 |
| | dj-script | 脚本编写 |
| | dj-audit | 全仓审计 |
| | dj-debt | 技术债追踪 |
| | dj-health | Agent 健康检查 |
| | dj-handoff | Session 交接 |
| **记忆** | dj-muse | 长期记忆系统 |

### 2.3 工作流

```
想法 → dj-dispatch（自动分流）
        │
        ├── S级 → 直接干
        ├── M-simple → dj-implement
        ├── M-phased → dj-grill → dj-tdd → dj-check
        └── L级 → dj-grill → dj-output → dj-tdd → dj-check
```

---

## 三、核心差异分析

### 3.1 差异总览

| 维度 | Trellis | DiJiang | 差距 |
|------|---------|---------|------|
| **任务管理** | 结构化 task.json + 生命周期 | 无，依赖 session 上下文 | 🔴 严重缺失 |
| **开发者身份** | .developer + workspace 追踪 | 无 | 🔴 缺失 |
| **规范系统** | spec/ 按 package/layer 组织 | 无独立规范系统 | 🔴 缺失 |
| **Session 追踪** | journal-N.md 自动记录 | dj-muse（手动触发） | 🟡 部分覆盖 |
| **规划产物** | prd.md + design.md + implement.md | dj-grill 输出 + dj-output | 🟡 有但不结构化 |
| **上下文注入** | JSONL manifests + get_context.py | 无，依赖 skill 加载 | 🔴 缺失 |
| **CLI 工具** | 完整 Python CLI（task.py 等） | 无 CLI | 🔴 缺失 |
| **Git 集成** | worktree + branch + PR 创建 | git-safety 规则（skill 内） | 🟡 有但不自动化 |
| **子 Agent 调度** | 原生支持 implement/check 子 agent | dj-dispatch 路由 | 🟡 有但不结构化 |
| **Hook 系统** | 平台 hook 注入工作流状态 | 无 | 🔴 缺失 |
| **归档系统** | archive/{year-month}/ | 无 | 🔴 缺失 |
| **代码规范** | spec/ 目录，按层组织 | 分散在各 skill 中 | 🟡 有但不集中 |

### 3.2 关键差异详解

#### 3.2.1 任务管理系统（最大差距）

**Trellis**:
```json
{
  "id": "skill-optimization",
  "title": "Skill Optimization and Evaluation",
  "status": "in_progress",
  "priority": "P2",
  "creator": "tiezhu",
  "assignee": "tiezhu",
  "createdAt": "2026-06-22",
  "branch": null,
  "parent": null,
  "children": [],
  "subtasks": []
}
```

**DiJiang**: 无任务管理，每次 session 独立，无法追踪任务状态。

#### 3.2.2 规范注入系统

**Trellis**:
```
.trellis/spec/
├── frontend/
│   ├── index.md          # 入口 + Pre-Development Checklist
│   ├── component-guidelines.md
│   ├── hook-guidelines.md
│   └── ...
├── backend/
│   ├── index.md
│   ├── error-handling.md
│   └── ...
└── guides/
    └── cross-layer-thinking-guide.md
```

规范通过 `get_context.py` 自动注入到 agent prompt。

**DiJiang**: 编码规范分散在 dj-ponytail、dj-implement 等 skill 中，无集中管理。

#### 3.2.3 Session 追踪

**Trellis**:
```bash
python3 ./.trellis/scripts/add_session.py \
  --title "Title" \
  --commit "hash" \
  --summary "Summary"
```

自动生成 `journal-N.md`，每文件最大 2000 行。

**DiJiang**: dj-muse 是手动触发的记忆系统，无自动 session 记录。

#### 3.2.4 工作流状态机

**Trellis**: 通过 `[workflow-state:STATUS]` 标签实现 per-turn breadcrumb：
```
[workflow-state:no_task]    → 无活跃任务
[workflow-state:planning]   → Phase 1
[workflow-state:in_progress] → Phase 2 + 3
[workflow-state:completed]  → 完成
```

**DiJiang**: 通过 dj-dispatch 路由，无持久化状态。

---

## 四、改造方案

### 4.1 改造目标

将 DiJiang 从 **Skill 集合** 升级为 **工程化管理系统**，保留 DiJiang 的 Skill 生态，增加 Trellis 的工程化能力。

### 4.2 架构设计

```
.dijiang/
├── .developer                    # 开发者身份
├── workflow.md                   # 工作流定义（类似 Trellis）
├── spec/                         # 代码规范（按项目/层组织）
│   ├── frontend/
│   │   ├── index.md
│   │   └── ...
│   └── backend/
│       ├── index.md
│       └── ...
├── tasks/                        # 任务管理
│   ├── {MM-DD-name}/
│   │   ├── task.json
│   │   ├── prd.md
│   │   ├── design.md
│   │   ├── implement.md
│   │   ├── research/
│   │   ├── implement.jsonl
│   │   └── check.jsonl
│   └── archive/
│       └── {YYYY-MM}/
├── workspace/                    # 开发者工作区
│   └── {developer}/
│       ├── index.md
│       └── journal-N.md
├── scripts/                      # CLI 工具
│   ├── task.py                   # 任务管理 CLI
│   ├── get_context.py            # 上下文注入
│   ├── add_session.py            # Session 记录
│   └── common/
│       ├── types.py
│       ├── paths.py
│       ├── config.py
│       └── ...
└── hooks/                        # 平台 Hook
    └── inject-workflow-state.py
```

### 4.3 分阶段实施计划

#### Phase 1: 基础设施（1-2 天）

**目标**: 建立 `.dijiang/` 目录结构和核心 CLI

1. **创建目录结构**
   ```
   .dijiang/
   ├── .developer
   ├── workflow.md
   ├── spec/
   ├── tasks/
   ├── workspace/
   └── scripts/
   ```

2. **实现核心 CLI**
   - `task.py` — 任务 CRUD（create, start, finish, archive, list）
   - `get_context.py` — 上下文注入
   - `add_session.py` — Session 记录

3. **定义 workflow.md**
   - Phase 1: Plan（dj-grill + 规划产物）
   - Phase 2: Execute（dj-implement + dj-tdd）
   - Phase 3: Finish（dj-check + 归档）

#### Phase 2: 任务系统集成（2-3 天）

**目标**: 将 DiJiang Skill 与任务系统打通

1. **改造 dj-dispatch**
   - 增加任务创建逻辑
   - 自动设置 active task
   - 与 workflow state 联动

2. **改造 dj-grill**
   - 输出写入 `prd.md`
   - 支持 `design.md` 和 `implement.md`
   - 调研结果写入 `research/`

3. **改造 dj-implement**
   - 读取 task artifacts
   - 实现结果关联到 task
   - 支持 JSONL 上下文注入

4. **改造 dj-check**
   - 对照 PRD 验收
   - 结果写入 task

#### Phase 3: 规范系统（1-2 天）

**目标**: 建立集中式代码规范管理

1. **创建 spec 目录结构**
   - 按项目类型（frontend/backend/fullstack）
   - 按层（component/hook/state/api/db）

2. **迁移现有规范**
   - 从 dj-ponytail 提取编码规范
   - 从 dj-implement 提取实现规范
   - 从 dj-check 提取审查规范

3. **实现规范注入**
   - `get_context.py --mode packages` 列出可用规范
   - 在 Phase 2 自动注入相关规范

#### Phase 4: Session 追踪（1 天）

**目标**: 实现自动 Session 记录

1. **实现 journal 系统**
   - 每个 session 自动生成 journal entry
   - 跨 session 追踪

2. **集成 dj-muse**
   - journal 作为 dj-muse 的数据源
   - 自动提取关键决策和教训

#### Phase 5: Hook 系统（1-2 天）

**目标**: 实现工作流状态自动注入

1. **实现 workflow state 管理**
   - 状态机：no_task → planning → in_progress → completed
   - 状态持久化

2. **实现 Hook**
   - 每个 turn 自动注入当前状态
   - 状态驱动 skill 路由

#### Phase 6: Git 集成增强（1 天）

**目标**: 自动化 Git 操作

1. **Worktree 自动管理**
   - 任务创建时自动创建 worktree
   - 归档时自动清理

2. **PR 创建**
   - `task.py create-pr` 自动生成 PR

### 4.4 与现有 Skill 的兼容性

| Skill | 改造方式 | 兼容性 |
|-------|----------|--------|
| dj-dispatch | 增加任务创建逻辑 | ✅ 完全兼容 |
| dj-grill | 输出写入 task artifacts | ✅ 完全兼容 |
| dj-implement | 读取 task artifacts | ✅ 完全兼容 |
| dj-check | 对照 PRD 验收 | ✅ 完全兼容 |
| dj-muse | 集成 journal 系统 | ✅ 完全兼容 |
| dj-tdd | 无变化 | ✅ 完全兼容 |
| dj-hunt | 无变化 | ✅ 完全兼容 |
| 其他 | 无变化 | ✅ 完全兼容 |

### 4.5 与 Trellis 的差异（保留 DiJiang 特色）

| 维度 | Trellis | DiJiang（改造后） | 原因 |
|------|---------|-------------------|------|
| **任务分级** | 无 | S/M/L 三级 | DiJiang 特色，保留 |
| **Skill 生态** | 无 | 16+ skills | DiJiang 核心价值 |
| **记忆系统** | journal only | journal + dj-muse | 双层记忆更强 |
| **编码规范** | spec only | spec + dj-ponytail | 极简+规范双保险 |
| **Darwin 优化** | 无 | dj-darwin 集成 | Skill 自进化能力 |

---

## 五、实施优先级

### P0（必须，立即开始）
1. 创建 `.dijiang/` 目录结构
2. 实现 `task.py` CLI
3. 定义 `workflow.md`

### P1（重要，Phase 2）
1. 改造 dj-dispatch 集成任务系统
2. 改造 dj-grill 输出 task artifacts
3. 实现 `get_context.py`

### P2（增强，Phase 3-4）
1. 建立 spec 规范系统
2. 实现 session 追踪
3. 集成 dj-muse

### P3（优化，Phase 5-6）
1. 实现 hook 系统
2. 增强 Git 集成
3. PR 自动创建

---

## 六、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 改造后 Skill 失效 | 🔴 高 | 逐个改造，保持向后兼容 |
| 用户习惯改变 | 🟡 中 | 保留原有触发词和交互方式 |
| 性能下降 | 🟡 中 | CLI 按需加载，不强制使用 |
| 与 Trellis 冲突 | 🟢 低 | 独立目录，可共存 |

---

## 七、成功标准

1. ✅ 任务可追踪：每个任务有 task.json，状态可查询
2. ✅ 规划可持久：prd.md / design.md / implement.md 自动保存
3. ✅ 规范可注入：相关规范自动注入 agent prompt
4. ✅ Session 可追溯：journal 自动记录，跨 session 可查
5. ✅ 工作流可驱动：状态机驱动 skill 路由
6. ✅ 现有 Skill 兼容：所有 dj-* skill 正常工作

---

## 八、下一步行动

1. **确认改造方案** — 与老板确认优先级和范围
2. **创建目录结构** — 建立 `.dijiang/` 骨架
3. **实现 task.py** — 核心 CLI 工具
4. **定义 workflow.md** — 工作流状态机
5. **改造 dj-dispatch** — 集成任务系统

---

*本报告基于对 Trellis 源码的深度分析和 DiJiang 现状的全面评估。*
