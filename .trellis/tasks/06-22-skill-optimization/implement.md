# Skill Optimization Implementation Plan

## 执行计划

### 阶段 1: 初始化（Phase 0）

1. **确认优化范围**
   - 基于现有基线，优先优化低分 skill
   - 建议范围：dispatch (81.9), tdd (81.9), ponytail (81.9), prototype (81.9)
   - 记录到 prd.md

2. **创建 git 分支**
   - 分支名：auto-optimize/YYYYMMDD-HHMM
   - 命令：`git checkout -b auto-optimize/YYYYMMDD-HHMM`
   - 状态：已创建 `auto-optimize/20260622-1000`

3. **初始化 results.tsv**
   - 状态：已存在，包含 round4 评估数据
   - 备份到 `.claude/skills/darwin-skill/results.tsv`

### 阶段 2: 测试 Prompt 设计（Phase 0.5）

1. **为每个 skill 设计测试 prompt**
   - 读取 SKILL.md，理解功能
   - 设计 2-3 个典型用户 prompt
   - 保存到 skill 目录/test-prompts.json

2. **展示给用户确认**
   - 列出所有测试 prompt
   - 用户确认后进入评估

### 阶段 3: 基线评估（Phase 1）

1. **结构评分（维度 1-7、9）**
   - 主 agent 逐项打分
   - 记录评分理由
   - 状态：已有 round4 评估，但为 dry_run

2. **效果评分（维度 8）**
   - 独立子 agent 评估
   - 对比 baseline 和 with_skill 输出
   - 计算加权总分
   - 需要补充 full_test 验证

3. **记录到 results.tsv**
   - 写入基线评估结果
   - 展示评分卡给用户确认

### 阶段 4: 优化循环（Phase 2）

1. **按分数排序**
   - 从低到高排序
   - 优先优化最低 5-10 个

2. **优化每个 skill**
   - 找出得分最低的维度
   - 提出改进方案
   - 执行改进
   - 重新评估
   - 决策：保留或回滚

3. **人审检查点**
   - 每个 skill 优化完后暂停
   - 展示改动摘要和分数变化
   - 用户确认后继续

### 阶段 5: 汇总报告（Phase 3）

1. **生成优化报告**
   - 总览：优化 skill 数、实验次数、保留改进、回滚次数
   - 分数变化表格
   - 主要改进列表

2. **生成成果卡片**
   - 使用 darwin-skill 的模板
   - 截图保存为 PNG

3. **记录到 results.tsv**
   - 写入最终结果

## 验证命令

```bash
# 检查 git 分支
git branch --show-current

# 检查 results.tsv
cat results.tsv

# 检查 skill 目录
ls -la */

# 检查测试 prompt
cat */test-prompts.json
```

## 回滚点

- 如果优化效果不佳，回滚到基线评估版本
- 使用 `git revert HEAD` 创建反向 commit
- 保留可追溯链

## 时间估算

- 阶段 1: 初始化 - 5 分钟（已完成）
- 阶段 2: 测试 Prompt 设计 - 30 分钟
- 阶段 3: 基线评估 - 60 分钟（需补充 full_test）
- 阶段 4: 优化循环 - 120 分钟
- 阶段 5: 汇总报告 - 30 分钟

总计：约 4 小时