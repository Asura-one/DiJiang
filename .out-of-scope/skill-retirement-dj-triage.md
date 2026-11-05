# 退役: dj-triage（issue 分诊）→ 融合入 dj-split

- **退役日期**：2026-11-05
- **来源**：全库技能健康审计，用户裁量「其实能用，或融合到 dj-split」——取融合路线
- **否决原因**：与 dj-split 的职责重叠被双源证据坐实——①两技能各持一份 95% 相同的 `references/issue-sync.md`（4089B/4208B，124 行差异大半是皮肤级措辞）；②SKILL.md 各有一段近乎逐字的「同步到 issue 跟踪器」节；③输入谱本来就是连续的：triage 吃零散 issue → agent-ready brief，split 吃 PRD → agent-ready 工单，都是「把输入变成可派发工作」的转换器。状态机五角色（needs-triage/needs-info/ready-for-agent/ready-for-human/wontfix）对单人项目过重。
- **最强理由**：issue 分诊本身的纪律有价值（类别先行、验证主张再拷问、冗余/先前拒绝双检查、brief 结构）——所以不是删除而是**降维融合**：dj-split 新增「issue 输入档」（分诊节），triage 的类型映射表与 bug/feat 正文骨架并入 issue-sync.md 单一权威源。杀掉的是重复的载体（第二份 issue-sync + 第二个技能名），留下全部纪律。
- **复活条件**：issue 量大到「分诊」与「拆分」需要不同认知模式且互相干扰（如开源项目接受外部贡献者的 issue 洪流、需要独立分诊队列/独立维护者角色）时，按融合档案反向拆出。

## 融合映射（triage → dj-split 的落点）

| triage 原件 | 融合落点 |
|---|---|
| 类别角色（bug/enhancement）+ 简化状态 | dj-split SKILL.md「issue 分诊档」 |
| 验证主张（bug 先复现）+ 冗余/先前拒绝双检查 | 同上 |
| agent brief 结构 + needs-info 模板 + 快速状态覆盖 | 同上 |
| bug/feat 正文骨架 + 类型映射表 | `dj-split/references/issue-sync.md` |
| 触发词（分诊、triage、issue 处理、新 issue） | dj-split description 与路由表 |
| 状态机五角色完整表述 | 本档案（git log 可溯） |

## 退役时技能快照

- 功能：本地任务队列 issue 分诊状态机（分类→验证→拷问→brief）
- 最后状态：verify PASS；TAIL_POINTERS 的 triage 条目移除（dj-split 已有 → dj-implement 指针）
- 本档案即复活时的最小记忆载体；完整历史见 git log skills/engineering/dj-triage/
