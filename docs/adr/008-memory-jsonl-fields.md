# 008 — 记忆 JSONL 字段扩展：writer 与 [主题域] 标签

## 状态

accepted

复验：2027-11-05（dj-gov 巡检提出复核）

## 上下文

dj-memory 的五类 JSONL 此前无写入者身份、无主题标签、无元记忆治理操作。三个缺口在多 agent 协作与长期运行下暴露：dj-channel 子代理写同一 JSONL 时无法归因冲突；同主题条目分散累积无合并机制；记忆与技能正文双写导致漂移（记忆里的操作规则与 SKILL.md 修订不同步）。

来源：吸收琅嬛「Agent 五层记忆分类法」（元记忆层缺失）「Shared Memory 污染与冲突控制」（写入者身份）+ Hermes [TAG] 实践，经 dj-absorb 拍板。

## 决策

扩展记忆条目三个约定（向后兼容，全部可选字段，旧条目不强制回填）：

1. **`writer` 字段**：写入者身份（主 agent / 子代理名）——多 agent 写同一 JSONL 时归因
2. **`[主题域]` 首标签**：content 首个方括号短标签（`[ENV]`/`[GIT-SAFETY]`/`[SKILL-名]`），供 grep 过滤
3. **skill 相关记忆用指针不复制**：content 引用技能路径与节名，不复制正文——单一权威源

并建立**元记忆层**治理四操作（lookup 查重 / merge 合并 / expire 过期 / review 复审），执行细化在 `skills/engineering/dj-memory/references/memory-layers.md`，由 dj-gov 完整路径与 finish-work 回顾承载，不新建定时机制。

### 替代方案（三态记录）

- **否决**：给每条记忆加向量索引与召回评分——需要 embedding 运行时，违反 ADR-005 无运行时决策
- **暂缓**：JSONL 之外的独立元数据索引文件——等 merge/expire 操作的实际频率证明纯 grep 不够用时复活
- **否决**：writer 强制必填——单 agent 场景（多数）下是噪音，可选字段 + 多 agent 场景要求即够

## 明确不做

- 不改 dj-memory 五类 JSONL 的文件划分（那是验证过的稳定结构）
- 不做自动后台巩固/定时清理（挂靠既有流程，不新增机制）
- 旧条目不回填（新约定只约束新写入）

## 影响

- 正面：多 agent 写入可归因；记忆-技能双写漂移有了结构防法；元记忆操作有落点
- 负面：写入路径多两个字段判断成本；merge/expire 依赖人工触发（dj-gov 路径），可能积压
- 兼容：全部可选字段，旧读取路径不受影响

## 关联

- `skills/engineering/dj-memory/`（执行方）
- `skills/engineering/dj-loop/references/three-asset-boundaries.md`（槽位边界）
- ADR-005（无运行时约束——本决策在纯文本形态内）
