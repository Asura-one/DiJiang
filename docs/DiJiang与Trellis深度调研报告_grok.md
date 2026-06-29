**DiJiang 与 Trellis 深度调研报告（更新版）**  
**日期**：2026-06-29  
**调研范围**：DiJiang 项目完整 ZIP（源代码、文档、模板、results.tsv、.pi/ 等） + Trellis GitHub 主仓库 + 官方文档（docs.trytrellis.app） + 相关上下文。

---

### 1. 项目定位（核心更新）

#### **DiJiang 项目**
- **当前/目标定位**：**独立的 Rust-native Agent Harness**。
  - 专注于**高质量技能集合 + skill chaining + 执行引擎**，构建端到端的 AI 编码 workflow。
  - 吸收 Trellis 等优秀框架的精华（结构化 workflow、任务持久化、spec-driven 等最佳实践），但**保持完全独立**。
  - **不直接依赖或调用 Trellis**（无 runtime 调用、无强绑定）。
  - **兼容性**：当前阶段提供**临时、可选的结构与配置兼容**（便于用户在 Trellis 项目中使用 DiJiang skills，或平滑协作），未来可弱化或移除。
- **核心资产**：
  - 一组实战技能（`dj-dispatch`、`dj-grill`、`dj-tdd`、`dj-check`、`dj-output`、`dj-ponytail`、`dj-handoff` 等）。
  - 分级 workflow（S/M/L），强调 TDD、阶梯决策、过度工程检查、文档对齐。
  - Rust 实现（`crates/cli`、`task`、`mem`、`configurator`）。
  - 模板系统 + 评估迭代（`results.tsv`）。
- **优势**：性能好、本地化强、技能颗粒度细、工程化基础扎实。
- **现状**：已使用 Trellis 管理自身项目（.trellis / .pi），但这属于**管理工具层**，不影响 DiJiang 作为独立 harness 的核心。

#### **Trellis 项目** (mindfold-ai/Trellis)
- **定位**：**生态级 Agent Harness 框架**（“The best agent harness”），专注于**持久化规范注入、多平台一致性、任务生命周期管理**。
- **核心**：提供 scaffolding（.trellis/ 目录结构、hooks、sub-agents、bundled skills），让 AI 在任何会话中都遵循项目标准。
- **特点**：高度自动化、团队友好、多平台支持（14+）、spec/task/workspace 持久化。

**关系总结**（更新后）：
- DiJiang **不是 Trellis 的子项目、插件或 fork**。
- DiJiang **主动吸收 Trellis 精华**（理念 + 结构最佳实践），目标是成为**平行且互补的独立实现**。
- 兼容是**阶段性策略**，服务于用户平滑体验，而非长期绑定。

---

### 2. 深度对比（更新视角）

| 维度              | DiJiang（独立 Harness）                      | Trellis（生态框架）                             | 对齐策略 |
|-------------------|---------------------------------------------|------------------------------------------------|----------|
| **核心目标**     | 独立 skill 执行引擎 + 实用 workflow         | 项目级 scaffolding + 持久化 + 多平台一致      | 吸收结构，保持独立实现 |
| **依赖关系**     | 完全自主（Rust CLI）                        | Node CLI + Python 脚本 + 平台 hooks            | 无 runtime 依赖，仅结构兼容 |
| **Workflow**     | S/M/L 分级 + 自定义 chaining                | 4-Phase + auto sub-agents                      | DiJiang 内化 Trellis 理念 |
| **持久化**       | 基于自身 crates（task/mem）+ 输出兼容格式   | 原生 .trellis/tasks/、spec/、workspace/       | 可选生成兼容文件 |
| **Skills**       | Rust 实现，细粒度实战技能                   | Bundled skills（多文件）+ 平台分发             | 标准化格式，独立加载 |
| **兼容性**       | 当前支持临时 Trellis 结构/配置同步         | 原生框架                                       | 可选特性 |
| **扩展性**       | Rust configurator + templates               | 多平台生成器 + marketplace                     | DiJiang 可发布为独立 skill 包 |

**关键洞察**：
- DiJiang 更专注**执行层**（skills + chaining）的深度与性能。
- Trellis 更专注**框架层**（持久化、注入、生态）。
- 二者高度互补：DiJiang skills 可显著增强 Trellis 项目，反之 Trellis 的结构经验可帮助 DiJiang 更工程化。

---

### 3. DiJiang 内部深度剖析（基于 ZIP）
- **架构**：Cargo workspace + CLI 入口，configurator 负责模板与配置，task/mem 处理状态与记忆。
- **Workflow**（`coding-workflow.md`）：完整分级路由 + 决策点 + 回归检查机制，体现系统化思考。
- **迭代证据**（`results.tsv`）：多轮优化记录（补全缺失、dim5 标准、Karpathy 融合），显示强烈的工程迭代文化。
- **模板系统**：default-rust CI、spec、task artifacts 等，易扩展。
- **Trellis 集成痕迹**：.pi/skills/trellis-meta 提供了大量 local-architecture 参考，证明 DiJiang 作者已深度学习 Trellis，但核心实现保持独立。

---

### 4. Trellis 精华提炼（DiJiang 吸收重点）
- **结构化持久化**：task/spec/workspace 目录模型。
- **Workflow 严谨性**：Plan-Execute-Finish + 检查循环 + spec 更新反馈。
- **Skill 组织**：多文件 bundled 风格 + 清晰触发条件。
- **配置与模板**：config.yaml + 可注入上下文。
- **最佳实践**：PRD-driven、verifiable success、团队共享规范。

DiJiang 将这些**转化为自身 Rust 实现**，而非直接复用。

---

### 5. 推荐演进路线（独立优先）
1. **短期**：内化 Trellis 精华到 DiJiang 核心（更新 workflow 文档、标准化 skill 格式、增强 configurator）。
2. **中期**：实现**可选兼容层**（结构生成 + 配置导入导出），默认纯 DiJiang 模式。
3. **长期**：DiJiang 作为独立 harness 成熟后，发布为可独立使用或与各类框架协作的 skill/engine 包。兼容特性按需维护。

**结论**：DiJiang 正走在成为**独立、高质量 Rust Agent Harness** 的正确道路上。通过系统性吸收 Trellis 的框架精华，并保持自身独立性，它有望在 AI 编码工具生态中占据独特且重要的位置——既实用又工程化。

---

此版本已严格按照“**DiJiang 独立 Agent Harness + 吸收 Trellis 精华 + 临时可选兼容**”的定位进行更新。

如果需要进一步细化某个部分（例如独立架构设计、具体兼容模块、roadmap 等），或输出代码/文档草稿，请直接指示。
