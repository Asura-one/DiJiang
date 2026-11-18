# 触发层测试：description 是路由杠杆

> 摘要：技能内容再好，description 不命中就永远不加载。本文件给三层评估法（结构 / 触发 / 行为）、配对案例纪律与一次结构体检基线。**不测就别报数字**。
>
> 来源：Addy Osmani `agent-skills/evals/`（MIT）实测 + 琅嬛《Agent Skills 进阶设计模式》《Skill Token Optimization 五维度质量工程》；DiJiang 2026-11-18 结构体检。

## 为什么单独测触发层

description 是注入层的常驻内容，也是路由引擎唯一的匹配信号。两类失败都真实发生过：

- **假阴性**：用户说的原话不在 description 里，模型自己把活干了，技能的流程/判据全部没生效。上游实测：给 code-review 的 description 加一句「即使 diff 是内联粘贴的也适用」，四种提法命中从 7/27 → 21/27（`claude-opus-5-5`，Claude Code 2.1.278）。
- **假阳性**：description 写成能力清单（「能做 A/B/C/D」），任何时候都像适用，真正相邻的请求被抢走。

结论：description 写**什么时候该用**（含用户会说的原话），不写能力清单。

## 三层评估

| 层 | 测什么 | 成本 | 什么时候跑 |
|---|---|---|---|
| **结构层** | 有触发词吗？有负向边界吗？长度是否失控？与相邻技能是否撞词？ | 零（脚本/grep） | 每次改 description、每次瘦身收尾 |
| **触发层** | 给定真实用户提法，该技能是否排到第 1；相邻提法是否**不**命中 | 中（需评测入口） | 新增/改名技能、路由误判复盘 |
| **行为层** | 命中后输出是否真的更好（对照裸模型） | 高（两次调用 + 判分） | 技能重大改写、进化候选验收 |

DiJiang 现有：行为层有 `tests/behavior-cases.json` + 门禁突变测试；结构层见下；**触发层尚无独立评测入口**——TeleAgent 的 available_skills 是注册时冻结的快照，重注册不刷新（2026-11-18 实测），所以不能在会话内自测火率。本节只给方法，不伪造实测数字。

## 配对案例纪律（触发层）

每个触发子句必须配一对案例，否则边界无人守：

- `fires-on-X`：用户会用的一种真实提法（原话照抄进案例，不改写成书面语）
- `stays-quiet-on-Y`：**相邻但不该命中**的提法（如「加测试」对 code-review、「修这个 bug」对 TDD）

硬规则三条：

1. **钉死版本**：报告任何命中率必须同时写模型名与 CLI/平台版本。上游同一条 case 在 2.1.283 与 2.1.312 之间从 6/6 掉到 1/6——不钉版本的数字没有意义。
2. **按 case 看，不看平均分**：单个提法 0/9 与 9/9 会被平均掉；只看总分等于自欺。
3. **被拒的改动记台账**：日期 / 技能 / 尝试 / 排名前后分 / 结局（进 `.dijiang/evolution-rejections.md`）。同一措辞第二次被提时先读台账。

## 结构层 Runbook

1. **列全 description**：
   ```bash
   python3 - <<'EOF'
   import re,glob,os
   for f in sorted(glob.glob('skills/*/*/SKILL.md')):
       d=re.search(r'^description:\s*(.*)$',open(f,encoding='utf-8').read(),re.M)
       print(f"{os.path.basename(os.path.dirname(f)):24} {len(d.group(1)) if d else 0:4} {d.group(1)[:80] if d else 'MISSING'}")
   EOF
   ```
2. **四查**：① 有无触发词（缺 = 只能靠语义碰运气）；② 有无负向边界（「不用于 X」明确排除相邻场景）；③ 长度是否 > 120 字符（不是硬线，但超过通常掺了能力清单）；④ 同族是否撞词（check / audit / hunt；split / wayfinder；implement / tdd）。
3. **只改有证据的**：路由误判复盘里出现过的错配才加边界，不为了整齐给 44 个技能各补一句——那是把瘦身成果又吹回去。
4. **改完复跑结构层**，并把结论写进当次变更说明。

## 2026-11-18 结构体检基线（44 技能）

| 查项 | 结果 |
|---|---|
| 有触发词 | 44 / 44 |
| description 平均长度 | 109 字符 |
| 超 120 字符 | 16 个（dj-gov 230、dj-split 178、dj-verify 174、dj-ask 169……） |
| description 内含显式负向边界 | 2 个（dj-read「不用于仓库里已有的本地文本文件」、dj-gov「区别于 dj-output」） |

判读：11-18 瘦身把「Use when…」从句压掉后，**负向边界只剩 2 处**——这是精简的已知代价（context-diet 首跑只抢救回 dj-read 一条）。当前不批量补，按下列高歧义对优先补边界（出现真实误路由时）：

| 易混对 | 建议边界写法（只在实际误判后加） |
|---|---|
| dj-check ↔ dj-audit | check 审一次交付的 diff；audit 扫全仓存量，只报告不改 |
| dj-check ↔ dj-hunt | check 是交付前闸门；hunt 用于已经坏掉、需要根因调查 |
| dj-split ↔ dj-wayfinder | split 拆已清晰的 PRD；wayfinder 用于路还没看清的大块工作 |
| dj-implement ↔ dj-tdd | implement 按计划推进并自带 TDD；单独要红绿循环时用 tdd |
| dj-output ↔ dj-gov | output 新建文档；gov 对账已有文档与实现是否一致 |

## 与 dj-evolve 的接口

- 进化候选的验证步骤加一问：**这次改动动到 description 了吗？** 动了就跑结构层，并记录是否影响已有触发词。
- 触发层实测目前只能在有独立评测入口时做；没有就写 `not run + 原因`，不写「应该没问题」。