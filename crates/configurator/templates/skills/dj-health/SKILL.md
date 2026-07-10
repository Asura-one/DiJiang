---
name: dj-health
description: >
  Agent 配置健康检查：检查 agent 指令、工具链、验证器是否正常工作。
  Use when the user suspects agent config is broken, instructions are being ignored, skills aren't loading, or tools aren't working.
  触发词：健康检查、配置检查、工作不正常、指令不生效、技能加载失败。
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | Agent 配置健康检查报告 |
| **Done when** | 所有检查项（配置目录、skill 列表、CLI 工具、测试命令）完成 |
| **Evidence** | 各检查项的执行结果 |
| **Output** | 健康检查报告（各维度 Pass/Fail + 修复建议） |

# Health: Agent 配置健康检查

检查 agent 指令、工具链、验证器是否正常工作。只报告，不修复。

## 工作流

### 1. 最小检查项

- [ ] `.hermes/` 配置目录是否存在且可读？
- [ ] 当前 skill 列表是否完整（`skill_view` 返回正常）？
- [ ] CLI 工具是否可用（`dijiang status`、`git status`）？
- [ ] 测试命令是否可用？
- [ ] 构建命令是否可用？

### 2. 深度检查

- 检查技能文件是否损坏（YAML frontmatter 是否能解析）
- 检查关键路径（skills/ 目录是否存在所有 `dj-*` 技能）
- 检查环境变量是否设置正确

### 3. 报告

```markdown
## Agent 健康报告
- 配置：✅/❌ <详情>
- 技能：✅/❌ <N/N 正常>
- 工具链：✅/❌ <详情>
- 测试：✅/❌ <详情>
- 构建：✅/❌ <详情>
```

## 边界

- 不修改配置
- 不安装缺少的工具
- 只做检查并报告

## Hard Rules

1. 不修改配置——只检查并报告
2. 不安装缺少的工具——只标记缺失
3. 检查范围明确：配置目录、skill 清单、CLI 工具、测试命令
4. 对每个检查项输出 Pass/Fail + 修复建议

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 顺手装了缺失的工具 | 只应检查不应修改 | 报告缺失就行 |
| 检查范围不完整 | 漏了关键问题 | 固定检查清单，全部覆盖 |
| 只报失败不给修复方法 | 用户不知道怎么做 | Pass/Fail + 修复建议 |
