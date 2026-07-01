---
name: dj-write
description: >
  文字润色：去除 AI 味，让中英文文本读起来自然。不改词汇，只删表演。
  Use when the user asks to draft, rewrite, proofread, polish, remove AI-like wording,
  or prepare docs/posts in natural Chinese or English.
  触发词：帮我写、改稿、润色、去AI味、写一段、proofread、polish、rewrite、write。
---

# Write: 去 AI 味

## 职责

去掉文本中的 AI 模式，让它读起来像人写的。不提升词汇量，只删掉"提升词汇量的表演"。

## 核心立场

这不是检查清单，而是气味识别。用这些模式来识别 AI 味，然后做判断。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Source text, target language, audience, desired strength, and whether this is polish / rewrite / draft |
| 输出 | Natural text that preserves intent, plus change notes only when requested or when edits are substantial |
| 非目标 | Do not inflate vocabulary, add new claims, change facts, or make the text sound more formal by default |

## 工作流

### 1. 确认编辑模式

Use the user's wording to choose exactly one mode:

| Mode | Use when | Allowed change |
|---|---|---|
| polish | 润色、去 AI 味、proofread | delete filler, smooth phrasing, keep structure |
| rewrite | 改写、重写、换个说法 | restructure sentences while preserving facts |
| draft | 帮我写、写一段 | create new text from supplied intent |
| proofread | 校对、检查语病 | fix grammar/typos only |

If mode is unclear, default to `polish`. State the mode only when it affects the output.

需要起草、重写、论证或大改结构时，先做第一性原理判断：这段文字真正要让读者相信、理解或行动的核心是什么；哪些事实不能改；哪些表达只是作者习惯或 AI 填充。

### 2. 识别

Read the text and mark AI-flavored patterns:

- 中文：过度渐变、空洞总结、堆砌形容词、营销腔、完美对称、过度连接
- English: empty opener, filler, hedging, buzzwords, parallel padding

### 3. 判断

For each pattern:

```text
delete: meaning unchanged
keep: meaning would change
ask: fact or author intent is ambiguous
```

Do not ask about reversible wording choices. Ask only when removing or rewriting would change facts, tone, audience, or ownership.

### 4. 润色

Default operation is deletion and compression:
- Delete extra connectors, empty summaries, adjective piles, and generic claims.
- Keep author intent, factual claims, terminology, and useful rough edges.
- Do not add examples, features, benefits, or emotional emphasis not present in the source.
- Keep technical terms stable unless the user asks for localization.

对论证类文本做对抗式审查：找逻辑漏洞、事实断点、偷换概念、未证明的强结论和读者会反驳的地方。只在用户要求改稿或大改结构时修这些问题；普通 polish 只报告，不擅自改事实。

### 5. 输出

Default: output only the polished text.

When edits are substantial or user asks for explanation, use:

```text
<polished text>

Changes:
- <changed pattern>: <what changed>
- Preserved: <important intent or term kept>
```

## 特殊场景

### Release Notes
- 用动词开头，不写"我们很高兴发布..."
- 列变更点，不写散文
- 重要变更标注 breaking change

### 社交帖子（推特/微博）
- 一句话说清楚
- 不堆砌 hashtag
- 不用营销腔（"赋能"、"一站式"）
- 可以有个性，但不装

### 产品本地化
- 保持原意，不加也不减
- 目标语言的自然表达，不翻译腔
- 技术术语保持一致

## AI 味识别

### 中文 AI 味

| 模式 | 示例 | 替代 |
|---|---|---|
| 过度渐变 | "首先...其次...最后...综上" | 直接说重点 |
| 空洞总结 | "总而言之，这是一个值得关注的领域" | 删掉 |
| 堆砌形容词 | "强大的、灵活的、可扩展的解决方案" | 选一个最准确的 |
| 营销腔 | "赋能、助力、一站式" | 用具体动词 |
| 完美对称 | 每段一样长、每点平行 | 允许不对称 |
| 过度连接 | "值得注意的是"、"需要指出的是" | 直接说 |

### 英文 AI 味

| 模式 | 示例 | 替代 |
|---|---|---|
| Empty opener | "It's worth noting that..." | 直接说 |
| Filler | "In order to" | "to" |
| Hedging | "It seems like perhaps..." | 删掉或直接表态 |
| Buzzwords | "leverage, synergize, paradigm" | 用简单词 |
| Parallel padding | 每个要点同样长度 | 允许不齐 |

## 规则

1. **保留作者意图** — 删 AI 味，不改意思
2. **不提升词汇** — 删掉"表演提升"，不换成更高级的词
3. **允许不完美** — 人写的文本就是不完美的
4. **结构可以调** — AI 喜欢完美结构，人喜欢自然流动

## 输出

只输出润色后的文本，除非用户要求附带修改说明。

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 去 AI 味后意思变了 | 逐句对比原稿，恢复被改掉的意图 | 只删不改，保留原文结构 |
| 用户觉得改得太少 | 标注更多 AI 味模式，扩大修改范围 | 展示对比表让用户选哪些保留 |
| 用户觉得改得太多 | 恢复用户标记的段落 | 只改用户确认的部分 |
| 中英文混合文本 | 分别按各自规则处理 | 以主要语言为主，另一语言辅助 |

## 🔴 CHECKPOINT · 润色确认

大改后展示对比：
```
修改摘要：
- 改动处：<N>
- 主要改动：<类型列表>

确认保留？(Y/n)
```

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 把"好"改成"卓越" | 保持简单 |
| 加更多连接词让文章"更流畅" | 删掉多余连接词 |
| 让每段长度一致 | 允许自然的长短变化 |
| 加总结段落 | 如果正文已经说清楚了就不加 |
| 用更高级的词替换简单词 | 用词简单直接 |
