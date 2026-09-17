---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '21588a9a-1bab-4771-8a57-3f9f18c32b73'
  PropagateID: '21588a9a-1bab-4771-8a57-3f9f18c32b73'
  ReservedCode1: '49516a37-dfae-4e8b-9f7c-ae2719712233'
  ReservedCode2: '49516a37-dfae-4e8b-9f7c-ae2719712233'
---

# ADR 0005: 从 Trellis 兼容迁移到纯 Skill 架构

**状态**: Accepted  
**日期**: 2026-09-17  
**决策者**: 老大

## 上下文

DiJiang 最初是融合 mattpocock/skills、ponytail 和 Waza 三大 skill 库的工程工作流框架，以 Rust CLI 运行时为核心，同时保持与 Trellis 规范的兼容性。随着项目发展，发现以下问题：

1. **Trellis 兼容约束过重**：task.json 24 字段固定顺序、状态降级映射、.trellis/ 读回退等兼容代码增加了维护成本，但实际使用场景有限。
2. **Rust 运行时与 agent 耦合**：Route Gate、Git Gate、Capability Gate 三个运行时门禁依赖 Rust CLI 在 PATH 中可用，限制了 agent 选择。
3. **平台适配层维护成本高**：configurator 为 6 个平台（Pi/Claude/Codex/Cursor/OpenCode/Hermes）生成配置，每个平台变更都需要更新 Rust 代码。
4. **mattpocock/skills 证明了纯 skill 方式的可行性**：小、可组合、可改造的 skill 集合，无运行时依赖，适用范围更广。

## 决策

放弃 Trellis 规范对照，改为基于 mattpocock/skills 的纯 skill 设计思路：

1. **移除全部 Rust 代码**（5 个 crate：cli/task/mem/configurator/mcp-server）
2. **移除 Trellis 兼容层**（24 字段 schema、.trellis/ 回退、状态降级映射）
3. **Skill 重组**：按 mattpocock/skills 方式组织到 `skills/<bucket>/<name>/SKILL.md`
4. **Frontmatter 对齐**：采用 mattpocock 格式（name/description/disable-model-invocation）
5. **门禁下沉**：Route/Git/Capability Gate 从运行时硬约束变为 skill 文本纪律
6. **记忆保留为 skill**：dj-memory（model-invoked）替代 Rust mem crate
7. **任务管理简化**：task.json 从 24 字段简化为 8 字段，支持可配置后端（默认本地文件）
8. **领域语言对齐**：新建 CONTEXT.md 替代 glossary.md，ADR 迁移到 docs/adr/

## 放弃的方案

- **保留 Rust CLI 瘦身版**：仍需维护 Rust 工具链和平台适配层，与"纯 skill"目标冲突。
- **保留 Trellis 兼容层**：维护成本高，使用场景小众。
- **保留运行时门禁**：依赖 CLI 在 PATH 中可用，限制 agent 选择。

## 影响

- 项目从 Rust workspace 变为纯 skill 集合，无需编译
- 新用户只需复制 skills/ 目录 + 运行 dj-setup 初始化
- 工作流纪律从运行时强制变为 skill 文本引导（依赖模型遵守）
- 记忆系统从 Rust 五层记忆 + Thompson sampling 简化为 JSONL 文件读写
- 平台配置从 configurator 生成变为 AGENTS.md/CLAUDE.md 手动维护