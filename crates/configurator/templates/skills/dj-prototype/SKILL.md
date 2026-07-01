---
name: dj-prototype
description: >
  造废品验证设计：用可运行的代码回答"这个方案行不行"的问题。
  Use when the user wants to validate a design idea before committing to it,
  or when a question can't be answered by staring at code.
  触发词：原型、prototype、先试试、验证一下、做个 demo、做个 POC。
---

# Prototype: 造废品验证

## 职责

用可运行的废品代码回答设计问题。废品从第一天就标记为废品。

## 什么时候用

- "这个逻辑/状态模型感觉对吗？" → 造一个终端交互程序推演状态机
  - 示例：用户说"我想验证一下订单状态机的流转逻辑" → 造一个终端交互程序，让用户输入事件，程序输出状态变化
- "这个 UI 长什么样？" → 造几个截然不同的 UI 变体
  - 示例：用户说"我不确定仪表盘用卡片布局还是表格布局" → 造 2 个 HTML 页面，分别展示卡片和表格布局
- "这个 API 集成能跑通吗？" → 造一个最小调用脚本
  - 示例：用户说"我想验证一下 WebSocket 实时通知方案能不能跑通" → 造一个最小 WebSocket 客户端和服务端，验证连接和消息传递

## 规则

1. **从第一天就标记为废品** — 文件名/目录名表明是 prototype，不是生产代码
2. **一个命令运行** — 用项目已有的 task runner（`pnpm dev`、`python main.py` 等）
3. **默认不持久化** — 状态在内存里，不建数据库
4. **能回答问题就行** — 不追求完美、不加错误处理、不做边缘情况
5. **靠近使用位置** — 放在它原型化的模块/页面旁边

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | One design question, prototype type, runnable command, allowed files, and discard policy |
| 输出 | Disposable prototype, run result, answer to the design question, and decision for formal implementation |
| 非目标 | Do not ship prototype code, wire it into production paths, persist real data, or turn it into architecture by accident |

## 工作流

1. Confirm the one question the prototype must answer.
2. Choose prototype type: logic simulation, UI variant, or integration check.
3. Define scope limit: smallest files/data/interaction needed to answer the question.
4. Write the minimum runnable code and label it as prototype in path or filename.
5. Run it and capture the result.
6. Decide: discard, keep as reference, or mark formal implementation as follow-up.

### 🔴 CHECKPOINT · 原型结论确认

原型运行后：
```
原型结果：<回答了什么问题>
结论：<方案可行 / 方案不可行 / 需要进一步验证>
原型代码：<删除 / 保留为参考 / 标记正式实现后续>
生产路径影响：none
确认结论？(Y/n)
```

- 方案可行 → 不直接复用原型代码；标记正式实现后续。
- 方案不可行 → 记录原因，换方案。
- 需要进一步验证 → 缩小问题范围，再做一轮原型。

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 原型跑不起来 | 检查依赖和环境，用最简方式启动 | 降级为伪代码/草图，明确 `not runnable` |
| 原型太像生产代码，用户不想删 | 重命名标记为 prototype，隔离到临时或参考路径 | 明确告知不能直接发布，正式实现需另走任务 |
| scope 变大做不出来 | 缩小问题范围，只验证核心假设 | 用文字决策表代替代码原型 |
| 不确定要回答什么问题 | 问用户"你最不确定的是什么" | 默认验证风险最高、最难从静态阅读判断的假设 |
| 原型需要真实数据或凭证 | 改用 mock/sample 数据 | 停止集成验证，标注 blocker |

## 🔴 CHECKPOINT · 原型确认

开始前确认：
```
原型目标：<要回答什么问题>
类型：<逻辑推演 / UI 变体 / 集成验证>
scope limit：<只验证哪些输入、状态或交互>
runnable command：<命令或 not runnable>
discard policy：<删除 / 保留为参考 / 标记正式实现后续>

开始？(Y/n)
```

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 原型里加完整错误处理 | 只处理能回答问题的 happy path |
| 原型变成生产代码 | 标记为废品；正式实现需另走任务 |
| 原型接入真实生产路径 | 用隔离入口、mock 数据或临时路径 |
| 不告诉用户这是原型 | 从一开始就说明是废品 |
| 原型放在奇怪的位置 | 放在相关代码旁边或明确的 prototype 路径 |
| 用真实凭证/真实用户数据验证 | 用 sample/mock 数据；需要凭证时停止并标注 blocker |
| 原型回答多个问题 | 一轮原型只回答一个设计问题 |
