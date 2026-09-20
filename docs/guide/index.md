---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: 'fdcec1cf-004d-4d08-9754-55b28f1fddf1'
  PropagateID: 'fdcec1cf-004d-4d08-9754-55b28f1fddf1'
  ReservedCode1: '250ba61a-8fa2-4775-b6e7-b0ff30ae5bdc'
  ReservedCode2: '250ba61a-8fa2-4775-b6e7-b0ff30ae5bdc'
---

# 用户指南

DiJiang 是纯 skill 架构：所有工作流通过调用 skill 完成，无运行时依赖。调用方式统一为 `Call the Skill tool with "dj-xxx"`。

## Skill 选择流程

### 按任务类型选择

```
你的任务是什么？
├── 不知道走哪条流程 / 想要全局路线
│   └── dj-ask（全局 flow 路由图）
├── 新功能 / 明确需求
│   ├── 有测试要求 → dj-tdd
│   └── 无测试要求 → dj-implement
├── Bug / 回归
│   ├── 排查根因 → dj-hunt
│   ├── 改码回归保护 → dj-regression-guard
│   └── 全量回归测试 → dj-fullstack-testing
├── 需求不明确 / 范围模糊
│   └── dj-grill → 对齐后再走实现路径
├── 审查现有代码
│   ├── 需要运行测试、检查完整性 → dj-check
│   ├── 只需要快速看一眼、不改代码 → dj-review
│   └── 全仓审计 / 过度工程扫描 / 技术债 → dj-audit
├── 设计 / UI
│   └── dj-design
├── 吸收融合 / 复刻
│   ├── 吸收外部项目模式 → dj-absorb
│   └── 1:1 复刻界面功能 → dj-remix
├── 原型验证
│   └── dj-prototype
├── 脚本 / 工具编写
│   └── dj-script
├── 文档
│   ├── 写 PRD / 设计文档 / API 文档 → dj-output
│   └── 文本润色、去 AI 味 → dj-write
├── 模式研究
│   └── dj-pattern
├── 复杂判断 / 系统透镜
│   └── dj-reason
├── 最小改动 / YAGNI 约束
│   └── dj-ponytail
└── Session 交接
    └── dj-handoff
```

### 按任务状态选择

| 当前任务状态 | 允许的 skill | 说明 |
|-------------|-------------|------|
| `none` | `dj-dispatch` | 先分流，不要直接干活 |
| `planning` | `dj-grill`、`dj-output` | 对齐阶段；实现类请求按 skill 纪律引导到 `dj-grill` |
| `in_progress` | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-regression-guard` / `dj-fullstack-testing` / `dj-script` / `dj-check` 等 | 实现阶段，按需选择 |
| `completed` | 无（走 `dijiang-finish-work`） | 收尾，不使用 skill |
| `archived` | 无 | 只读，需 `dijiang-start` 重新激活 |
| `paused` | `dijiang-continue` | 恢复后回到 planning 或 in_progress |

`dj-review` 是只读的双维度审查：spec 匹配度与代码质量必须分别完成后再汇总。平台有并行执行能力时可并行，没有时按相同输入串行执行，不能因工具能力不同而省略任一维度。

## 常见工作流

### 场景 A：开始一个新功能

```
1. Call the Skill tool with "dijiang-start" → 加载任务上下文，明确路线
2. 需求已明确？是 → Call "dj-implement"（有测试要求 → "dj-tdd"）；否 → Call "dj-grill" 对齐 → Call "dj-output" 产出 PRD → 再实现
3. 改码过程走 "dj-regression-guard" 三明治协议（改前基线 → 修改 → 专项验证 → 改后回归）
4. Call "dj-check" 双轴审查（Standards + Spec）
5. Call "dijiang-finish-work" → 验证 → 版本决策 → 提交 → 归档
```

### 场景 B：修复一个 Bug

```
1. Call "dj-hunt" 排查根因（先定位，再修复）
2. 改码护送走 "dj-regression-guard" 三明治协议
3. 需要全量回归时 Call "dj-fullstack-testing"（API 深度验证、契约校验、UI 交互、覆盖度核对）
4. Call "dijiang-finish-work" 收尾（提交、归档）
```

### 场景 C：需求不明确

```
1. Call "dj-grill" 追问 2-3 轮，明确具体范围 → 输出对齐结论
2. Call "dj-output" 根据对齐结论创建 PRD
3. 确认 PRD 后 → Call "dj-implement" 实现
```

### 场景 D：跨 Session 交接

```
1. 当前 session 结束前：Call "dj-handoff" → 输出 session 摘要、未决问题、下一步建议
2. 新 session 开始：Call "dijiang-continue" → 恢复活跃任务、加载产物、报告下一步
```

### 场景 E：并行任务

```
1. Call "dj-channel" 生成多个子 agent
2. 每个子 agent 独立执行一个任务
3. 收集结果，汇总到主流程
```

## Worktree 工作流

DiJiang 使用 git worktree / 分支隔离代码修改，主 checkout 始终保持干净。该纪律由 `dj-git-guardrails` 以文本形式约束（ADR-005 后无运行时强制）：

```
主仓库（main checkout）  ← 始终保持干净
  │
  ├── 工作纪律（dj-git-guardrails）
  │   ├── 不在 main 上直接改 → 永远使用 worktree 或分支
  │   ├── 不 push 到 main → 使用 PR/MR 流程
  │   └── push 前检查 → 无 debug 代码、凭证、TODO
  │
  └── dijiang-finish-work 只从任务 worktree 收尾
      → 确认 Git 隔离 → 提交 → 归档
```

### 常见误区和处理方法

| 问题 | 处理方式 |
|------|----------|
| 不确定在哪个 worktree | `git worktree list` 列出所有 worktree，确认当前分支与顶层目录 |
| 在主 checkout 直接改代码 | 切到任务 worktree 或分支再改（dj-git-guardrails 铁律） |
| finish-work 被阻断 | 通常是验证证据缺失（缺 TDD evidence / 测试未跑 / 文档同步未说明），补齐后重试 |
| 收尾时 diff 混入无关文件 | 只暂存已审查路径，不盲目 `git add .` |