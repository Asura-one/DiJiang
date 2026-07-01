---
name: dj-handoff
description: >
  Session 交接：将当前对话压缩为交接文档，让下一个 agent 无缝接手。
  Use when the user needs to switch sessions, hand off work to another agent,
  or preserve context before a break.
  触发词：handoff、交接、换个session、接手、总结一下当前状态。
---

# Handoff: Session 交接

## 职责

将当前对话的上下文压缩为一份结构化文档，让新的 session 或 agent 可以无缝接手。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Current task goal, conversation state, git state, changed files, verification output, and known blockers |
| 输出 | One structured handoff document plus its save path |
| 非目标 | Do not solve remaining work, rewrite PRD/design docs, or create a commit |

## 工作流

### 1. 收集上下文

Run these commands when available:

```bash
dijiang task current
dijiang status
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
```

Collect exactly these fields:

| Field | Required content |
|---|---|
| Task | 一句话说清楚在做什么 |
| Goal | 用户真正要的结果 |
| Completed | 已完成步骤、验证结果、保留的改动 |
| Pending | 未完成项、下一步动作、阻塞原因 |
| Decisions | 关键取舍和理由 |
| Files | 相关文件路径，每行一个 |
| Verification | 已运行命令及结果；未运行则写 `not run` 和原因 |
| Risks | 已知风险、失败点、需要人审的地方 |
| Next skill | 建议下一个 agent 使用的 `dj-*` skill |

If a command fails, record the command and error summary in `Risks`; do not fabricate missing state.

### 2. 脱敏检查

保存前扫描草稿中的 secret 和个人数据：

```text
API keys: sk-, ghp_, xoxb-, AKIA
Generic: password=, token=, secret=, private_key
Files: .env, id_rsa, credentials.json
```

Replace the value with `[REDACTED]` and keep the key name when the key name is useful context.

### 3. 生成交接文档

Use this exact template:

```markdown
## 交接文档

### 任务
<一句话说清楚在做什么>

### 目标
<用户真正要的结果>

### 已完成
- <已完成的步骤、结果、验证>

### 未完成
- <待办事项或阻塞>

### 关键决策
- <决策：理由>

### 当前状态
- Branch: <branch>
- Dirty files: <count and paths>
- Verification: <command => result>

### 相关文件
- <path>

### 风险
- <risk or none>

### 建议 skill
- <下一个 agent 应该用哪个 skill 接手>

### 下一步
- <一条最小可执行动作>
```

### 4. 保存并验证

保存到用户 OS 的 temp 目录，不写入工作区。文件名使用：

```text
dijiang-handoff-<task-name>-<YYYYMMDD-HHMM>.md
```

保存后确认文件存在并输出路径：

```bash
test -f <handoff-path> && wc -l <handoff-path>
```

保存到临时目录（用户 OS 的 temp 目录），不是工作区。

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 交接文档太长 | 压缩到 1 页以内，只保留关键信息 | 只留"任务+未完成+下一步"三项 |
| 脱敏遗漏了凭证 | 扫描文档中的 token/password/key 模式 | 用 `[REDACTED]` 替换所有可疑字符串 |
| 临时目录不可写 | 尝试用户桌面或 Downloads | 直接输出到对话中，让用户复制保存 |
| 上下文太复杂无法压缩 | 用结构化列表代替叙述 | 只保留文件路径和关键决策，省略过程 |

## 🔴 CHECKPOINT · 交接确认

生成交接文档后：
```
交接文档已生成：
- 任务：<一句话>
- 未完成项：<N 个>
- 建议 skill：<下一个 skill>
- 保存位置：<路径>

确认保存？(Y/n)
```

## 规则

- 不复制其他已有产物（PRD、设计文档、commit message）的内容，引用路径即可
- 脱敏（删除 API key、密码、个人信息）
- 如果用户说明下一个 session 要做什么，针对那个方向调整交接文档
- 保持简洁——新 agent 需要能快速理解，不是读一篇论文

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 把整个对话历史贴进去 | 只提炼关键信息 |
| 不脱敏直接保存 | 删除凭证和个人信息 |
| 保存到工作区 | 保存到临时目录 |
| 不标注下一步建议 | 标注建议用什么 skill 接手 |
| 交接文档写成小说 | 简洁结构化，新 agent 30 秒能理解 |
