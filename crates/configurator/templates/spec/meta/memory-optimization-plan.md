# DiJiang Memory System 优化方案

基于 LangHuan 知识库 + 业界实践（Mem0/AgentCore/Zep）的优化设计。

## 核心哲学

引用 LangHuan 的结论：

> **上下文是稀缺资源**。记忆系统不应该预注入所有内容，而应该作为工具让 agent 按需调用。

DiJiang 当前的记忆系统"只存不用"——findings/learnings/patterns 写进了 jsonl 但 workflow 中无人读取。这是首要断裂点。

## 四个优化点

### 1. 记忆召回工具（最高优先级）

**现状**：无检索入口。agent 无法主动查询历史记忆。

**方案**：新增 `dijiang mem recall <query>` 命令，搜索项目本地 memory 中的相关内容。

```
dijiang mem recall "数据库连接超时"
  → findings.jsonl 匹配 "连接池配置不足导致超时"
  → patterns.jsonl 匹配 "数据库故障恢复流程"
  → learnings.jsonl 匹配 "max_connections 需要和连接池联动"
```

**实现**：`ProjectMemory` 新增 `recall(query: &str) -> Vec<ScoredMemory>` 方法，对 findings/learnings/patterns 做关键词匹配（FTS5 或简单 grep），返回评分排序的结果。

**使用方式**：agent 在工作流中按需调用，不是预注入。

### 2. 工作流记忆感知

**现状**：`dj-implement` / `dj-hunt` 的工作流中完全不查询历史教训。

**方案**：在 `dj-implement` 的准备阶段和 `dj-hunt` 的定位阶段，自动调用 `dijiang mem recall` 检索与当前任务相关的历史记录。

```text
dj-implement 准备阶段：
  recall "<task description>" → 返回相关 patterns/findings
  → 如果有匹配，输出"历史相关记录"供参考

dj-hunt 代码定位阶段：
  recall "<error message>" → 返回相关 findings/patterns
  → 如果有匹配，提示"曾遇到过类似问题"
```

### 3. 自动晋升通道

**现状**：`dijiang mem evolve` 需要手动触发，无人记得跑。

**方案**：集成到 `dijiang finish-work` 流程中，作为可选步骤：

```text
finish-work 收尾阶段
  ├── 生成 session closure
  ├── 可选：分析本次 session 是否有可晋升的 pattern
  │   └── 如果有 → dijiang mem evolve（轻量模式）
  └── 提交/归档
```

晋升条件：
- 同一 pattern 在 3+ 次 session 中出现 → 建议晋升为全局 tactic
- 用户纠正（correction）→ 自动晋升到 project pattern

### 4. 记忆条目增强

**现状**：Finding/Learning 只有 `content` + `session_id` + `project`，无标签和分类。

**方案**：新增可选字段：

```rust
struct Finding {
    timestamp, content, session_id, project,
    tags: Vec<String>,        // 新增：分类标签
    scope: MemoryScope,       // 新增：适用范围
    ttl: Option<String>,       // 新增：过期时间
    importance: u8,            // 新增：重要度 1-5
}

enum MemoryScope {
    Project,    // 仅本项目
    Global,     // 可晋升到全局
    Sensitive,  // 敏感，不进全局
}
```

## 技术选型

| 需求 | 方案 | 理由 |
|------|------|------|
| 召回 | grep/FTS5 | 数据量不大（千级），无需向量库。LangHuan 说"不要一开始搞向量数据库" |
| 排序 | 关键词命中数 + 时间衰减 | 简单有效 |
| 存储 | 现有 jsonl + 索引文件 | 不改存储格式，新增 `.index/` 目录辅助检索 |

## 路线图

| 阶段 | 内容 | 工作量 |
|------|------|--------|
| P0 | `dijiang mem recall` 命令 + 关键词检索 | ~200 行 Rust |
| P0 | `dj-implement` / `dj-hunt` 集成 recall | ~50 行 skill 修改 |
| P1 | `finish-work` 集成 evolve | ~100 行 Rust |
| P1 | Finding/Learning tags/scope 字段 | ~50 行 types 修改 |
| P2 | FTS5 索引替代 grep | ~150 行 Rust |
