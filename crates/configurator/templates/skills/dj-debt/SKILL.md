---
name: dj-debt
description: >
  追踪被推迟的技术捷径：收集代码中所有 ponytail: 标记，生成债务台账。
  Use when the user wants to see what shortcuts were taken and what needs to be revisited.
  触发词：debt、债务、捷径、dj-ponytail debt、之前标记的、推迟的、shortcut。
---

# Debt: 捷径追踪

## 职责

收集代码中所有 `ponytail:` 注释标记，生成一份债务台账。让"以后再说"不会悄悄变成"永远不做"。

## 核心原则：排熵通道

**山源于「不敢删」，不是代码质量差。** 只加不减 = 自焊排熵管道。

排熵通道：删除、退役、归档、迁移、收敛概念。

**治理路径必须是最便宜的路径**，否则你建的每一道墙，都在为翻墙者训练肌肉。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Scan scope, source file types, and whether the user asked for export |
| 输出 | Read-only debt ledger grouped by file, with trigger quality, risk, and recommended follow-up |
| 非目标 | Do not fix debt, create docs by default, or label ordinary style preferences as debt |

## 工作流

### 1. 扫描

```bash
grep -rnE '(#|//) ?ponytail:' . \
  --include='*.py' --include='*.ts' --include='*.js' \
  --include='*.go' --include='*.rs' --include='*.swift' \
  --exclude-dir=node_modules --exclude-dir=.git \
  --exclude-dir=dist --exclude-dir=build
```

### 2. 整理

每个命中是一行台账。注释格式约定：`ponytail: <简化内容>, <触发条件>`

分诊字段：

```text
Location: <file>:<line>
Shortcut: <what was simplified>
Ceiling: <when it stops being acceptable>
Trigger: <specific revisit condition or no-trigger>
Risk: <low/medium/high>
Follow-up: <leave / revisit / implementation needed>
```

风险规则：

- `high`: no trigger, touches security/data loss/release flow, or blocks future work.
- `medium`: trigger exists but is vague, or affected code is central.
- `low`: trigger is precise and local impact is limited.

### 3. 报告

```text
## 债务台账

<按文件分组的台账>

Summary:
- Total: <N> markers
- No trigger: <M>
- High risk: <H>
- Exported: no
```

无触发条件的标记是风险最高的，因为没人知道什么时候该回来修。

## 🔴 CHECKPOINT · 台账确认

扫描完成后：

```text
债务台账：
- 总计：<N> 个标记
- 无触发条件：<M> 个（最高风险）
- 高风险：<H> 个
- 按文件分组：<文件数>

默认动作：只报告
导出文件：<仅在用户明确要求时填写>
```

- 默认只在对话中展示台账。
- 只有用户明确要求导出时，才写入用户指定文件。
- 用户要求修复 → 拒绝在 `dj-debt` 中修复，只提供 follow-up type。

## 标记约定

在代码中使用 ponytail 简化时，加上注释：
```python
# ponytail: 硬编码配置, 超过3个配置项时提取
timeout = 30
```

```typescript
// ponytail: 内联实现, 需要第二种实现时抽象
function processOrder(order: Order) { ... }
```

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| grep 没找到任何标记 | 确认扫描路径和文件类型正确 | `No ponytail: debt. Clean ledger.` |
| 标记格式不统一 | 尽量解析，无法解析的标注"格式异常" | 只统计数量，不解析内容 |
| 标记太多无法排序 | 按文件分组，每组取 top 5 | 只输出无触发条件的（最高风险） |
| 用户要求自动修复 | 拒绝，只生成报告 | 提供修复建议，但不执行 |

## 边界

- 只读报告，不修改代码
- 如果没有找到标记：`No ponytail: debt. Clean ledger.`
- 不主动创建 `DEBT.md` 或任何文档文件
- 用户明确要求导出时，使用用户指定路径；未指定路径时先确认路径
- 技术债必须来自明确 shortcut 标记或可证据化的延期决策，不把审美偏好算作债务

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 找到标记后自动修复 | 只报告，用户决定 |
| 默认写入 `DEBT.md` | 默认只报告，明确要求导出才写文件 |
| 忽略无触发条件的标记 | 标记 no-trigger，重点提示 |
| 不扫描就声称没有债务 | 实际跑 grep 确认 |
| 把风格偏好当技术债 | 只记录 shortcut 或延期决策 |
| ponytail 简化时不加注释 | 每个简化都加 ponytail: 标记 |
