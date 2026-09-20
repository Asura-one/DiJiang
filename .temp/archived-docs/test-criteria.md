# 测试标准

## 测试架构

DiJiang 使用两层测试策略：

```
crates/cli/tests/e2e.rs          # 集成：完整二进制作为子进程执行
crates/*/src/**/*.rs (mod tests)  # 单元：crate 内部逻辑
```

| 层 | 范围 | 运行方式 |
|----|------|----------|
| E2E | 以完整 CLI 二进制验证工作流和用户可见行为 | `cargo test -p dijiang --test e2e` |
| Crate 单元测试 | 验证 task、configurator、mem、CLI 等 crate 的公共接口与边界逻辑 | `cargo test --workspace --lib` |
| Contract tests | 验证 Pi extension 与 managed skill registry 契约 | `make pi-extension-contract`、`make validate-skills` |

测试数量会随功能增长，不作为稳定契约。`cargo test --workspace` 是否全部通过以及关键行为是否有回归覆盖才是验收依据。

## 测试原则

### 1. 测试外部行为，而非实现

通过公共 API 调用并断言可观测输出。不要断言中间状态、私有函数调用或内部数据结构。

- ✅ 正确：`dijiang status` 返回包含正确任务状态的 JSON。
- ❌ 错误：断言 `route_decision()` 使用正确参数调用了 `evaluate_capsule()`。

### 2. 使用最高可用的 seam

| Seam | 优先使用时机 |
|------|-------------|
| CLI 二进制（子进程） | 测试 dispatch、workflow、门禁、finish-work、channel、mem 命令 |
| 库公共 API | 测试 task store、route gate 逻辑、doc-sync 分析、configurator init |
| 独立测试辅助函数 | 测试解析器、序列化器、数据变换等无需 crate 上下文的逻辑 |

E2E seam（`cli/tests/e2e.rs`）是主要测试面。库单元测试为隔离逻辑提供快速反馈。

### 3. 仅在系统边界测试

不要为无法从公共接口到达的内部状态添加错误处理、校验或测试断言。信任类型系统和框架保证。

### 4. 编译并缓存二进制一次

E2E 测试应复用单一编译好的二进制。当前方案使用 `CARGO_BIN_EXE_dijiang` 自动检测，fallback 到 `target/debug/dijiang`。

## 覆盖要求

覆盖要求按行为边界维护，不规定固定测试数量：

| Crate / 测试面 | 必须覆盖的行为 |
|----------------|----------------|
| `task` | 任务状态与 route/git/capability gates；task store containment；spec/doc sync；context manifest 安全、完整性与行号错误；skill manifest 路由和完整名称集合 |
| `configurator` | 各平台配置生成；初始化与更新；template registry 安装完整性；managed skill YAML frontmatter schema 与模板名称集合 |
| `mem` | JSONL 存储、归档、备份和平台适配器在不可用时的显式降级 |
| `cli` | 完整任务生命周期、所有公开子命令 help、参数冲突、失败退出状态和跨 crate 生产调用链 |
| Pi contract | Extension 事件注册、上下文注入、字符预算和 managed artifact inventory |

新增或修复行为必须附带能在修改前失败、修改后通过的回归测试。共享边界或跨 crate 契约应同时保留公共接口测试和至少一个生产调用链测试。

## 测试事项（按层）

### 门禁行为

- Route Gate：每个任务状态产生正确 `action`（`allow`/`redirect`/`block`）
- Git Gate：有 worktree 或无 worktree 需求时为 `ready`；创建后为 `provisioned`；冲突时为 `blocked`
- Capability Gate：每个破坏性操作为 `approved`/`denied`

### Doc-sync

- 每种变更事件类型（pub API、新增模块、测试变更等）映射到正确的文档类型
- 单个 diff 中的多个变更事件产生多条受影响文档记录
- 置信度不小于 0，不大于 1.0

### Spec-sync

- Spec 文件内容变更时 SHA256 hash 变化
- `check` 报告已更文件；`record` 更新 checksum 数据库
- 新增/删除 spec 文件被检测
- 无变更文件：`check` 报告空结果；`record` 无操作

### CLI 工作流

- `dijiang init` 创建正确的 `.dijiang/` 和 `.pi/` 结构
- `dijiang start` → `dijiang task current` → `dijiang status` 显示正确状态
- `dijiang finish-work --verification "done"` 在有效状态下执行成功
- `dijiang finish-work` 在无效状态下返回错误
- 所有子命令的 `--help` 产生非空输出
- 所有命令输出已中文本地化

### Context 与 Skill 完整性

- Context 路径拒绝项目外目标、`.env*`、credential 组件和高风险配置目录，并覆盖项目内 symlink 指向敏感目标的情况。
- Context manifest 遇到损坏 JSONL 时返回 manifest 路径与 1-based 行号；合法空行和普通路径继续可用。
- Embedded skill context 只移除完整闭合 block；未闭合 `<skill ...` 保留原始文本供路由使用。
- Managed skill validation 覆盖 frontmatter 边界、YAML 类型、目录名与 `name` 一致、非空 `description`、可选字段类型及 task runtime manifest/template 集合一致性。
- `dijiang skills --sync` 与 `--validate` 参数互斥，CLI validation 测试必须经过生产调用链。

### 记忆

- `dijiang mem findings --finding "x"` 追加到项目记忆
- `dijiang mem archive` 归档当前 session
- `dijiang mem backup` 同步到全局存储

## 不测试的事项

- 不公开的内部实现（私有函数、中间状态）
- 平台不可用时的平台特定行为（适配器优雅降级）
- 外部服务（GitHub API、平台记忆 API）—— 需 mock 或跳过
- 错误消息逐字比较 —— 按语义内容比较，而非字符串相等
- 性能特性 —— 除非存在回归基准测试

## 运行测试

权威本地 gate 为 `make ci`，按顺序执行构建、`git diff --check`、workspace check、workspace tests、Pi extension contract 和 managed skill validation。不要用 PATH 中可能过时的全局 `dijiang` 替代构建后的 `./target/debug/dijiang`。

```bash
# 完整 CI gate
make ci

# 全工作空间
cargo test --workspace

# 单个 crate
cargo test -p dijiang-task

# 仅 E2E（需先编译二进制）
cargo test -p dijiang --test e2e

# 单个测试
cargo test -p dijiang-task -- route_gate::test_planning_redirects

# 快速反馈（跳过 E2E）
cargo test --workspace --lib
```
