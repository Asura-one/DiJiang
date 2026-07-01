---
name: dj-pattern
description: >
  模式识别：从历史修复、代码模式中学习，发现可复用的抽象和需要改进的反模式。
  Use when investigating recurring issues, code duplication, or inconsistent patterns.
  触发词：pattern、模式、重复、重构、copy-paste、复用、recurring、repetitive。
---

# Pattern: 模式识别

## 职责

分析代码库中的模式和反模式，输出结构化报告：

1. **重复模式报告** — 识别重复代码和相似实现
2. **历史修复分析** — 从 git log 中发现重复的 bug 模式
3. **抽象建议** — 推荐可复用的共享代码
4. **一致性检查** — 识别相似功能的实现差异

只报告，不修改。一次性分析。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Scan scope, source type, target language, and user goal for pattern discovery |
| 输出 | Pattern report with evidence, YAGNI decision, affected files, and optional next-skill recommendation |
| 非目标 | Do not edit code, create abstractions, or run broad audits unrelated to recurring patterns |

## 扫描范围

```text
scope: <目录/文件/函数>
source: <代码扫描 / git 历史 / 两者>
language: <detected or specified>
goal: <重复 / 历史修复 / 抽象建议 / 一致性>
```

默认全仓扫描。可限定到目录。

## 🔴 CHECKPOINT · 模式扫描范围

扫描前先报告：

```text
Scope: <path or whole repo>
Source: <code / git history / both>
Goal: <analysis goal>
Output limit: <top N findings>
Will modify code: no
```

🛑 STOP if the user asked for direct refactoring. Finish the pattern report first and mark implementation as follow-up; do not edit inside `dj-pattern`.

## 重复模式扫描

### 方法

```bash
# 1. 找代码相似性（按文件大小 + 行数过滤）
find . -name "*.rs" -not -path "./target/*" | xargs wc -l | sort -rn | head -20

# 2. 找同名函数在不同文件的实现
grep -rn "fn [a-z_]*(" src/ --include="*.rs" | awk '{print $1}' | sort | uniq -c | sort -rn | head -20

# 3. 找结构体/枚举定义
grep -rn "struct\|enum " src/ --include="*.rs" | head -30
```

### 判定标准

| 等级 | 定义 | 操作 |
|------|------|------|
| 精确重复 | 完全相同代码块出现 >=2 次 | 建议提取函数/常量 |
| 结构相似 | 相同逻辑但命名/类型不同 | 建议泛型/宏/枚举 |
| 概念相似 | 类似功能用不同方式实现 | 建议统一实现 |
| 模式变异 | 同一算法有多个变体 | 评估是否需要统一 |

### 输出格式

```
<file>:<line> <tag> <模式描述>. <建议>.

<file>:<line> duplicate: 与 <other>:<line> 完全重复. 建议提取为 shared::util::xxx().
<file>:<line> similar: 与 <other>:<line> 结构相似但命名不同. 建议泛型化.
<file>:<line> variant: 实现与 <other>:<line> 有 <N> 处差异. 评估是否需要统一.
```

## 历史修复分析

### 方法

```bash
# 1. 找频繁修改的文件（hot files）
git log --oneline --name-only HEAD~50..HEAD | grep -v "^$" | sort | uniq -c | sort -rn | head -15

# 2. 找修复类提交
git log --oneline --grep="fix\|bug\|crash\|regression\|repair" --all | head -15

# 3. 找变更集中的模块
git log --oneline --name-only --diff-filter=M HEAD~100..HEAD | grep -v "^$" | awk -F/ '{print $1}' | sort | uniq -c | sort -rn | head -10
```

### 输出格式

```
HOT FILES（过去 50 次提交中高频修改）:
  src/core/xxx.rs edited <N> times — <分析>

FIX PATTERNS:
  <提交>: <fix 内容> — <分类到模式名>
  ...

模式归类：
  <模式名> — 出现 <N> 次 — 在 <file> 中出现
```

## 抽象建议

基于发现出的模式，给出具体的重构建议：

```
TO ABSTRACT:
  <模式描述>
  Affected: <文件列表>
  Suggested: <提取/泛型/宏/共享模块>
  Estimated: -<N> lines, -<N> duplications
```

If the abstraction is not clearly necessary, mark it as `YAGNI: <reason> — skipped`.

## 🔴 CHECKPOINT · 建议门禁

推荐提取或共享抽象前先检查：

```text
Evidence count: <N occurrences>
Change history: <same bug/fix repeated? yes/no>
Coupling risk: <low/medium/high>
Simpler alternative: <leave as-is / local helper / docs / shared abstraction>
Recommendation: <skip / investigate / extract>
```

Recommend extraction only when duplication is harmful, repeated, and likely to change together. Otherwise report the pattern and skip the abstraction.
## 一致性检查

识别同一项目/模块中不一致的实现方式：

```
INCONSISTENT:
  <功能>: <file1> 用 <方式A>, <file2> 用 <方式B>
  <建议统一方式>
```

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 扫描范围太大 | 缩小到指定目录或模块 | 先扫最核心的 src/ |
| git 历史太少 | 只用代码静态分析 | 用 find/grep 找重复 |
| 项目不是代码项目 | 跳过代码分析 | 仅输出"无分析对象" |
| 发现大量重复报告过多 | 按等级排序，每个等级只展示 top 5 | 只展示 DRP 精确重复 |

## 边界

- 只报告，不修改
- 不分析依赖（那是 dj-audit 的事）
- 不分析性能（那是 dj-hunt 的事）
- 建议的抽象必须 YAGNI 验证过（不是大炮打蚊子）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|-----------|------------|
| 发现重复就建议提取 | 先评估重复是否真的有害 |
| 把所有相似代码建议抽象 | 只在 >=3 次时建议 |
| 分析完直接改代码 | 只报告，用户决定 |
| 只做静态分析不看 git 历史 | 两者结合，git 历史揭示真问题 |
