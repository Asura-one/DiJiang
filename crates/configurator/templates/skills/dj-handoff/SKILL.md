---
name: dj-handoff
description: >
  Session 交接：将当前对话压缩为结构化交接文档，让下一个 agent 无缝接手。
  Use when the user needs to switch sessions, hand off work to another agent,
  or preserve context before a break.
  触发词：handoff、交接、换个session、接手、总结一下当前状态。
---

# Handoff: Session 交接

## 工作流

### 1. 收集上下文

```bash
dijiang task current
dijiang status
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
```

收集：Task 目标、已完成项、未完成项、关键决策、相关文件、验证结果。

### 2. 脱敏检查

扫描这些模式并替换值为 `[REDACTED]`：
- API keys: `sk-`, `ghp_`, `xoxb-`
- 凭据: `password=`, `token=`, `secret=`, `private_key`
- 文件: `.env`, `id_rsa`, `credentials.json`

### 3. 生成交接文档

```markdown
## 交接文档

### 任务
<一句话>

### 目标
<用户要的结果>

### 已完成
- <已完成项>

### 未完成
- <待办项或阻塞>

### 关键决策
- <决策：理由>

### 当前状态
- Branch: <branch>
- Dirty files: <count>
- Verification: <command => result>

### 相关文件
- <path>

### 风险
- <risk or none>

### 建议 skill
- <下一个 skill>

### 下一步
- <一条最小可执行动作>
```

### 4. 保存

保存到系统 temp 目录：`dijiang-handoff-<task-name>-<YYYYMMDD-HHMM>.md`

## 规则

- 不复制其他产物（PRD、design doc）的内容，引用路径即可
- 脱敏（删除 API key、密码、个人信息）
- 简洁——新 agent 30 秒能理解

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 贴整个对话历史 | 只提炼关键信息 |
| 不脱敏直接保存 | 删除凭证和个人信息 |
| 保存到工作区 | 保存到临时目录 |
| 交接文档写成小说 | 新 agent 30 秒能理解 |
