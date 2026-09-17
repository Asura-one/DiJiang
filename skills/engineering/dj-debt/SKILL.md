---
name: dj-debt
description: 兼容入口：将技术债审计委派给 dj-audit 的 debt profile。
disable-model-invocation: true
---

# Debt Compatibility Alias

此名称为兼容现有调用保留。它不参与 dispatch、route gate、bucket 或 runtime skill 注入。

执行 `dj-audit`，并选择 `debt` profile：扫描 TODO/FIXME/HACK、弃用代码、依赖、测试和构建债务；只报告，不修复。
