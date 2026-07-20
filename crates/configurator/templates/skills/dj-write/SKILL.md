---
name: dj-write
description: >
summary: 文字润色：去除 AI 味，让文本读起来自然
phases: [align, check, finish]
risk: low
  文字润色：去除 AI 味，让中英文文本读起来自然。不改词汇，只删表演。
  Use when the user asks to draft, rewrite, proofread, polish, remove AI-like wording, or improve tone of text.
  触发词：润色、改文字、rewrite、polish、去 AI 味、改语气、写得更自然。
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 去 AI 味的自然文本 |
| **Done when** | 原文按写作立场审查完毕，AI 味标记段落已处理 |
| **Evidence** | 修改前后对比 |
| **Output** | 润色后的最终文本 |

# Write: 文字润色

去除 AI 味，让中英文文本读起来自然。不改词汇，只删表演。

## 核心立场

好文字像玻璃——读者看穿它看到内容，而不是注意到文字本身。

## 工作流

1. 读一遍原文，标记"AI 味"段落
2. 删：
   - 多余的副词（值得注意的是、不可避免地、本质上）
   - 表演性过渡（不仅...而且、总而言之、换句话说）
   - 虚假的谦逊（希望这对你有帮助、我建议）
   - "深入"类空洞词（深入分析、深度探讨）
3. 改：
   - 被动 → 主动
   - 长句 → 短句（超过 30 字拆开）
   - 模糊 → 具体
4. 保持原文的词汇选择和信息量——只改表达方式

## AI 味识别

| AI 特征 | 例子 | 改法 |
|---|---|---|
| 冗余副词 | "值得注意的是，这个方案" | "这个方案" |
| 空洞强调 | "本质上就是" | 直接说是什么 |
| 过度过渡 | "不仅提高了性能，而且还降低了成本" | "性能提升，成本降低" |
| 虚伪谦虚 | "希望这能为你提供一些参考" | "可参考" |
| 完美主义 | "通过深入分析，我们可以发现" | "分析显示" |

参考规范：`references/chinese-writing.md`（中文反 AI 模式）、`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 不提升词汇量——只是去掉 AI 味
2. 不修改事实、数据、代码片段
3. 中文文本遵守 `references/chinese-writing.md` 的 8 条反模式
4. 技术术语保持原文，不要翻译

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 把技术术语翻译了 | 比原文更难读 | API/hook/workflow 保持原文 |
| 润色过繁 | 失去了原始语气 | 只改 AI 味，保留作者声音 |
| 改了代码/数据 | 润色不是编写 | 代码和数据标注"不修改" |
| 用更华丽的词替代 | 更装了 | 简化为好 |
