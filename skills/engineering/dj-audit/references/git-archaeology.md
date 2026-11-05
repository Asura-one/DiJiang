# Git 考古学：5 条命令诊断陌生代码库

> 摘要：接手陌生库或 debt/health profile 审计的前置全景诊断——变更热点、bug 聚类、bus factor、项目活力、救火频率五维；用 git 历史定位「改哪里风险最高」。

## 为什么要先考古

审计陌生库时直接扫代码会把精力平均分配——但风险从来不平均。**变更热点 × bug 聚类的交集**才是最高风险文件：又常改又常坏的文件，投入产出比最高。历史比静态结构更诚实。

## 五条命令

```bash
# 1. 变更热点：谁被改得最频繁（Top 20）
git log --format=format: --name-only | grep -v '^$' | sort | uniq -c | sort -rg | head -20

# 2. bug 聚类：fix 提交集中在哪（Top 20）
git log --format=format: --name-only --grep='fix\|修复\|bug' -i | grep -v '^$' | sort | uniq -c | sort -rg | head -20

# 3. bus factor：核心文件几个人在改（<3 人 = 知识孤岛）
git shortlog -sn -- <关键文件>   # 逐文件看作者分布

# 4. 项目活力：近期提交节奏（近 90 天提交数 vs 之前）
git log --since='90 days ago' --oneline | wc -l

# 5. 救火频率：revert/hotfix/patch 类提交占比
git log --oneline --grep='revert\|hotfix\|回滚\|紧急' -i | wc -l
```

## 五维判读

| 维度 | 信号 | 审计含义 |
|------|------|---------|
| 变更热点 | 高频改动文件 | 优先审：活跃区的新问题最可能还没显形 |
| bug 聚类 | fix 提交集中的文件 | 优先审：反复坏 = 结构性弱点，不是运气差 |
| **热点×聚类交集** | 又常改又常坏 | 最高优先级——两个榜单都上榜的文件单独列 |
| bus factor | 关键文件 1-2 人改 | 报告知识风险：不是代码问题是组织问题 |
| 救火频率 | revert/hotfix 占比高 | 流程问题信号：验证门禁可能失效，关联 dj-regression-guard 的推广建议 |

## 纪律

- 考古只产出**优先级排序**，不产出结论——文件上了热点榜不等于有问题，仍要真读代码
- 输出一节「审计优先级」进报告头部：Top 5 优先审文件 + 一句话理由
- 历史浅的仓库（<100 提交）跳过本节，直接常规扫描

来源：琅嬛「Git 考古学：5 条命令诊断陌生代码库」
