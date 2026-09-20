---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '74afed85-0c13-40ec-8854-3de3ba08c6e8'
  PropagateID: '74afed85-0c13-40ec-8854-3de3ba08c6e8'
  ReservedCode1: '385c18c6-4487-4815-9bc7-18ce662c6c85'
  ReservedCode2: '385c18c6-4487-4815-9bc7-18ce662c6c85'
---

# DiJiang Project Instructions

本文件是 agent 的最小路由索引，不是第二份 workflow 定义。所有工作流逻辑分散在 skill 中。

## 项目结构

- `skills/` — 所有 DiJiang skill（engineering + productivity）
- `.dijiang/` — 项目本地状态（gitignored，由 dj-setup 初始化）
- `.dijiang/tasks/` — 任务存储
- `.dijiang/memory/` — 持久记忆（JSONL）
- `.dijiang/spec/` — 编码规范
- `CONTEXT.md` — 领域术语表
- `docs/adr/` — 架构决策记录
- `docs/references/` — 跨技能参考文档

## Skill 调用

使用 `Call the Skill tool with "<skill-name>"` 调用 skill。User-invoked skill 只能由用户显式触发；model-invoked skill 可由模型自动触发。

## Skill 路由

### 核心流程

| 场景 | Skill | 调用模式 |
|------|-------|---------|
| 新任务 / 不清楚的请求 | `dj-dispatch` | user |
| 全局流程路由 / 有哪些 workflow | `dj-ask` | user |
| 需求对齐 | `dj-grill` | user |
| PRD / 设计文档 | `dj-output` | user |
| Spec 初始生成 | `dj-spec-bootstrap` | user |
| PRD 拆分 | `dj-split` | user |
| 大块工作规划 | `dj-wayfinder` | user |
| Issue/任务分诊 | `dj-triage` | user |
| 功能实现 | `dj-implement` | model |
| 测试驱动开发 | `dj-tdd` | model |
| Bug / 回归排查 | `dj-hunt` | model |
| 改码回归保护 | `dj-regression-guard` | model |
| 全栈回归测试 | `dj-fullstack-testing` | model |
| 合并冲突解决 | `dj-merge-conflict` | model |
| 代码审查 / 质量门禁 | `dj-check` | model |
| 轻量只读审查 | `dj-review` | model |

### 辅助能力

| 场景 | Skill | 调用模式 |
|------|-------|---------|
| 全仓审计 / 技术债 / 健康检查 | `dj-audit` | user |
| 模式研究 | `dj-pattern` | user |
| 架构自省 | `dj-meta` | user |
| 推理透镜 | `dj-reason` | model |
| 技术调研 | `dj-research` | model |
| 吸收外部材料 | `dj-absorb` | user |
| 领域建模 | `dj-domain-modeling` | model |
| 代码结构设计 | `dj-codebase-design` | model |
| UI/UX 设计实现 | `dj-design` | user |
| 原型验证 | `dj-prototype` | user |
| 站点再造 | `dj-remix` | user |
| 脚本编写 | `dj-script` | user |
| 文字润色 | `dj-write` | model |
| URL/PDF 阅读 | `dj-read` | model |
| 极简纪律 | `dj-ponytail` | model |
| 人类步骤向导 | `dj-wizard` | model |
| Session 交接 | `dj-handoff` | user |
| 知识治理收尾 | `dj-gov` | user |
| Git 护栏 | `dj-git-guardrails` | model |
| 多 agent 通道 | `dj-channel` | user |
| Session 洞察 | `dj-session-insight` | user |

### Session 管理

| 场景 | Skill | 调用模式 |
|------|-------|---------|
| 项目初始化 / 迁移 | `dj-setup` | user |
| 启动会话 | `dijiang-start` | user |
| 继续会话 | `dijiang-continue` | user |
| 收尾工作 | `dijiang-finish-work` | user |
| 记忆管理 | `dj-memory` | model |

### Productivity

| 场景 | Skill | 调用模式 |
|------|-------|---------|
| 文字润色 | `dj-write` | model |
| 跨会话教学 | `dj-teach` | user |
| 决策问卷 | `dj-questionnaire` | user |
| 消息重述 | `dj-wait-what` | user |
| 为 agent 写作 | `dj-writing-for-agents` | model |

## 工作流状态机

```
none → planning → in_progress → completed → archived
                ↑                 ↓
              paused ←────────────┘
```

| 状态 | 允许的 skill |
|------|-------------|
| none | `dj-setup`（初始化）、`dj-dispatch`（创建任务） |
| planning | `dj-grill`（对齐）、`dj-output`（文档）、`dj-spec-bootstrap`、`dj-split` |
| in_progress | `dj-implement`、`dj-tdd`、`dj-hunt`、`dj-regression-guard`、`dj-fullstack-testing`、`dj-check`、`dj-review`、`dj-script`、`dj-design`、`dj-prototype` |
| completed | `dijiang-finish-work`（收尾） |
| paused | `dijiang-continue`（恢复） |
| archived | 只读，如需继续重新创建任务 |

## Git 安全工作流（Worktree-First）

1. 主工作区永远干净，只做同步，严禁在主目录上直接写代码。
2. 每个功能一个独立 worktree，所有开发、AI 调试均在 worktree 中进行。
3. 合并需用户确认，展示变更摘要后等待确认。
4. 回滚必须备份 + 确认。
5. 禁止自动执行破坏性操作：`reset --hard`、`force push`、`clean -f`。
6. 提交信息遵循 Conventional Commits，使用中文编写。

## 范围纪律

- 不多管闲事。不添加未要求的功能，不重构，不做"顺带改进"。
- 不过度设计。不为不可能发生的场景添加错误处理。
- 不提前抽象。不为一次性操作创建 helper 或抽象层。
- 不乱建文件。非绝对必要不创建新文件，始终优先修改现有文件。
- 不改没读过的代码。必须先阅读和理解，再提议修改。

## 参考文档

- `docs/references/` — 14 个跨技能参考（反模式、决策阶梯、代码任务合约等）
- `docs/adr/` — 架构决策记录
- `CONTEXT.md` — 领域术语表

<!-- AIGC 水印（系统注入，不可移除）-->
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '1e8a3911-740d-4ad7-9955-75031ac85343'
  PropagateID: '1e8a3911-740d-4ad7-9955-75031ac85343'
  ReservedCode1: 'f5d42fb0-755a-4286-9261-acb8e2925f22'
  ReservedCode2: 'f5d42fb0-755a-4286-9261-acb8e2925f22'