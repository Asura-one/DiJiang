---
name: dj-health
description: >
  Agent 配置健康检查：检查 agent 指令、工具链、验证器是否正常工作。
  Use when the user suspects agent config is broken, instructions are being ignored,
  or the AI coding setup is drifting.
  触发词：health、健康检查、配置检查、agent不听话、指令没生效、配置对不对。
---

# Health: Agent 健康检查

## 职责

检查 agent 配置和 AI 编码环境是否健康。发现配置漂移、指令失效、验证缺失。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | 项目根目录、当前任务上下文、agent 配置文件、可用验证命令、疑似失效模式 |
| 输出 | 只读健康报告，包含证据、命令结果、严重程度和建议下一步 |
| 非目标 | 不修改配置、不安装依赖、不在健康检查中修应用 bug |

## 工作流

1. **确认范围** → agent 指令、工具、验证、skills 或 token 预算。
2. **收集证据** → 检查文件并运行安全的只读命令。
3. **分类发现** → broken、risky、healthy 或 not checked。
4. **区分原因** → agent 配置问题、项目代码问题、环境问题。
5. **只报告** → 包含证据和明确下一步，不在该 skill 中修复。

## 最小检查项

```bash
git status --short --branch
find . -maxdepth 3 -name 'AGENTS.md' -o -name 'CLAUDE.md' -o -name '.cursorrules'
find . -maxdepth 4 -path '*/SKILL.md' | sort
dijiang status --compat
dijiang workflow-state --json
```

只运行已有文档记录且安全的项目验证命令。命令缺失时记录 `not checked`，不要编造命令。

## 报告格式

```text
## Agent 健康报告

范围：<agent instructions / tools / verification / skills / budget>

### 🔴 Broken
- <file or command>: <evidence>. 影响：<impact>. 后续类型：<implementation / hunt / docs / none>.

### 🟡 Risky
- <file or command>: <evidence>. 影响：<impact>. 后续类型：<implementation / hunt / docs / none>.

### 🟢 Healthy
- <check>: OK (<evidence>)

### ⚪ Not Checked
- <check>: not checked (<reason>)

结论：<healthy / degraded / broken>
后续类型：<implementation / hunt / docs / none>
```

## 检查项

### 1. 指令层

- `AGENTS.md`、`CLAUDE.md`、`.cursorrules` 是否存在？
- 内容是否引用不存在的文件、命令、runtime 或路径？
- 多个指令文件是否冲突？
- 指令是否要求无法执行的工具或流程？

### 2. 工具层

- 项目依赖是否安装完整？
- documented test/lint/typecheck 命令是否能正常运行？
- Git hooks 是否正常？
- 外部工具或 agent 服务不可用时是否有兜底路径？

### 3. 验证层

- CI 与本地验证命令是否一致？
- 有没有应该跑但没跑的验证？
- 验证命令是否指向正确路径/配置？
- 失败输出是否足够定位问题？

### 4. Skill 层

- 已安装的 skill 是否有冲突？
- skill 引用的路径/命令是否可达？
- skill 是否有过时 runtime、绝对路径或不可用工具引用？
- skill 是否有明确输入/输出、失败处理和边界？

### 5. Token 预算（budget-aware）

- 上下文窗口使用率
- 配额使用率是否影响当前任务
- 哪些 skill 或步骤消耗最大
- 是否需要 handoff、summary 或减少扫描范围

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 配置文件缺失 | 检查默认位置和环境变量 | 标注"配置缺失"，给出创建命令 |
| 验证命令跑不起来 | 检查依赖安装和 PATH | 标注"验证未执行"，手动检查关键文件 |
| 指令冲突无法判断 | 列出所有冲突项让用户选择 | 以最新修改的指令为准 |
| 检查项太多导致报告太长 | 只输出 🔴 和 🟡 级别 | 按层级分批输出 |

## 🔴 CHECKPOINT · 修复确认

报告发现后：

```text
发现 <N> 个问题：
- [Broken] <problem> -> <recommended follow-up type>
- [Risky] <problem> -> <recommended follow-up type>

健康检查是只读流程。修复路线只作为后续类型输出，不在 `dj-health` 中自行切换或修复。
```

🛑 STOP before making any fix. Start a separate task for remediation. Do not repair or switch workflow inside `dj-health`.

## 边界

- 只检查 agent 配置，不检查应用代码质量
- 一次性报告
- 不自动修复（除非用户明确要求）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 自动修复发现的问题 | 报告后等用户决定，修复走单独任务 |
| 只检查配置文件不实际运行命令 | 对安全的只读命令做实际验证 |
| 把应用 bug 当 agent 配置问题 | 区分 agent 配置、项目代码、环境问题 |
| 不检查就声称健康 | 逐项检查后报告证据 |
| 命令不存在时编造结果 | 标注 `not checked` 和原因 |
| 输出一堆健康项淹没严重问题 | Broken/Risky 优先，Healthy 简短列证据 |
| 在 `dj-health` 中直接修配置 | Do not repair inside health check；只输出后续类型 |
| 把 `not checked` 写成 OK | 保留 `not checked`，说明缺少什么证据 |
