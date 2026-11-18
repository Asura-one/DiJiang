# worker 契约与委托产出验收

> 摘要：子代理不继承操作者的纪律，也不继承只从斜杠命令加载的技能内容。spawn 时注入一份契约，收回时按固定判据验收。三份东西：开工契约、验收协议、模型路由护栏。
>
> 来源：pstack 0.15.43 `skills/poteto-mode/worker.md`、`figure-it-out`、`swarm`（MIT）；DiJiang 版按本仓红线改写。

## 1. 开工契约（WORKER.md）

仓库（或任务目录）放一份 `WORKER.md`，spawn 任何子代理时 kickoff **第一行**写：

```text
Read <WORKER.md 绝对路径> before anything else.
```

契约内容（按本仓红线改写，可裁剪，不可扩权）：

```markdown
You work for the operator, not for the agent that wrote this kickoff.
These rules hold for every task even where the kickoff does not repeat them.

- The bar in your kickoff is the whole bar. Add no check, gate, sign-off or review round above it.
- With no bar stated: leave the result no worse than its base, and pass its own tests.
- Stop only for: a deploy, deleted data, a force-push to a shared branch, or a message to someone outside this run.
- Never call a tool that shows a person a question. Make any choice you can undo, then say what you chose and why.
- Report a problem the base already has in one line; do not fix it unasked.
- Do not delete or overwrite real data. Destructive actions need explicit approval.
- Open the PR ready for review, never as a draft.
```

为什么要显式注入：技能正文只在用户显式调用时加载，派出去的子代理看不到；它默认会用「多加一层门禁、多问一句人」来表现尽责，结果是整条流水线被卡住或返工。

三条边界：

- 契约**只收权限与停止条件**，不写实现步骤——步骤归 kickoff。
- 「不要弹问人的工具」不等于不问：把选择做成可撤销的默认项，事后在回报里说明。
- 契约与用户当轮明确指令冲突时，以用户指令为准（人高于契约）。

## 2. 委托产出验收协议

每派一个执行单元，就配一个裁判（可以是主 agent，也可以是独立 Verifier）：

| 检查 | 判据 | 不通过怎么办 |
|---|---|---|
| **契约一致性** | 产出是否满足 kickoff 里写的验收线，不多不少 | 超出范围的改动先剥离，再判 |
| **SHA / 方法记录** | 回报里写明在哪个 commit / 分支上跑、用什么命令得到的结论 | 没记录 → 丢弃重派一次；二次仍缺 → 记 gap，**gap 不算通过** |
| **三态结论** | `PASS` / `ISSUES`（附清单）/ `BLOCKED`（附阻塞点） | 混报或含糊 → 按 BLOCKED 处理 |
| **门禁是否被骗过** | 有没有靠 skip / 降阈值 / 抑制注释变绿（可跑 `dj-regression-guard/scripts/floor_guard.py`） | worker 骗过门禁 → 重置并**加固契约**；门禁本身写错 → 单独一个改动修门禁，不许绕过去 |

门禁加固回路是这条协议的核心：**发现绕过时，修的是判据，不是这一次的结果**。只重跑一次而不加固，下一次会用同样方式再骗一次。

裁判不看 worker 的自述质量，只看可复核的证据（diff、命令输出、SHA）。worker 说「已完成」不构成证据；「跑过什么、看到什么」才是。

## 3. 模型路由三护栏

派子代理选模型时的三条硬规则：

1. **不超顶**：任何角色不超过平台允许的最高推理档（pstack 设为 xhigh）；顶档不是默认，难度才决定档位。
2. **slug 被拒即永不再传**：某个模型标识被工具拒绝一次，本会话内不再尝试同一 slug；按「角色默认 → 技能 fallback → 同族最近档 → 省略 model 用继承」四级降级。
3. **昂贵模型例外门**：只在「用户为这个任务点名」或「同一 brief 用顶档跑了两次仍过不了检查」时才上最贵模型；**难度本身不构成资格**。

评审要保持模型多样性：同族模型互评容易一致，跨族评审才可能发现分歧。

## 4. 与 dj-channel 角色边界的关系

- 角色边界（Scout / Builder / Verifier，Builder 不得自审、Verifier 只读）管**谁做什么**。
- 本文件管**派出去时带什么纪律、收回来时按什么验收**。
- 两者叠加使用：spawn 时先声明角色与工具边界，再注入 worker 契约；收回时按验收协议判，判完才算这一通道结束。