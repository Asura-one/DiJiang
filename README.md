# 帝江 (Dijiang)

> 浑敦无面目，是识歌舞。——《山海经·西山经》

融合 [mattpocock/skills](https://github.com/mattpocock/skills)、[ponytail](https://github.com/DietrichGebert/ponytail)、[Waza](https://github.com/tw93/Waza) 三大 skill 库的工程工作流。

## 定位

三层合一：
- **流程骨架**（mattpocock）：从想法到上线的全链路
- **编码风格**（ponytail）：极简、YAGNI、不过度工程
- **具体习惯**（Waza）：思考、审查、调试、写作的工程纪律

## 工作流

```
想法 → dispatch（自动分流）
        │
        ├── S级（零碎）→ 直接干
        │
        ├── M级（中等）→ grill（快速确认）→ implement
        │
        └── L级（完整）→ grill → output → implement/tdd → hunt ↔ check → 完成
                                                              ↑___________┘
                                                          （循环直到没问题）
```

## 全局约束：Git 安全工作流（Worktree-First）

所有涉及 git 操作的 skill 自动遵守以下规则：
1. 主工作区永远干净，只做同步 — 严禁在主目录上直接写代码
2. 每个功能一个独立 worktree — 所有开发、AI 调试均在 worktree 中进行
3. 合并需用户确认 — 展示变更摘要后等待确认
4. 回滚必须备份 + 确认 — tag 备份 → 用户确认 → 执行
5. 禁止自动执行破坏性操作 — `reset --hard`、`force push`、`clean -f`、`rm -rf worktree` 等
6. **Conventional Commits** — 提交信息必须遵循 Conventional Commits 规范（详见 implement/SKILL.md）
7. **语义化版本** — 版本号使用 Major.Minor.Revision 格式（详见 check/SKILL.md）
8. **详细回滚流程** — 备份 tag → 用户确认 → 确认完成三步（详见 hunt/SKILL.md）
## Skill 清单

### 核心流程

| Skill | 触发 | 来源 |
|---|---|---|
| `dispatch` | 任何任务进来时 | 新建 |
| `grill` | 想法细化、调研 | mattpocock + waza/think |
| `output` | 项目文档 + 文档代码对齐 | mattpocock/to-prd + waza/write |
| `implement` | 特性代码实现 | mattpocock/implement |
| `tdd` | 红绿重构 | mattpocock/tdd |
| `hunt` | 排查 bug | mattpocock/diagnosing-bugs + waza/hunt |
| `check` | 代码审查 | waza/check + ponytail-review |

### 辅助

| Skill | 触发 | 来源 |
|---|---|---|
| `ponytail` | 极简编码模式 | ponytail |
| `write` | 文字润色、去 AI 味 | waza/write |
| `design` | UI 设计 | waza/design |
| `prototype` | 废品验证 | mattpocock/prototype |
| `script` | 脚本/工具编写 | 新建 |
| `audit` | 过度工程 + 安全扫描 | ponytail-audit |
| `debt` | 捷径追踪 | ponytail-debt |
| `health` | agent 健康检查 | waza/health |
| `handoff` | session 交接 | mattpocock/handoff |

## 安装

```bash
# 复制到 Hermes skills 目录
cp -r dijiang/* ~/.hermes/skills/

# Pi 同理（Pi 读取 ~/.hermes/skills/）
```

## 兼容性

- Hermes ✅
- Pi ✅
- Codex ✅

## 设计原则

1. **Predictability** — 每次运行走相同流程，而非产出相同结果
2. **YAGNI** — 不需要的不写，stdlib 能做的不引入依赖
3. **Fail-safe** — 破坏性操作必须确认，回滚必须备份
4. **Composable** — skill 之间可串联，也可单独使用
5. **Runtime-neutral** — 不绑定特定 agent runtime
