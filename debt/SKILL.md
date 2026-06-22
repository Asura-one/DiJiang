---
name: debt
description: >
  追踪被推迟的技术捷径：收集代码中所有 ponytail: 标记，生成债务台账。
  Use when the user wants to see what shortcuts were taken and what needs to be revisited.
  触发词：debt、债务、捷径、dj-ponytail debt、之前标记的、推迟的、shortcut。
---

# Debt: 捷径追踪

## 职责

收集代码中所有 `ponytail:` 注释标记，生成一份债务台账。让"以后再说"不会悄悄变成"永远不做"。

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

每个命中是一行台账。注释格式约定：`ponytail: <上限>, <触发条件>`

输出格式：
```
<file>:<line>, <简化了什么>. 上限: <天花板>. 触发: <何时重新审视>.
```

没有标记触发条件的 → 加 `no-trigger` 标签（这些最容易悄悄腐烂）

### 3. 报告

```
## 债务台账

<按文件分组的台账>

总计：<N> 个标记，<M> 个无触发条件
```

无触发条件的标记是风险最高的——没人知道什么时候该回来修。

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
- 用户可以要求将台账写入文件（如 `DEBT.md`）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 找到标记后自动修复 | 只报告，用户决定 |
| 忽略无触发条件的标记 | 标记 no-trigger，重点提示 |
| 不扫描就声称没有债务 | 实际跑 grep 确认 |
| ponytail 简化时不加注释 | 每个简化都加 ponytail: 标记 |
