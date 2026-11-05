# 记忆分层与元记忆层治理

> 摘要：dj-memory 五类 JSONL 与 Agent 五层记忆分类的映射、元记忆层（Meta-Memory）的四操作治理（去重/合并/过期/复审）；ADR-008 的执行细化。

## 五层映射（分类自检用）

| Agent 记忆层 | DiJiang 载体 | 说明 |
|-------------|-------------|------|
| 工作记忆（Working） | 当前上下文 + `active_task.txt` | 这一轮看见什么 |
| 情节记忆（Episodic） | `sessions.jsonl` + `.dijiang/tasks/` | 发生过什么 |
| 语义记忆（Semantic） | `findings.jsonl` / `learnings.jsonl` | 知道什么 |
| 程序记忆（Procedural） | `skills/`（终极形态）+ `patterns.jsonl`（暂存） | 会做什么 |
| **元记忆（Meta）** | **本文件治理的对象** | 如何管理记忆自身 |

写入前自问一层归属：写错层 = three-asset-boundaries.md 的退化模式。

## 元记忆层（此前整层空白）

记忆库不是只进不出的堆场——需要**管理记忆自身的记忆**。四操作（dj-gov 完整路径或 finish-work 回顾时执行）：

| 操作 | 触发 | 动作 |
|------|------|------|
| **lookup（查重）** | 每次写入前 | grep 同主题既有条目；已有且一致 → 跳过；已有但矛盾 → 更新原条目而非追加 |
| **merge（合并）** | 同主题 ≥3 条分散条目 | 合并为一条综合条目，保留各条 sources；原条目标记 superseded |
| **expire（过期）** | 环境事实变更（换机/升级/端口迁移） | 旧条目标 `expired` 保留（历史事实不删），新条目显式取代 |
| **review（复审）** | 条目带 `last_confirmed` 超 6 个月未刷新 | 高价值条目下次命中时验证后刷新；长期不命中的候选降级 |

## 字段约定（ADR-008）

- **`writer`**：多 agent（dj-channel 子代理）写同一 JSONL 时记录写入者身份——冲突归因与信任分级的前提；主 agent 写入可省略
- **`[主题域]` 首标签**：content 首个方括号短标签（`[ENV]` `[GIT-SAFETY]` `[SKILL-名]`…），供 grep 快速过滤
- **skill 相关记忆用指针不复制**：content 写「见 skills/xxx 的 Y 节」而非复制技能正文——防记忆与技能双写漂移（单一权威源原则）
- **`last_confirmed`**：条目最近一次被验证的时间戳，元记忆 review 操作的依据

## 不做的

- 显著度衰减/闪回召回/向量检索——需 embedding 运行时，超出纯 skill 库形态
- 自动后台巩固——巩固动作挂在 dj-gov/finish-work 的既有流程上，不新建定时机制

来源：琅嬛「Agent 五层记忆分类法」「Agent 记忆质量门」+ Evolver OPD management 四操作 + Hermes [TAG] 前缀实践
