---
name: dj-health
description: 兼容入口：将仓库健康检查委派给 dj-audit 的 health profile。
summary: 兼容入口：dj-audit health profile
risk: low
---

# Health Compatibility Alias

此名称为兼容现有调用保留。它不参与 dispatch、route gate、bucket 或 runtime skill 注入。

执行 `dj-audit`，并选择 `health` profile：检查构建、测试、Git、依赖、格式、agent 配置和 CI 状态；只报告，不修复。
