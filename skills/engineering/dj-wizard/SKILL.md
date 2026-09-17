---
name: dj-wizard
description: "生成一个交互式 bash 向导，引导人类走过只有他们能做的步骤。Use when provisioning infrastructure, setting up credentials or CI secrets, walking an unfamiliar third-party dashboard, or running a one-off migration or cutover. Don't invoke this for steps the agent can perform itself."
---

# Wizard 向导

一个 **wizard** 是一个 bash 脚本，逐步引导人类走过手动过程：打开每个 URL、说清楚点什么和复制什么、捕获值、写到它们该在的地方（`.env`、GitHub secrets）、每阶段确认、显示还剩多少阶段。

**你的工作只是划定过程并编排其阶段。** 脚本的交互模板是标准化的：阶段式进度、确认门、跨平台 URL 打开、隐藏密文输入、幂等 `.env` upsert、`gh secret`/`gh variable` 写入、结尾总结。库部分在所有 wizard 中一致；不要手改它。

一个 wizard 默认是**临时**的：为一次运行而构建，保存到临时或 `scripts/` 路径，任务完成后删除。只有用户想要可重复的设置路径时才提交到仓库。

## 创建 Wizard

1. **划定过程**：列出人类必须执行的步骤。只有"只有人类能做的步骤"进 wizard —— 能由 agent 完成的步骤不要放进 wizard。

2. **写脚本**：按 `scripts/wizard-template.sh` 的模板写各阶段。每个阶段包含：
   - 打开 URL（如有）
   - 精确说明点击什么、复制什么
   - 捕获值（隐藏输入）
   - 写入目标位置（`.env`、secrets）
   - 确认门（是否继续）

3. **测试**：与用户一起跑一遍。

4. **清理**：一次性任务完成后删除临时脚本；可重复流程按用户要求提交。

## 规则

| 规则 | 说明 |
|------|------|
| 只放人类步骤 | agent 能自己做的不要放进去 |
| 每阶段确认 | 逐步推进，不批量 |
| 幂等 upsert | 重复运行不产生重复写入 |
| 临时默认 | 用完即删，除非用户要求保留 |

## 反模式

| 不要 | 改为这样做 |
|------|-----------|
| 用 wizard 做 agent 自己能的步骤 | 直接做 |
| 把 secrets 打到输出 | 隐藏密文录入 |
| 一次做完所有阶段 | 每阶段确认门 |

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'af24e606-0072-4b07-8dd0-a9f1cb147244'
  PropagateID: 'af24e606-0072-4b07-8dd0-a9f1cb147244'
  ReservedCode1: 'b54271dd-1a6d-43b7-b3ad-4ea22db59b86'
  ReservedCode2: 'b54271dd-1a6d-43b7-b3ad-4ea22db59b86'
