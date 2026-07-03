# 测试标准

## 测试架构

DiJiang 使用两层测试策略：

```
crates/cli/tests/e2e.rs          # 集成：完整二进制作为子进程执行
crates/*/src/**/*.rs (mod tests)  # 单元：crate 内部逻辑
```

| 层 | 测试数 | 范围 | 运行方式 |
|----|--------|------|----------|
| E2E | 47 | CLI 二进制，通过子进程调用 | `cargo test -p dijiang` |
| 单元（cli） | 20 | `main.rs` 中的命令处理器 | `cargo test -p dijiang --lib` |
| 单元（task） | 74 | Route gate、git gate、spec/doc sync | `cargo test -p dijiang-task` |
| 单元（configurator） | 50 | Init、模板、更新、注册表 | `cargo test -p dijiang-configurator` |
| 单元（mem） | 16 | 存储、适配器、序列化 | `cargo test -p dijiang-mem` |
| **总计** | **~207** | | `cargo test --workspace` |

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

### Crate: `task`

| 模块 | 要求覆盖 | 关键场景 |
|------|---------|----------|
| `route_gate` | 12+ 测试 | 每个任务状态的 capsule 评估，`planning` 的重定向规则，`completed` 的阻断 |
| `spec_sync` | 12+ 测试 | SHA256 比较，check vs record，文件变更检测，无变更时无操作 |
| `doc_sync/analyzer` | 8+ 测试 | Diff 解析，变更事件分类，多文件 diff |
| `doc_sync/mapper` | 10+ 测试 | 事件→文档映射，置信度评分，触发证据提取 |
| `store` | 7+ 测试 | 任务创建/读取/更新，状态转移，边界情况 |
| `git_gate` | 4+ 测试 | Worktree 就绪检测，已有 worktree 检测 |
| `capability_gate` | 4+ 测试 | integrate/push/cleanup 的批准条件 |
| `workflow_state` | 5+ 测试 | 状态注入，对等工作流面 |
| `skill_manifest` | 9+ 测试 | 注册表填充，懒加载，body 缓存 |

### Crate: `configurator`

| 模块 | 要求覆盖 | 关键场景 |
|------|---------|----------|
| `templates` | 17+ 测试 | 每个平台的模板渲染，变量替换 |
| `template_registry` | 9+ 测试 | 模板发现，fallback，版本化 |
| `pi` | 7+ 测试 | Pi 平台配置生成，hook 注入 |
| `init` | 5+ 测试 | 脚手架生成，冲突检测，强制覆盖 |
| `registry` | 6+ 测试 | 平台注册，发现，去重 |
| `update` | 3+ 测试 | Hash 比较，GitHub 下载，本地 fallback |
| `changelog` | 3+ 测试 | 输出格式化，版本映射 |

### Crate: `mem`

| 模块 | 要求覆盖 | 关键场景 |
|------|---------|----------|
| `store` | 3+ 测试 | JSONL 追加/读取，项目范围查找 |
| 平台适配器 | 各 2-4 测试 | 适配器创建，平台不可用时 `sync` 优雅降级 |

### Crate: `cli`

| 区域 | 要求覆盖 | 关键场景 |
|------|---------|----------|
| E2E（二进制） | 47+ 测试 | 完整工作流：init→start→dispatch→finish-work，所有子命令 |
| 单元（main.rs） | 20+ 测试 | 命令解析，dispatch 逻辑，路径解析 |

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

```bash
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
