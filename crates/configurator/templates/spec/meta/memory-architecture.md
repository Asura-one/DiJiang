# DiJiang Memory System 设计文档

## 架构总览

DiJiang 的记忆系统分三层，每层有独立的存储位置、生命周期和访问策略：

```
全局层  ~/.dijiang/memory/         跨项目共享   战术经验、统计、备份
项目层  .dijiang/memory/           项目隔离     findings/learnings/corrections
运行时  .dijiang/.runtime/sessions/ 会话临时     session 状态
```

## 三层结构

### 1. 全局层（`~/.dijiang/memory/`）

| 文件 | 类型 | 用途 |
|------|------|------|
| `tactics.json` | L3 语义记忆 | Thompson sampling 策略库，跨项目可用 |
| `ledger.jsonl` | L3 语义记忆 | 策略执行记录，每条带 `project` 字段 |
| `meta/stats.json` | 元数据 | 各项目 memory 统计 |
| `backups/<project>/` | 备份 | `dijiang mem backup` 备份的项目 memory |

**访问规则**：自动注入的只有 `tactics`（Thompson sampling）。其余通过 `dijiang mem stats` / `dijiang mem backup` 显式操作。

### 2. 项目层（`.dijiang/memory/`）

| 文件 | 类型 | 用途 |
|------|------|------|
| `findings.jsonl` | L2 情景记忆 | 项目发现和结论 |
| `learnings.jsonl` | L2 情景记忆 | 学到的经验教训 |
| `corrections.jsonl` | L1 纠正记忆 | 用户纠正记录 |
| `patterns.jsonl` | L4 程序记忆 | 可复用的模式/SOP |
| `sessions.jsonl` | L2 情景记忆 | session closure 记录 |

**访问规则**：天然项目隔离——存在项目 `.dijiang/` 下，不同项目不同目录。

### 3. 运行时层（`.dijiang/.runtime/sessions/`）

Session 运行时状态，关机即清理。不持久化。

## 类型定义

```rust
// L1 — 纠正记忆：用户纠正了 agent 的行为
struct Correction {
    timestamp, session_key, task, source,
    correction: String,   // 纠正内容
    lesson: String,       // 提炼的教训
    scope, confidence, freshness, conflict, actionability  // 元数据
}

// L2 — 情景记忆：session 中的发现
struct Finding {
    timestamp, content, session_id, project
}

// L2 — 学习记录
struct Learning {
    timestamp, content, session_id, project
}

// L2 — Session 关闭记录
struct SessionClosure {
    timestamp, task, summary, verification,
    docs_sync, version_impact, status, confidence,
    attempts: Vec<AttemptEntry>  // 循环追踪
}

// L3 — 语义记忆：Thompson sampling 策略
struct Tactic {
    name, description,
    alpha: u64, beta: u64,  // Beta 分布参数
    source, created_at, last_used
}

// L4 — 程序记忆：可复用模式
struct Pattern {
    name, description, content,
    scope, tags, created_at
}
```

## 操作命令

```
dijiang mem list             列出平台 sessions
dijiang mem sync             同步平台 sessions
dijiang mem findings        追加项目 finding
dijiang mem learn           追加项目 lesson
dijiang mem correction      追加项目 correction（含质量元数据）
dijiang mem tactic          添加全局 tactic
dijiang mem tactics         列出或选择 tactics（Thompson sampling）
dijiang mem record          记录 tactic 执行结果
dijiang mem pattern         添加项目 pattern
dijiang mem patterns        列出项目 patterns
dijiang mem stats           列出全局 memory 统计
dijiang mem backup          备份项目 memory 到全局
dijiang mem evolve          L5 进化：分析 session 提取 tactics
dijiang mem finetune        精细调优（慢）
```

## 隔离策略

| 记忆类型 | 隔离级别 | 原理 |
|---------|---------|------|
| Tactics | 全局共享 | Thompson sampling 跨项目受益 |
| Ledger | 全局 + project 标签 | 统计时按 project 过滤 |
| Findings/Learnings | 项目隔离 | 存在 `.dijiang/memory/` 下 |
| Corrections | 项目隔离 | 同上 |
| Patterns | 项目隔离 | 同上 |
| Session Closure | 项目隔离 | 同上 |

## 访问路径

```
agent session 启动
  ├── ~/.hermes/memories/memory.md     ← Hermes 层（非 DiJiang）
  ├── ~/.hermes/memories/user.md       ← Hermes 层（非 DiJiang）
  ├── dijiang mem tactics              ← DiJiang 全局：Thompson 推荐
  ├── .dijiang/memory/findings.jsonl   ← DiJiang 项目：读取项目记忆
  └── .dijiang/memory/patterns.jsonl   ← DiJiang 项目：读取模式
```

## 晋升通道

```
项目层 → 全局层的唯一路径：dijiang mem backup
项目层 pattern → 全局层 tactic：dijiang mem evolve（自动分析）
```

## 安全与隐私

1. `findings` / `learnings` / `corrections` 存在项目本地，不会跨项目泄漏
2. 全局 `ledger.jsonl` 的 `project` 字段用于统计过滤，不用于自动注入
3. Thompson sampling `tactics` 只有名称和贝叶斯参数，不含项目内容
4. `dijiang mem backup` 是显式操作，不自动备份
