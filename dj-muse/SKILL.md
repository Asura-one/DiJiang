---
name: dj-muse
description: "Dual-layer memory system: Long-term memory (MUSE framework: strategic/procedural/tool) + Working memory (Manus mode: task planning/findings/progress). Supports cross-session persistence and session-level progress tracking. **Default mode**: `/dj-muse` + 简短指令 → 自动分析对话、提取经验、写入记忆，不问问题。"
---

# MUSE (Dual-Layer Memory System)

> **Language / 语言**: [English](docs/SKILL.en.md) | [中文](docs/SKILL.zh.md)
>
> Set `MUSE_LANG=zh` (default) or `MUSE_LANG=en` to change output language.

## Architecture Overview

```
+-------------------------------------------------------------------------+
|                    MUSE Skill Architecture (5-Layer)                     |
+-------------------------------------------------------------------------+
|  Layer 5: Meta-Memory (元记忆)                                          |
|  +-------------------------------------------------------------------+  |
|  | 清理过期记录 | 合并重复实体 | 处理冲突事实 | 废弃失效流程      |  |
|  | → muse_memory_distill | muse_learn_review | muse_learn_promote  |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  Layer 4: Procedural Memory (程序记忆)                                 |
|  +-------------------------------------------------------------------+  |
|  | SOP Skills | 可复用流程 | 版本化的工作流                          |  |
|  | → muse_sop_add | muse_sop_list | dj-* skills                     |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  Layer 3: Semantic Memory (语义记忆)                                   |
|  +-------------------------------------------------------------------+  |
|  | Strategic Memory | 稳定事实 | 结构化知识                          |  |
|  | → muse_strategic_write | muse_langhuan_sink | MEMORY.md           |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  Layer 2: Episodic Memory (情节记忆)                                   |
|  +-------------------------------------------------------------------+  |
|  | 带时间的轨迹 | 过去发生过什么 | 经验教训                          |  |
|  | → muse_learn_write | muse_memory_append_today | learnings/        |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  Layer 1: Working Memory (工作记忆)                                    |
|  +-------------------------------------------------------------------+  |
|  | 当前状态 | 对话内容 | 中间推理 | 任务上下文                      |  |
|  | → muse_session_* | task_plan.md | findings.md | progress.md      |  |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

### 5 层记忆对照表

| 层级 | 名称 | 用途 | 生命周期 | MUSE API |
|------|------|------|---------|----------|
| L1 | 工作记忆 | 当前任务的临时状态 | 会话结束清空 | `muse_session_*` |
| L2 | 情节记忆 | 过去发生过什么（带时间戳） | 长期保留 | `muse_learn_write`、`muse_memory_append_today` |
| L3 | 语义记忆 | 稳定的事实和知识 | 持续更新 | `muse_strategic_write`、`muse_langhuan_sink` |
| L4 | 程序记忆 | 可复用的流程和 SOP | 版本化管理 | `muse_sop_add`、dj-* skills |
| L5 | 元记忆 | 管理层：清理、压缩、遗忘 | 定期触发 | `muse_memory_distill`、`muse_learn_review` |

## 默认行为（/dj-muse 直接调用）

当用户输入 `/dj-muse` + 简短指令（如"记忆一下"、"记住这个"、"沉淀一下"）时，**直接执行，不问问题**：

```bash
# 1. 前置检查
~/.config/muse/bin/muse version || echo "muse CLI 不可用"

# 2. 分析当前对话，提取可记录的经验
#    - 错误/排查 → type=ERR
#    - 纠正/学习 → type=LRN  
#    - 新功能/发现 → type=FEAT
#    - 重要决策 → strategic_write

# 3. 写入学习记录（自动判断类型和优先级）
~/.config/muse/bin/muse api muse_learn_write '{
  "type": "<ERR|LRN|FEAT>",
  "summary": "<一句话总结>",
  "details": "<详细描述>",
  "priority": "<high|medium|low>",
  "tags": ["<相关标签>"],
  "suggested_action": "<后续建议>"
}'

# 4. 如果有战略价值，同时写入战略记忆
~/.config/muse/bin/muse api muse_strategic_write '{
  "dilemma": "<问题描述>",
  "strategy": "<解决策略>",
  "reason": "<为什么有效>",
  "tags": ["<相关标签>"]
}'

# 5. 输出确认
echo "✅ 已记录：[ID] <摘要>"
```

### 自动判断规则

| 对话内容特征 | 记录类型 | 优先级 |
|-------------|---------|--------|
| 报错/崩溃/异常 | ERR | high |
| 用户纠正/反馈 | LRN | high |
| 排查过程/调试 | ERR | medium |
| 新发现/新模式 | FEAT | medium |
| 架构决策/策略 | strategic_write | high |
| 优化/改进 | LRN | medium |

### 执行原则

1. **直接执行**：不问"要记录什么类型？"、"优先级？"——自动判断
2. **完整记录**：summary + details + tags + suggested_action，一个都不少
3. **双重写入**：有战略价值时同时写 learn_write + strategic_write
4. **输出确认**：只告诉用户记录了什么，不问问题

---

## 标准工作流（按顺序执行）

### 1. 任务开始

```bash
# 解析工作区 → 初始化记忆 → 创建会话
muse_memory_init '{"workspace":"/path/to/project"}'
muse_session_create '{"project_id":"my-project","task_description":"实现XXX"}'
```

### 2. 执行中（每完成一个阶段）

```bash
# 记录进度
muse_progress_update '{"session_id":"sess-xxx","step":"完成核心实现","status":"completed"}'
# 记录发现
muse_findings_append '{"session_id":"sess-xxx","发现":"发现重要模式..."}'
# 需要参考历史时
muse_retrieve '{"workspace":"/path","q":"关键词"}'
```

### 3. 学到新东西时

```bash
# 写入学习记录（错误/纠正/最佳实践）
muse_learn_write '{"workspace":"/path","type":"LRN","summary":"...","priority":"high"}'
# 写入当日记忆
muse_memory_append_today '{"workspace":"/path","content":"关键发现...","session_id":"sess-xxx"}'
```

### 4. 任务结束

```bash
# 🔴 CHECKPOINT: 确认 summary 完整后再归档
muse_session_archive '{"session_id":"sess-xxx","summary":"完成XXX实现"}'
# 有价值的知识沉淀到琅嬛
muse_session_sink_langhuan '{"session_id":"sess-xxx"}'
```

### 5. 定期维护（cron 自动或手动）

```bash
# 蒸馏旧 daily 文件（先 dry_run）
muse_memory_distill '{"workspace":"/path","dry_run":true}'
# 审查学习记录
muse_learn_review '{"workspace":"/path"}'
```

## Quick Start

Load the script:

```bash
source ~/.config/muse/scripts/muse.sh
# Or from project location:
# source /Users/cimer/Project/sop/skills/muse/scripts/muse.sh
```

> v3 开始，shell 仅做兼容入口，核心逻辑由 Go CLI（`~/.config/muse/bin/muse`）执行。

### 前置检查

在执行任何 MUSE 操作前：

```bash
# 1. 确认 CLI 可用
~/.config/muse/bin/muse version
# 如果失败 → go build -o ~/.config/muse/bin/muse ./cmd/muse/

# 2. 确认 qmd 可用（LangHuan 集成需要）
which qmd || echo "qmd 未安装，LangHuan 集成不可用"

# 3. 确认 workspace 目标路径存在
test -d /path/to/project || echo "项目目录不存在"
```

### 失败恢复表

| 操作 | 失败表现 | 一线修复 | 仍失败兜底 |
|------|---------|---------|-----------|
| `muse_session_create` | "task_description 为必填" | 补全必填参数 | 检查 JSON 格式是否正确 |
| `muse_memory_append_today` | "content 为必填" | 补全 content | 确认 workspace 路径存在 |
| `muse_langhuan_recall` | "LangHuan 知识库未找到" | 设置 `MUSE_LANGHUAN_ROOT` 环境变量 | 手动传 `langhuan_root` 参数 |
| `muse_langhuan_recall` | "qmd 执行失败" | `which qmd` 确认安装 | 降级为 `muse_memory_get_recent` |
| `muse_memory_distill` | "没有超过N天的daily文件" | 降低 `retain_days` | 正常情况，无需处理 |
| `muse_learn_promote` | "需要人类审核" | 设置 `confirm: true` | 不要绕过，必须用户确认 |
| `muse_retrieve` | 无结果返回 | 检查 workspace 是否有 skill cards | 用 `muse_langhuan_recall` 直接搜索 |
| `muse_skill_upsert` | "技能卡片必须包含name与intent" | 补全 card 的 name 和 intent 字段 | 检查 JSON 嵌套结构是否正确 |
| `muse_session_archive` | "会话已归档" | 用 `muse_session_list` 查看状态 | 从 archives 目录直接读取 |
| `muse_reflect` | 返回空结果 | 确认 workspace 下有 learnings/ | 手动创建 `muse_learn_write` |
| `muse_session_create` | 会话创建后工作记忆为空 | 检查 session 目录结构 | 手动创建 task_plan.md/findings.md/progress.md |
| `muse_memory_append_today` | 情节记忆写入失败（daily 文件不存在） | 确认 workspace/memory/ 目录存在 | 手动创建 YYYY-MM-DD.md 文件 |
| `muse_strategic_write` | 语义记忆写入失败（strategic 目录不存在） | 确认 ~/.config/muse/strategic/ 目录存在 | 手动创建 memories.md 文件 |
| `muse_sop_add` | 程序记忆写入失败（SOP 索引损坏） | 检查 sop-index.md 格式 | 重建 sop-index.md |
| `muse_memory_distill` | 元记忆蒸馏失败（daily 文件格式异常） | 检查 daily 文件是否为有效 markdown | 手动修复格式或跳过异常文件 |
| `muse_learn_review` | 元记忆审查失败（learnings 目录为空） | 确认 learnings/ 目录存在 | 手动创建 learnings/ 目录 |

## Storage Structure

```
~/.config/muse/
├── strategic/                    # Strategic memory (dilemma-strategy pairs)
│   └── memories.md              # All strategic memory entries
├── procedural/                   # Procedural memory index
│   └── sop-index.md             # SOP Skill index
├── sessions/                     # Active sessions (working memory)
│   └── <session_id>/
│       ├── task_plan.md         # Task planning
│       ├── findings.md          # Discoveries
│       ├── progress.md          # Progress tracking
│       └── context.md           # Session context
├── workspaces/                   # Workspace-scoped memory (v3)
│   └── <slug>__<hash8>/
│       ├── meta.json
│       ├── memory/
│       │   ├── MEMORY.md
│       │   └── YYYY-MM-DD.md
│       ├── skills/
│       └── logs/
└── archives/                     # Archived sessions
    └── <session_id>/
```

---

## Long-term Memory (MUSE Framework)

### Strategic Memory

Stores dilemma-strategy pairs for recording major decisions and strategies for solving difficult problems.

**Write**:

```bash
muse_strategic_write '{
  "dilemma": "Problem description",
  "strategy": "Solution strategy",
  "reason": "Why this strategy works",
  "tags": ["architecture", "performance"]
}'
```

**Search**:

```bash
muse_strategic_search '{"q": "keyword", "limit": 10}'
muse_strategic_list
```

**When to write**:
- After major architectural decisions
- After solving recurring difficult problems
- When discovering global best practices

### Procedural Memory

Stores SOP Skills (reusable multi-step operation flows).

**Add to index**:

```bash
muse_sop_add '{
  "name": "sop-deploy",
  "description": "Standard deployment flow",
  "path": "~/.claude/skills/sop-deploy/SKILL.md"
}'
```

**List all SOPs**:

```bash
muse_sop_list
```

**When to write**:
- After completing reusable multi-step operations
- When discovering repeated operation patterns

### Tool Memory

Located in the "Next Actions" section at the end of each Skill file, recording common follow-up steps after operations.

---

## Working Memory (Manus Mode)

### Session Management

**Create session**:

```bash
muse_session_create '{
  "project_id": "my-project",
  "task_description": "Implement user authentication"
}'
```

**Resume session**:

```bash
muse_session_resume '{"session_id": "sess-20260124-001"}'
```

**Archive session**:

```bash
muse_session_archive '{
  "session_id": "sess-20260124-001",
  "summary": "Completed user authentication implementation"
}'
```

**List sessions**:

```bash
muse_session_list '{"status": "active"}'
muse_session_list '{"project_id": "my-project"}'
```

### Three-File Operations

| File | Purpose | Update Timing |
|------|---------|---------------|
| task_plan.md | Phases, progress, decisions | After each phase |
| findings.md | Research, discoveries | After any discovery |
| progress.md | Session log, test results | Throughout session |

**Update task plan**:

```bash
muse_plan_write '{"session_id": "sess-xxx", "content": "..."}'
muse_plan_read '{"session_id": "sess-xxx"}'
```

**Append findings**:

```bash
muse_findings_append '{
  "session_id": "sess-xxx",
  "finding": "Discovered an important code pattern..."
}'
```

**Update progress**:

```bash
muse_progress_update '{
  "session_id": "sess-xxx",
  "step": "Completed core implementation",
  "status": "completed"
}'
```

### Key Timing Rules

| Timing | Operation | Description |
|--------|-----------|-------------|
| SessionStart | Create three files | Initialize at task start |
| PreToolUse | Read task_plan | Refresh context before major decisions |
| PostToolUse | Update progress | Record state after file operations |
| Every 2 Actions | Save findings | After visual/browse operations |
| 🔴 Stop | Verify completion | Check all phase statuses before archive |

🛑 **STOP before archive**: Session 归档不可逆。归档前必须：(1) 确认 summary 完整 (2) 检查是否有未保存的 findings (3) 用户确认。

---

## Reflection Workflow

Trigger reflection to extract recordable experiences:

```bash
muse_reflect
```

Check existing memories to prevent duplicates:

```bash
muse_check '{"type": "strategic", "q": "keyword"}'
```

### Reflection Process

1. **Trace execution trajectory** - Identify key decision points
2. **Check existing memories** - Prevent duplicates (important!)
3. **Analyze extractable experiences** - Determine memory type and update method

🔴 **CHECKPOINT**: Show the user WHERE the memory will be written (strategic/daily/skill) and WHAT content will be written. Wait for confirmation.

4. **Execute write** - According to update method
5. **Confirm completion** - Inform how to trigger usage

### Memory Update Methods

| Method | Condition | Operation |
|--------|-----------|-----------|
| Embed | Operation process standard | Embed into operation steps |
| Incremental | Content complementary | Add to existing content |
| Overwrite | Content outdated/wrong | Replace existing content |

---

## Command Reference

| Command | Description |
|---------|-------------|
| `/muse` | Enter memory management mode |
| `/muse reflect` | Trigger reflection, extract recordable experiences |
| `/muse check` | Check existing memories, prevent duplicates |
| `/muse strategic` | View strategic memories |
| `/muse sop` | List all SOP Skills |
| `/muse session` | Manage session working memory |

---

## API 选择决策树

```
用户需求是什么？
├── "记住这件事" → muse_memory_append_today
├── "上次怎么做的？" → muse_retrieve（四路召回）
├── "这个决策很重要" → muse_strategic_write
├── "做个可复用流程" → muse_sop_add
├── "开始新任务" → muse_session_create
├── "任务做完了" → muse_session_archive（🔴 先确认 summary）
├── "出错了/我纠正你" → muse_learn_write（type=ERR/LRN）
├── "把这个知识存起来" → muse_langhuan_sink
├── "旧日记太多" → muse_memory_distill（先 dry_run）
├── "看看学到了什么" → muse_learn_review
├── "把学习变成规则" → muse_learn_promote（需 confirm=true）
└── "session 内容存到知识库" → muse_session_sink_langhuan
```

## API Contract

### Long-term Memory

- `muse_strategic_write(input_json)` -> `{ok, file, entry}`
- `muse_strategic_search(input_json)` -> `{ok, results, count}`
- `muse_strategic_list()` -> `{ok, entries, count}`
- `muse_sop_add(input_json)` -> `{ok, file, name}`
- `muse_sop_list()` -> `{ok, content, count}`

### Workspace / Memory (v3)

- `muse_workspace_resolve(input_json)` -> `{ok, workspace_key, workspace}`
- `muse_memory_init(input_json)` -> `{ok, workspace_key, paths}`
- `muse_memory_read_long_term(input_json)` -> `{ok, workspace_key, file, content}`
- `muse_memory_write_long_term(input_json)` -> `{ok, workspace_key, file}`
- `muse_memory_append_today(input_json)` -> `{ok, workspace_key, file}`
- `muse_memory_get_recent(input_json)` -> `{ok, workspace_key, entries, count, content}`
- `muse_memory_context(input_json)` -> `{ok, workspace_key, context, recent_count, days}`

### Session Management

- `muse_session_create(input_json)` -> `{ok, session_id, path, files}`
- `muse_session_resume(input_json)` -> `{ok, session_id, path, files, context}`
- `muse_session_archive(input_json)` -> `{ok, session_id, archive_path}`
- `muse_session_list(input_json)` -> `{ok, sessions, count}`

### Three-File Operations

- `muse_plan_write(input_json)` -> `{ok, file}`
- `muse_plan_read(input_json)` -> `{ok, content, file}`
- `muse_findings_append(input_json)` -> `{ok, file}`
- `muse_progress_update(input_json)` -> `{ok, file}`

### Reflection Workflow

- `muse_reflect()` -> `{ok, message, existing_memories, files, workflow}`
- `muse_check(input_json)` -> `{ok, results}`

### Skill / Evolution (v3)

- `muse_skill_upsert(input_json)` -> `{ok, workspace_key, file, card}`
- `muse_skill_search(input_json)` -> `{ok, workspace_key, count, results}`
- `muse_skill_extract_from_session(input_json)` -> `{ok, workspace_key, action, candidate}`
- `muse_retrieve(input_json)` -> `{ok, mode, recommended_execution, strategy_rationale, references_risks}`
- `muse_evolution_review(input_json)` -> `{ok, pending|candidate}`
- `muse_migrate_v3(input_json)` -> `{ok, report}`

### LangHuan Integration (v3.2)

- `muse_langhuan_recall(input_json)` -> `{ok, query, mode, count, results, langhuan}`
  - 参数: `q`(必填), `hybrid`(bool, 默认false), `limit`, `collection`, `langhuan_root`
  - hybrid=false 用 BM25 快速搜索, hybrid=true 用 BM25+向量+reranking
- `muse_langhuan_sink(input_json)` -> `{ok, file, fileName, title, langhuan}`
  - 参数: `title`(必填), `content`(必填), `description`, `tags`, `session_id`, `source_type`, `langhuan_root`
  - 创建 OKF 合规的 concept 文件到 LangHuan/concepts/
- `muse_langhuan_update_index(input_json)` -> `{ok, file, entry, category}`
  - 参数: `title`(必填), `file_name`(必填), `description`, `category`, `langhuan_root`

### Memory Lifecycle (v3.2)

- `muse_memory_distill(input_json)` -> `{ok, processed_days, weekly_files, memory_updated, archived_files, distilled_facts}`
  - 参数: `workspace`(必填), `retain_days`(默认7), `dry_run`(bool)
  - daily→weekly 蒸馏, 关键事实沉淀到 MEMORY.md, 归档已处理 daily 文件
- `muse_memory_distill_status(input_json)` -> `{ok, pending_days, weekly_count, archive_count, memory_size, needs_distill}`
  - 参数: `workspace`(必填), `retain_days`(默认7)

### Session → LangHuan (v3.2)

- `muse_session_sink_langhuan(input_json)` -> `{ok, action, session_id, title, file, langhuan}`
  - 参数: `session_id`(必填), `summary`(可选LLM摘要), `langhuan_root`
  - 从 session 数据生成 LangHuan concept 并写入
- `muse_session_enhance_summary(input_json)` -> `{ok, session_id, summary_file}`
  - 参数: `session_id`(必填), `enhanced_summary`(必填, LLM合成的摘要)

### Learning / Self-Improvement (v3.3)

- `muse_learn_write(input_json)` -> `{ok, id, type, file, status}`
  - 参数: `summary`(必填), `type`(LRN/ERR/FEAT), `priority`, `area`, `details`, `suggested_action`, `tags`, `source`, `session_id`, `pattern_key`, `related_files`
  - 写入学习记录到 learnings/ 目录。pattern_key 相同的条目自动合并 recurrence_count
- `muse_learn_list(input_json)` -> `{ok, entries, count, stats}`
  - 参数: `type`(过滤), `status`(过滤), `priority`(过滤), `limit`
- `muse_learn_review(input_json)` -> `{ok, pending, suggested, stats, conflicts, meta_warning}`
  - 参数: `workspace`(必填)
  - 包含元反思：接受率统计、冲突检测、触发条件警告
- `muse_learn_promote(input_json)` -> `{ok, id, target, action}`
  - 参数: `id`(必填), `target`(memory/skill/langhuan), `confirm`(必须为true)
  - 晋升到 MEMORY.md / skill 候选 / LangHuan 知识库

---

## 反例与黑名单（不要做什么）

| # | 反模式 | 为什么不要做 | 替代做法 |
|---|--------|-------------|---------|
| 1 | **不确认 workspace 就写入** | 写入错误目录，数据丢失 | 先 `muse_workspace_resolve` 确认路径，再执行写操作 |
| 2 | **session 归档前不写 summary** | 归档后上下文丢失，无法追溯 | 归档前调用 `muse_session_archive` 带 summary 参数 |
| 3 | **strategic memory 写入临时状态** | 战略记忆是长期决策，临时状态污染长期记忆 | 临时状态写 daily memory，只有反复验证的决策才写 strategic |
| 4 | **跳过 `muse_check` 直接写入** | 产生重复记忆条目 | 写入前先 `muse_check` 检查是否已有同类记录 |
| 5 | **learn_promote 不带 confirm** | Agent 自己改自己的规则，无人审核 | `confirm` 参数必须为 `true`，否则 API 拒绝执行 |
| 6 | **distill retain_days 设太小** | 蒸馏掉还在活跃使用的 daily 文件 | `retain_days` 至少 7，活跃项目建议 14 |
| 7 | **retrieve 关闭 use_langhuan** | 丢失知识库召回源，重复犯已知错误 | 除非 qmd 不可用，否则保持 `use_langhuan: true` |
| 8 | **memory_distill 不设 dry_run 先试** | 直接蒸馏可能误删有价值的 daily 文件 | 首次蒸馏先 `dry_run: true` 查看会处理哪些文件 |
| 9 | **skill_upsert 不带 workspace** | 技能卡写入默认目录而非项目目录 | 始终传 `workspace` 参数指定项目路径 |
| 10 | **session 写入无 session_id** | 记忆条目无法追溯来源 | `memory_append_today` 和 `learn_write` 始终传 `session_id` |

### 危险操作（需用户确认）

| 操作 | 风险 | 确认方式 |
|------|------|---------|
| `muse_learn_promote` | 修改 MEMORY.md 或触发 skill 进化 | `confirm: true` 必填 |
| `muse_memory_distill` (非 dry_run) | 归档 daily 文件，不可逆 | 先 dry_run 查看结果 |
| `muse_langhuan_sink` | 写入外部知识库 | 确认 title/content 准确 |
| `muse_session_archive` | 会话归档后不可编辑 | 确认 summary 完整 |

---

## Next Actions

After completing memory operations:

- After strategic memory write -> Consider updating related CLAUDE.md
- After SOP creation -> Test if SOP flow is reproducible
- After session archive -> Extract valuable experiences to long-term memory
