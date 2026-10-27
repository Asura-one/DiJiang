# workflow 状态机（workflow.md 自描述运行时状态机）

- **否决日期**：2026-10-27
- **提案来源**：references/Trellis（融合分析报告 P0-1）
- **否决原因**：该提案的核心价值是"消除 Rust CLI 代码与文档间的 drift"，但 ADR-005（2026-09-17）已放弃 CLI 路线、移除全部 Rust 代码，DiJiang 无运行时状态机，drift 一说不复存在。
- **重新打开条件**：DiJiang 重新引入任何运行时（CLI/守护进程），且该运行时需要状态机驱动。
