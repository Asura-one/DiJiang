# DiJiang 优化计划

> 制定日期：2026-06-29
> 制定依据：`docs/DiJiang与Trellis深度调研报告_复核版.md`
> 适用范围：DiJiang 当前 0.x 阶段

---

## 0. 计划定位

### 0.1 核心立场（必须严格遵守）

> DiJiang 是**独立的 Rust-native Agent Harness**。它吸收 Trellis 等优秀框架的精华（结构化 workflow、任务持久化、spec-driven 等最佳实践），但**保持完全独立**——不直接依赖或调用 Trellis（无 runtime 调用、无强绑定）。当前阶段提供**临时、可选的结构与配置兼容**（便于用户在 Trellis 项目中使用 DiJiang skills，或平滑协作），未来可弱化或移除。

这条立场不是宣传话术，而是**设计硬约束**。本计划的所有动作都必须通过下面三条检查：

1. **零 runtime 依赖**：DiJiang 的 `Cargo.toml` 不应引入任何对 `@mindfoldhq/trellis` 或 `@mindfoldhq/trellis-core` 的间接/直接调用通道。Trellis 是 Node/TS 生态，DiJiang 是 Rust 工作区，**物理隔离**本身已经保证这一点——本计划不破坏这个隔离。
2. **兼容是单向可选**：DiJiang 能读 Trellis 产物（task.json 24 字段、`.trellis/` 目录、AGENTS.md 块），是为了让已有 Trellis 项目的用户可以**用 DiJiang skills** 起步；Trellis 读 DiJiang 产物同理（task.json 字段顺序兼容、status 降级）。**任何兼容点都是"能读就行"，不是"调用"**。
3. **兼容层可拆卸**：本计划引入的所有"兼容点"（字段顺序测试、status 降级、AGENTS.md 块标记、init 冲突检测）都必须设计成**可独立回退**的——如果未来决定弱化或移除 Trellis 兼容，删除这些兼容点时**不应**牵连 DiJiang 核心工作流。

**审视清单**（每个动作落地前自问）：
- [ ] 这条改动会让 DiJiang 依赖任何 Trellis 运行时产物吗？ → 必须为否
- [ ] 这条改动是"DiJiang 读 Trellis 产物"还是"DiJiang 调用 Trellis 代码"？ → 只允许前者
- [ ] 如果明年决定"完全不兼容 Trellis"，删除这条改动时 DiJiang 自身会坏吗？ → 必须不坏

### 0.2 本计划覆盖范围与不覆盖范围

**覆盖**：P0 防回归、P1 硬化兼容点、P2 主动降级、P3 体验与文档
**不覆盖**：Mem 模块实现、平台适配器扩张到 18 个、Migration manifest 系统、`.dijiang/` 替代 `.trellis/`

---

## 1. P0 — 防止兼容性破坏

### 1.1 TaskRecord 字段顺序硬约束（防回归）

**问题**：DiJiang `crates/task/src/types.rs` 的 `TaskRecord` 当前以"自然书写顺序"定义字段。Rust 序列化时默认按 struct 字段声明顺序输出 JSON keys，**但**：
- 任何对字段的重排、插入、删除都会破坏与 Trellis task.json 的兼容
- 当前**没有**编译期测试保证这个顺序
- 复核报告 §2.2 已确认 24 字段固定顺序是 Trellis 硬约束

**动作**：
1. 在 `crates/task/src/types.rs` 中加一个 `TASK_RECORD_FIELD_ORDER` 常量（24 个字段名的 `&[&str]`）
2. 加一个 `#[cfg(test)]` 测试：序列化一个全字段填充的 `TaskRecord`，断言输出的 JSON keys 顺序与常量完全一致
3. 在 `TASK_RECORD_FIELD_ORDER` 上加 `#[deprecated]` 不需要——它本身就是公开契约

**验证**：
- `cargo test -p dijiang-task field_order_test` 通过
- 人工 diff：DiJiang 生成的 task.json vs Trellis Python `cmd_create` 生成的 task.json，字段顺序应完全一致

**风险**：低。纯加测试，不改行为。

**完成标准**：测试 + 实际跑一次 `cargo test -p dijiang-task` 看到 `field_order_test ... ok`。

**与核心立场的契合**：
- 这条是"DiJiang 写出的 task.json Trellis 能读"——属于"DiJiang 产物被 Trellis 读"的单向兼容
- **不**引入对 Trellis 的调用
- 删除这个测试时 DiJiang 自身行为完全不变（仅失去兼容保证）

---

### 1.2 修复 Trellis 目录冲突检测（init.rs）

**问题**：`crates/configurator/src/init.rs` 的 `init_project` 在用户目录下**无条件**写 `.trellis/`（含 `tasks/`、`workspace/`、`spec/`、`workflow.md`）。如果用户机器上已经有 Trellis 装的 `.trellis/`，DiJiang 会**静默覆盖** Trellis 的 `workflow.md` 和 `spec/` 内容。

**动作**：
1. 在 `init_project` 入口处加检测：`Path::new(".trellis").exists()`
2. 存在时：
   - 读取现有 `workflow.md`（如有），做内容 hash
   - 与 DiJiang 准备写入的 `workflow.md` 内容 hash 对比
   - 相同：跳过写入并打印"已存在，匹配，跳过"
   - 不相同：**不覆盖**，打印警告并列出冲突文件，要求用户用 `--force` 或 `--merge` 显式确认
3. 加 `--force` 和 `--merge` CLI flag（`Merge` 策略：对 `workflow.md` 走 DiJiang 块插入而非替换；对 `spec/` 跳过；对 `tasks/` 跳过——保护用户任务数据）

**验证**：
- 单元测试：模拟已存在 `.trellis/` 的目录，验证 `init_project` 默认不覆盖
- 集成测试：在一个临时 git 仓库里跑 Trellis init（mock）+ DiJiang init，验证不冲突

**风险**：中。改变了 init 的默认行为，但方向是"更安全"（不破坏用户已有数据）。

**完成标准**：
- `cargo test -p dijiang-configurator init_conflict_test` 通过
- 手动在临时目录跑 `dijiang init` 验证有/无 `.trellis/` 的两种行为

**与核心立场的契合**：
- 这是"DiJiang 不破坏 Trellis 项目已有数据"——属于"DiJiang 与 Trellis 共存"的安全机制
- 没有任何对 Trellis runtime 的调用
- 删除这个检测时 DiJiang 自身行为完全不变（仅失去共存保护）

---

### 1.3 DiJiang 专有状态的跨链降级

**问题**：DiJiang `TaskStatus` 有 5 变体（Planning/InProgress/Completed/Archived/Paused），Trellis 只识别 4 个（planning/in_progress/review/completed）。当 DiJiang 写出 `Archived` 或 `Paused` 状态的 task.json 时，Trellis 工具会读到 `unknown` phase（复核报告 §2.3 确认）。

**动作**：
1. 在 `crates/task/src/types.rs` 加一个 `to_trellis_status()` 方法：
   ```rust
   pub fn to_trellis_status(&self) -> &'static str {
       match self {
           Self::Planning => "planning",
           Self::InProgress => "in_progress",
           Self::Completed => "completed",
           Self::Archived => "completed",  // 降级
           Self::Paused => "in_progress",  // 降级
       }
   }
   ```
2. 序列化逻辑改用 `to_trellis_status()` 而不是 `self` 的 Debug/Serialize 输出
3. 备注：原 status 写入 `meta` 字段（`meta.original_status: "archived"`），Trellis 读到不识别
4. 反向：读 Trellis task.json 时，把 `unknown` 状态的 task 映射到 DiJiang 的 `Paused`（最保守回退）

**验证**：
- 单元测试：5 个 status 变体各自的 `to_trellis_status()` 输出
- 单元测试：读一个 `status: "something_weird"` 的 Trellis task.json 不会 panic

**风险**：低。纯加方法，不改现有调用。

**完成标准**：`cargo test -p dijiang-task status_mapping_test` 通过。

**与核心立场的契合**：
- 这是"DiJiang 产物的字段对 Trellis 工具可读"——属于"产物兼容"层面
- **不**调用 Trellis
- 方法名是 `to_trellis_status`（标记清楚用途），不调用任何外部代码
- 未来若决定完全不兼容 Trellis，可以直接删除这个方法（DiJiang 内部仍用 5 状态，只是不再为 Trellis 做降级）

---

## 2. P1 — 硬化已确认的兼容点

### 2.1 活跃任务标记的双格式兼容

**问题**：`crates/task/src/store.rs::read_active_task()` 已实现"先查 `.runtime/sessions/*.json`，回退到 `.trellis/active_task.txt`"（复核报告 §3.5 确认）。但**没有写时兼容**：
- DiJiang 写活跃任务时只写 `.runtime/sessions/*.json`
- 如果用户在 DiJiang 和 Trellis CLI 之间切换，Trellis 写 `.trellis/active_task.txt` 时 DiJiang 完全感知不到
- 反过来 DiJiang 写 `.runtime/sessions/` 时 Trellis CLI 也读不到

**动作**：
1. `write_active_task` 时**双写**：写 `.runtime/sessions/<id>.json` 同时也写 `.trellis/active_task.txt`（仅 task id 一行）
2. 读时按现有逻辑（先 sessions 后 active_task.txt）
3. 加一个 `.runtime/.trellis_owned` 标记文件，声明 DiJiang 拥有 `.runtime/` 子树（避免未来读时和 Trellis 冲突）

**验证**：
- 单元测试：调用 `write_active_task` 后两个路径都存在
- 单元测试：删除 `.runtime/sessions/<id>.json` 后 `read_active_task` 仍能从 `.trellis/active_task.txt` 读到

**风险**：低。增量行为，不影响已有调用。

**完成标准**：`cargo test -p dijiang-task active_task_dual_write_test` 通过。

**与核心立场的契合**：
- "DiJiang 写出的活跃标记 Trellis CLI 能读"——单向产物兼容
- **不**调用 Trellis
- 标记文件 `.runtime/.trellis_owned` 明确划清 DiJiang 与 Trellis 的目录所有权
- 删除时仅失去兼容，DiJiang 单边行为不变

---

### 2.2 AGENTS.md 双注入的块标识

**问题**：`crates/configurator/src/pi.rs` 已知会注入 DiJiang 风格块 + Trellis 风格块到 `AGENTS.md`（复核报告 §3.2 确认）。但**没有块标识**：
- 重复跑 `dijiang init` 会重复注入
- 用户手动编辑 AGENTS.md 后再跑 init，可能把 DiJiang 块的位置搞乱

**动作**：
1. 在 DiJiang 风格块前后加显式标记：
   ```markdown
   <!-- BEGIN DIJIANG-MANAGED BLOCK: do not edit between these markers -->
   ... 内容 ...
   <!-- END DIJIANG-MANAGED BLOCK -->
   ```
2. 在 Trellis 风格块前后也加对应标记（`BEGIN TRELLIS-COMPAT BLOCK`）
3. init 时按标记查找并替换块内容，而不是无脑 append

**验证**：
- 单元测试：跑 init 两次，AGENTS.md 中每个标记块只出现一次
- 单元测试：用户编辑了标记外的内容后跑 init，外内容不变

**风险**：中。改 AGENTS.md 处理逻辑，需保证向后兼容（已存在但无标记的 AGENTS.md 也能正确处理——fallback 到"追加而非替换"）。

**完成标准**：
- 单元测试覆盖：干净项目 / 已有 AGENTS.md 无标记 / 已有 AGENTS.md 有标记 三种场景
- 手动：临时仓库跑 3 次 init，AGENTS.md 不会无限增长

**与核心立场的契合**：
- Trellis 风格块的存在是为"已在 Trellis 项目里用 DiJiang skills"的用户服务——属于"共存场景"的产物
- 块标记的命名（`DIJIANG-MANAGED` vs `TRELLIS-COMPAT`）清楚区分"DiJiang 自己管理"和"为 Trellis 兼容而注入"
- 删除 Trellis 兼容块时仅失去"在 Trellis 项目里用 DiJiang"的能力，DiJiang 自身 AGENTS.md 注入不受影响

---

## 3. P2 — 主动降级明确不兼容项

### 3.1 Trellis 未知状态的优雅降级

**问题**：与 §1.3 反向。读 Trellis task.json 时，Trellis 可能写出 `status: "in_progress"`（带下划线）而 DiJiang 期望的是 `in_progress`（一致），但如果未来 Trellis 加新状态（如 `blocked`、`cancelled`），DiJiang 现在的 `Deserialize` 会直接失败。

**动作**：
1. `TaskStatus` 改用 `#[serde(other)]` 或自定义 `Deserialize`：未知 status 落到 `Paused`（最保守）
2. 写时保持现有 5 变体不变
3. 在 `meta.unmapped_status` 字段记录原始 status 字符串

**验证**：
- 单元测试：反序列化 `{"status": "blocked", ...}` 得到 `TaskStatus::Paused`，`meta.unmapped_status == Some("blocked")`

**风险**：低。

**完成标准**：`cargo test -p dijiang-task unknown_status_test` 通过。

**与核心立场的契合**：
- "DiJiang 读 Trellis 产物时不出错"——属于"读侧兼容"
- **不**调用 Trellis
- 这是"容错"行为，不是"依赖"行为——删除时 DiJiang 自身工作流不变

---

### 3.2 Skills 路径不兼容的明确提示

**问题**：DiJiang 用 `.pi/skills/`（单文件 SKILL.md），Trellis Pi configurator 也用 `.pi/skills/`（但内容是多文件 bundled skills）。两者**物理上**写同一目录，**逻辑上**互不识别对方的 skill。

**动作**：
1. DiJiang skill 的 `SKILL.md` 文件名加 `dijiang-` 前缀（**已做到**：`dijiang-start`、`dijiang-continue`、`dijiang-finish-work` 都在）
2. 加一个 `.pi/skills/.dijiang_owned` 标记文件
3. 跑 init 时检测 `.pi/skills/` 下已有非 `dijiang-*` 目录，**不覆盖**，仅打印"检测到非 DiJiang skills，请手动检查冲突"
4. **不**做 Trellis skill → DiJiang skill 的格式转换（复核报告 §5.4 明确不做）

**验证**：
- 单元测试：mock 一个含 `trellis-foo` 目录的 `.pi/skills/`，验证 init 不删除它

**风险**：低。

**完成标准**：单元测试 + 手动在临时目录跑 init 看到提示信息。

**与核心立场的契合**：
- "DiJiang 不破坏 Trellis 在 `.pi/skills/` 写的内容"——属于"共存安全"
- **不**调用 Trellis
- `.dijiang_owned` 标记文件清楚划清目录所有权

---

## 4. P3 — 体验与文档

### 4.1 CLI 增强：status 子命令显示兼容状态

**问题**：`crates/cli/src/main.rs` 有 `Status` 子命令，但当前只显示基本任务信息。用户跑 `dijiang status` 时看不到：
- 当前 task 的 Trellis phase 是什么
- DiJiang task.json 是否与 Trellis 兼容
- `.trellis/` 目录是否被 DiJiang 拥有

**动作**：
1. `Status` 子命令输出加以下字段：
   ```
   Trellis Phase:     implement
   Compatible:        yes
   Original Status:   in_progress (DiJiang: Paused → mapped to in_progress)
   ```
2. 加 `--compat` flag 输出详细兼容诊断

**验证**：
- 单元测试：5 种 status 变体的 `Status` 输出格式正确

**风险**：低。仅改输出格式。

**与核心立场的契合**：
- 仅仅是"展示 DiJiang 自己的 task 与 Trellis 字段的对应关系"——是诊断信息
- **不**调用 Trellis
- 删除时仅失去诊断输出，DiJiang 自身工作流不变

---

### 4.2 文档：明确"DiJiang 不是 Trellis 替代品"

**问题**：复核报告 §5.1 指出 DiJiang 定位应是"轻量级 Trellis 兼容子集"。但当前没有用户文档传达这个定位。

**动作**：
1. 在 `docs/DiJiang与Trellis深度调研报告_复核版.md` 末尾追加一节"§8 用户定位指南"（避免新建文件）
2. 内容包含：
   - DiJiang 与 Trellis 的关系（不是替代，是兼容子集）
   - 24 字段兼容契约（指向 §1.1 的测试）
   - 不做的事清单（mem 模块、平台扩张、migration 兼容）
   - 适合用 DiJiang 的场景 vs 适合用 Trellis 的场景

**风险**：低。纯文档。

**完成标准**：在复核报告追加一节"§8 用户定位指南"，200-400 字。

---

### 4.3 Init 输出加 Trellis 检测提示

**问题**：`dijiang init` 跑完时，如果检测到当前目录有 Trellis 痕迹（如 `package.json` 含 `@mindfoldhq/trellis` 依赖，或 `.trellis/` 中有 DiJiang 不会写的内容），应主动提示。

**动作**：
1. init 结束时扫一遍：
   - `package.json`（pnpm 项目）含 Trellis 依赖 → 提示"检测到 Trellis 项目，DiJiang 将与 Trellis 共存"
   - `.trellis/scripts/` 存在（DiJiang 不写 scripts）→ 提示"检测到 Trellis scripts，DiJiang 不会修改它们"
2. 输出到 stdout 末尾，不影响 init 成功/失败

**验证**：
- 单元测试：mock 含 Trellis 依赖的 package.json，验证提示信息出现

**风险**：低。

**完成标准**：`cargo test -p dijiang-configurator init_detection_test` 通过。

**与核心立场的契合**：
- 这是"用户友好提示"，不是"调用"——**不**调用 Trellis
- 是 DiJiang 主动告诉用户"我检测到 Trellis 痕迹，请注意我们独立工作"
- 删除时仅失去提示，DiJiang 自身工作流不变

---

## 5. 不做的事（明确边界）

| 不做 | 理由 | 复核报告依据 |
|------|------|-------------|
| Mem 模块实现 | ROI 太低，会引入 4 个适配器（claude/codex/opencode/pi） | §5.4 |
| 平台 configurator 扩张到 18 个 | DiJiang 6 个已覆盖核心场景 | §5.5 |
| Mem API 兼容（与 Trellis 子路径对齐） | 决定了不做 mem 就不需要这条 | §5.4 |
| CLI 子命令名兼容 Trellis 20 个 channel 子命令 | 哲学差异，DiJiang 偏 SKILL 引导 | §5.6 |
| Migration manifest 系统 | DiJiang 仍在 0.x 早期 | §0 |
| `.dijiang/` 目录替代 `.trellis/` | 已经做兼容，不要回退 | §7 决策表 |
| 把 DiJiang 专有状态（Paused/Archived）合并掉 | 满足 DiJiang 内部工作流需要 | §1.3 决策 |
| **引入 Trellis 运行时依赖** | 核心立场 §0.1 | §0.1 |
| **为兼容而写"DiJiang 调用 Trellis 代码"的桥接层** | 核心立场 §0.1 | §0.1 |
| **把 Trellis 状态机/调度器移植到 DiJiang** | 核心立场 §0.1——会变成"DiJiang 复刻 Trellis"，破坏独立 | §0.1 |

---

## 6. 实施顺序与里程碑

按依赖关系排序（每完成一个里程碑必须有测试 + 手动验证）：

| 里程碑 | 包含项 | 依赖 | 预估产出 |
|--------|--------|------|----------|
| **M1: 兼容性硬化** | §1.1 字段顺序测试 + §1.3 状态降级 + §3.1 未知状态回退 | 无 | 3 个测试通过，task crate 行为更可预测 |
| **M2: 写时安全** | §1.2 init 冲突检测 + §2.1 双写活跃任务标记 + §3.2 skills 路径检测 | 无（与 M1 并行） | 4 个测试通过，init 不再静默破坏用户数据 |
| **M3: 体验提升** | §2.2 AGENTS.md 块标记 + §4.1 status 输出 + §4.3 init 检测提示 | M1, M2 | 3 个测试通过，CLI 输出更友好 |
| **M4: 文档收尾** | §4.2 在复核报告追加"用户定位指南"章节 | 无 | 文档 200-400 字 |

**M1 优先**：因为它纯加测试+加方法，零行为变更风险，最容易评审。

**M2 次之**：改变了 init 默认行为，需要仔细评审（涉及用户已有数据保护）。

**M3 延后**：体验优化，重要性低于数据安全。

**M4 收尾**：文档工作可以最后做。

---

## 7. 验证矩阵

每个里程碑完成后必须跑：

| 命令 | 目的 |
|------|------|
| `cargo test -p dijiang-task` | 验证 task crate 所有测试通过 |
| `cargo test -p dijiang-configurator` | 验证 configurator crate 所有测试通过 |
| `cargo test` | 全工作区测试通过 |
| `cargo build -p dijiang` | 全 release build 通过（**注意** session 记录：编辑 configurator 后必须 build 不只是 test） |
| 临时仓库手动 init 一次 | 验证 CLI 端到端可用 |

---

## 8. 风险登记

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| §1.2 init 冲突检测改变了用户既有工作流 | 中 | 中 | `--force` flag + 详细 changelog |
| §2.2 AGENTS.md 块标记对历史项目不兼容 | 中 | 低 | Fallback 到"追加而非替换"逻辑 |
| §1.3 状态降级让 DiJiang 内部行为微妙变化 | 低 | 低 | 写 `meta.original_status` 保留可追溯性 |
| 用户期望 DiJiang 替代 Trellis（定位错位）| 中 | 中 | §4.2 文档 + init 提示（§4.3）|
| Cargo build 因 rust-embed 资源变更失败 | 低 | 中 | M2 之后跑一次 `cargo build --release` 验证 |
| **未来误把"兼容"扩展成"依赖"** | 中 | 高 | 核心立场 §0.1 审视清单（每个 PR 必查） |

---

## 9. 立场自检工具

未来每个 PR 涉及本计划任何动作时，PR 描述必须包含以下自检（**不可省略**）：

```markdown
## 核心立场自检

- [ ] 本改动未引入对 Trellis 运行时（`@mindfoldhq/trellis` / `@mindfoldhq/trellis-core`）的任何调用
- [ ] 本改动仅属于"DiJiang 产物对 Trellis 工具可读"或"DiJiang 读 Trellis 产物"的单向兼容
- [ ] 删除本改动时，DiJiang 自身工作流（不依赖 Trellis 的部分）保持完好
- [ ] 本改动的命名/注释清晰表明用途（例如方法名 `to_trellis_status` 而非 `serialize_status`）
```

---

## 附录：与 session 历史的关键衔接

- **路径冲突**：session 已确认 DiJiang 根在 `.trellis/tasks/`（不是 `.dijiang/tasks/`），本计划与此一致
- **Pi 0.80 API**：session 已确认使用 `@earendil-works/pi-coding-agent` factory function 模式，本计划不涉及 Pi 适配器代码（Pi 适配器深化不在本计划范围）
- **cargo build 不只是 cargo test**：session 已确认编辑 configurator 后必须 `cargo build -p dijiang` 而非仅 `cargo test`，本计划 §7 验证矩阵已包含
- **字段顺序契约**：Trellis 的 `TASK_RECORD_FIELD_ORDER`（TS 端）+ Python `TaskData`（Python 端）必须结构一致，本计划 §1.1 是把 DiJiang 也加入这个契约
- **新立场强化**：用户明确重申 DiJiang 必须保持完全独立，本计划 §0.1 已转化为三条设计硬约束 + §9 立场自检工具

---

> 计划完。本计划基于复核报告的 8 个事实修订，每个动作都有验证标准。优先级 P0 > P1 > P2 > P3，与复核报告的决策矩阵（§5）保持一致。核心立场（独立 Rust-native Agent Harness，零 Trellis runtime 依赖，兼容层可拆卸）已转化为 §0.1 的三条设计硬约束 + §9 PR 自检工具，确保未来不会被无意侵蚀。
