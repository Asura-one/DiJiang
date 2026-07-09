---
name: dj-health
description: >
  Agent 配置健康检查：检查 agent 指令、工具链、验证器是否正常工作。
  Use when the user suspects agent config is broken, instructions are being ignored, skills aren't loading, or tools aren't working.
  触发词：健康检查、配置检查、工作不正常、指令不生效、技能加载失败。
---

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
