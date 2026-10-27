# 运行时生命周期 hooks（after_create/after_start/after_finish/after_archive）

- **否决日期**：2026-10-27
- **提案来源**：references/Trellis（融合分析报告 P2-9）
- **否决原因**：hooks 依赖运行时生命周期事件（任务创建/启动/完成/归档时执行 shell 命令）。纯 skill 架构无生命周期事件可订阅；任务收尾纪律已由 dijiang-finish-work 文本承载。
- **重新打开条件**：出现真实的 CI 触发/通知自动化需求且项目接受引入运行时。
