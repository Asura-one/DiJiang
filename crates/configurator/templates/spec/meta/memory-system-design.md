# DiJiang 记忆系统设计

## 层次架构

```
全局 Hermes memory (~/.hermes/memories/)
    memory.md (2.2KB)    ← 环境 quirks + 稳定偏好 + 反复踩坑
    user.md (1.4KB)      ← 用户画像：身份、偏好、风格
    │  注入每个 session，不分项目
    │
DiJiang 全局 memory (~/.dijiang/memory/)
    findings.jsonl        ← dijiaang mem findings
    learnings.jsonl       ← dijiaang mem learn
    corrections.jsonl     ← dijiaang mem correction
    ledger.jsonl          ← tactics + project 字段标记
    │  通过 muse_retrieve 四路召回，可按 project 过滤
    │
项目本地 memory (.dijiang/memory/)
    findings.jsonl        ← 本项目 findings
    learnings.jsonl       ← 本项目 learnings
    corrections.jsonl     ← 本项目 corrections
    │  天然项目隔离
```

## 记忆门禁（Memory Gate）

进入全局 Hermes memory 的条目必须满足以下条件之一：

| 类别 | 示例 | 准入 |
|------|------|------|
| 稳定偏好 | "用户要求中文优先" | ✅ |
| 环境 quirks | "macOS brew python 在 /opt/homebrew" | ✅ |
| 反复踩坑 | "Go 测试需 -tags unit" | ✅ |
| 稳定约定 | "项目用 Makefile" | ✅ |
| 项目敏感信息 | "社工演练 gophish 路径" | ❌ → 项目本地 |
| 架构决策 | "SKILL.md 结构设计" | ❌ → ADR/skill |
| 工作流步骤 | "前端先加载 dj-design" | ❌ → skill |
| 任务跟踪 | "正在做 PR #42" | ❌ → session |
| 可复现知识 | "先 build 再 test" | ❌ → skill |

## 容量预算

```
memory.md 2,200 字符 — 当前 76%
user.md   1,375 字符 — 当前 99%
```

每个字必须值回票价。超过 85% 时触发自动精简。

## 晋升通道

项目本地的经验 → 经过验证 → 晋升到 DiJiang 全局 → 筛选后进 Hermes 全局

```
项目本地 memory
    ↓ (dijiang mem backup)
DiJiang 全局 memory
    ↓ (muse_learn_promote + memory gate)
Hermes 全局 memory / skill / ADR
```

晋升门禁：必须同时满足：
1. 跨项目有效（不是某个项目特有的）
2. 高频使用（不是一次性事实）
3. 不易过期（不是老得快的信息）

## 关联

- 相关技能：`diang mem findings/learn/correction/tactic/backup`
- 相关规范：`.dijiang/spec/guides/memory-lifecycle-guide.md`
- 设计决策：ADR 001（skill 精简）、ADR 003（Phase 3 改进）
