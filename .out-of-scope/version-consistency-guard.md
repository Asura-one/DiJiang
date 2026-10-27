# 多 manifest 版本一致性守卫（跨 7 个插件清单校验同一版本号）

- **否决日期**：2026-10-27
- **提案来源**：references/ponytail（scripts/check-versions.js）
- **否决原因**：该守卫防御"多个平台 manifest 一起陈旧"的失败模式，前提是存在多个声明版本号的 manifest。DiJiang 无版本号体系、单 manifest 分发，无失败模式可防（YAGNI）。
- **重新打开条件**：DiJiang 引入版本号并在 2 处以上声明。
