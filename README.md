# 帝江 (Dijiang)

> 浑敦无面目，是识歌舞。——《山海经·西山经》

融合 [mattpocock/skills](https://github.com/mattpocock/skills)、[ponytail](https://github.com/DietrichGebert/ponytail)、[Waza](https://github.com/tw93/Waza) 三大 skill 库的工程工作流。

## 定位

三层合一：
- **流程骨架**（mattpocock）：从想法到上线的全链路
- **编码风格**（dj-ponytail）：极简、YAGNI、不过度工程
- **具体习惯**（Waza）：思考、审查、调试、写作的工程纪律

同时提供 Rust 编写的 **`dijiang` CLI**，管理项目生命周期（初始化、状态检查、任务跟踪），兼容 Trellis task 格式。

## 工作流

```
想法 → dispatch（自动分流）
        │
        ├── S级（零碎）→ 直接干
        │
        ├── M级（中等）
        │      ├── M-simple → implement（直接实现）
        │      └── M-phased → grill → dj-tdd → check
        │
        └── L级（完整）→ grill → output → dj-tdd → check → 完成
                                                          ↑_________┘
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
| `dj-tdd` | 红绿重构 | mattpocock/tdd |
| `hunt` | 排查 bug | mattpocock/diagnosing-bugs + waza/hunt |
| `check` | 代码审查 | waza/check + dj-ponytail-review |

### 辅助

| Skill | 触发 | 来源 |
|---|---|---|
| `dj-ponytail` | 极简编码模式 | ponytail |
| `write` | 文字润色、去 AI 味 | waza/write |
| `dj-design` | UI 设计 | waza/design |
| `prototype` | 废品验证 | mattpocock/prototype |
| `script` | 脚本/工具编写 | 新建 |
| `audit` | 过度工程 + 安全扫描 | dj-ponytail-audit |
| `debt` | 捷径追踪 | dj-ponytail-debt |
| `health` | agent 健康检查 | waza/health |
| `handoff` | session 交接 | mattpocock/handoff |

## 安装

```bash
# 复制到 Hermes skills 目录
cp -r dijiang/* ~/.hermes/skills/

# Pi 同理（Pi 读取 ~/.hermes/skills/）
```

## CLI 工具

`dijiang` 是 Rust 编写的命令行工具，管理项目生命周期和 Trellis 兼容的任务。

### init — 初始化项目

```bash
# 创建一个新项目（自动检测 Trellis/DiJiang 目录结构）
dijiang init my-project --yes

# 强制重新初始化（已存在 .dijiang/ 时）
dijiang init my-project --force
```

### status — 查看项目状态

```bash
# 显示项目名、活跃任务、任务列表、Pi 平台状态
dijiang status

# 显示详细兼容诊断（status 映射表、Trellis 检测）
dijiang status --compat
```

### task — 任务管理

```bash
dijiang task list              # 列出所有任务
dijiang task create <name>      # 创建新任务
dijiang start <name>            # 设为活跃任务
```

### mem — 记忆管理

```bash
dijiang mem list                # 列出记忆记录
dijiang mem save                # 保存当前会话记忆
```

## 兼容性

- Hermes ✅
- Pi ✅
- Codex ✅

## 测试验证

```bash
# 全量测试
cargo test

# 分 crate 测试
cargo test -p dijiang-task            # task crate
cargo test -p dijiang-configurator     # configurator crate
cargo test -p dijiang --test e2e       # CLI 集成测试

# 编译检查
cargo build
```
## 设计原则

1. **Predictability** — 每次运行走相同流程，而非产出相同结果
2. **YAGNI** — 不需要的不写，stdlib 能做的不引入依赖
3. **Fail-safe** — 破坏性操作必须确认，回滚必须备份
4. **Composable** — skill 之间可串联，也可单独使用
5. **Runtime-neutral** — 不绑定特定 agent runtime
