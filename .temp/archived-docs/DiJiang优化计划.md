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

## 7A. Canonical workflow model 与边界定义

### 7A.1 问题定位

当前割裂不只是 `review` 命名冲突，而是 CLI、skills、workflow、AGENTS.md、prompt、agent、平台模板分别定义了局部流程。后续改造必须先承认一条原则：DiJiang 只能有一套 canonical workflow model，其他文件都是它的投影。

本节只定义模型和迁移边界，不改变现有 CLI 行为，也不批量改 skill 内容。后续 README、AGENTS、workflow、skills、CLI 对齐都必须以本节为准。

### 7A.2 Canonical layer boundary

| 层级 | Canonical 责任 | 允许出现的内容 | 不应承担的内容 |
|------|----------------|----------------|----------------|
| `dijiang` CLI | 项目状态、任务生命周期、记忆持久化、模板/平台管理、agent channel 运行时 | `init/update/status/start/finish-work/task/mem/template/skills/workflow-state/channel` | 不定义 skill 方法论，不把 prompt 生成器伪装成执行器 |
| `.dijiang/workflow.md` | 高层流程编排和路由规则 | 阶段、状态、推荐 skill、产物、完成标准 | 不复制每个 skill 的完整手册，不引用不存在命令 |
| `AGENTS.md` | 给 agent 注入的最小入口索引 | 项目结构、常用 CLI、canonical skill routing、状态路由 | 不维护另一套 workflow，不列不存在 skill/status |
| `dj-*` skills | 原子工作能力 | `grill/implement/check/audit/output` 等执行细则 | 不管理 session 生命周期，不定义全局状态模型 |
| `dijiang-*` skills | session wrapper | `start/continue/finish-work` 三个入口，加载上下文并路由 | 不成为第二套工作流，不直接替代 `dj-*` 能力 |
| `.pi/prompts/*` | 平台快捷 prompt | 调用 canonical CLI/skill 的简短入口文案 | 不维护独立 workflow，不指向 legacy 主路径 |
| `.pi/agents/*` 与跨平台 agents | 子代理聚合器 | `implement/check/research` role 的上下文加载和委派规则 | 不创造新状态、新命令语义或平台专属 workflow |
| 平台 configurator | 平台适配 | hook、agent config、context injection | 不各自写一套 DiJiang 流程说明 |

### 7A.3 Canonical lifecycle

| Task status | Workflow phase | 推荐入口 | 产物与完成标准 |
|-------------|----------------|----------|----------------|
| none | dispatch | `dijiang start <name>` 或 `dj-dispatch` | 创建/选择任务，判断任务级别与主路径 |
| `planning` | align | `dj-grill`，必要时 `dj-output` | `prd.md`，复杂任务补 `design.md` / `implement.md` |
| `in_progress` | implement | `dj-implement` / `dj-tdd` / `dj-hunt` / `dj-script` / `dj-design` / `dj-absorb` | 代码、测试、验证记录，准备进入质量闸门 |
| `in_progress` | check | `dj-check` | diff、功能完整性、安全性、回归影响通过检查 |
| `completed` | finish | `dijiang finish-work --verification ...` | journal、session closure、active task 清理、归档 |
| `archived` | closed | 只读；如需继续则重新 `dijiang start <task>` | 不继续在旧任务上工作 |
| `paused` | resume | `dijiang-continue` + 读取 workspace journal | 恢复上下文后回到 `planning` 或 `in_progress` 对应路径 |

`review` 不作为 canonical task status。Trellis 兼容层可继续把 legacy `review` 读成 DiJiang 可处理的状态，但 workflow、AGENTS、skills 不应再把 `review` 当作正式状态。

### 7A.4 Canonical skill taxonomy

| 类别 | Canonical skills | 边界 |
|------|------------------|------|
| Routing | `dj-dispatch` | 只负责分类与路由，不直接实现 |
| Alignment | `dj-grill` | 需求澄清和范围对齐，不写代码 |
| Planning docs | `dj-output` | PRD、design、implement、spec 与代码一致性 |
| Implementation | `dj-implement`、`dj-tdd`、`dj-hunt`、`dj-prototype`、`dj-script`、`dj-design`、`dj-absorb` | 允许改代码；按任务类型选择 primary skill |
| Quality gate | `dj-check` | 改动完成前的唯一质量闸门 |
| Analysis reports | `dj-audit`、`dj-debt`、`dj-health`、`dj-pattern` | 产出报告或建议；除明确要求外不作为交付闸门 |
| Style overlays | `dj-ponytail`、`dj-karpathy` | 作为约束叠加到实现路径，不单独代表 workflow 阶段 |
| Writing polish | `dj-write` | 文本润色、去 AI 味、proofread；不拥有工程文档生命周期 |
| Session transfer | `dj-handoff` | 跨 session 交接，不替代 finish-work journal |
| Session wrappers | `dijiang-start`、`dijiang-continue`、`dijiang-finish-work` | 加载上下文、路由、收尾；不复制 `dj-*` 方法论 |

### 7A.5 Known split points to resolve

| 类型 | 割裂点 | 目标状态 |
|------|--------|----------|
| 重复入口 | `dijiang start` 与 `dijiang task start` | 前者定义为生命周期入口，后者定义为 task 原子操作，并在 README/AGENTS/help 中说明 |
| 重复入口 | `dijiang finish-work` 与 `dijiang task archive/status` | 前者定义为完成工作流，后者定义为底层状态操作 |
| 语义冲突 | `dijiang review` / `dj-review` / `dj-check` | `dj-check` 成为 canonical quality gate；`review` 成为 check mode、preset 或 deprecated alias |
| 状态不一致 | workflow/AGENTS 中的 `review` 状态 | 移除正式路由；只保留 legacy compatibility 说明 |
| 旧系统残留 | `.trellis/*` 作为主路径 | 模板主路径统一为 `.dijiang/*`；`.trellis` 只出现在兼容说明 |
| 旧系统残留 | `muse_*` 与 `dj-muse` | workflow/skills 对外统一使用 `dijiang mem ...`；底层 MUSE 只作为 import/legacy scanner 细节 |
| 文档漂移 | README 写 `task create`、`mem save` | 对齐真实 CLI 或新增兼容 alias，二者必须二选一 |
| 生成重复 | `.pi/skills/dj-dj-*` | 增加检测和清理策略，防止 update/init 继续扩散重复目录 |
| 平台漂移 | Pi/Claude/Cursor/Codex/Hermes/OpenCode 各写一套 workflow | 平台模板只引用同一 canonical lifecycle 摘要 |

### 7A.6 分批迁移计划

**Batch A — 文档入口对齐**

更新 README、`crates/configurator/templates/config/agents.md`、`crates/configurator/templates/config/workflow.md`。目标是让对外文档只暴露 7A.2 到 7A.4 的模型，删除不存在的 `/dj-muse`、不存在的 `review` 状态、错误 CLI 命令和坏表格。

验证：`cargo test -p dijiang --test e2e` 中 init/update 相关快照或断言仍通过；人工检查生成的 `.dijiang/workflow.md` 与 `AGENTS.md` 不含 `dj-muse`、`review → dj-check`、`task create`、`mem save`。

**Batch B — wrapper / prompt / agent 模板对齐**

更新 `dijiang-start`、`dijiang-continue`、`dijiang-finish-work`、`.pi/prompts/*`、`.pi/agents/*` 与跨平台 agent 模板。目标是全部使用 `.dijiang/*` 主路径、`dijiang mem ...` 对外命令、canonical lifecycle，不再读 `.trellis/workflow.md` 或 `.trellis/spec/` 作为主路径。

验证：`dijiang init --platforms pi,codex,cursor,claude,opencode,hermes` 生成文件后，搜索模板产物中 legacy 主路径只出现在明确标注的 compatibility 段落。

**Batch C — skill taxonomy 对齐**

给 `dj-*` skills 补齐统一边界描述，重点处理 `dj-check`/`dj-review`、`dj-output`/`dj-write`、`dj-audit`/`dj-health`/`dj-debt`/`dj-pattern`、`dj-implement`/`dj-tdd`/`dj-hunt` 的选择规则。`dj-review` 的最终去留在本批次决定：保留为 lightweight preset、迁入 `dj-check`，或 deprecated。

验证：`dj-dispatch` 的路由表只引用存在的 skill；每个质量类 skill 都能回答“检查对象是什么、是否改代码、是否阻塞交付”。

**Batch D — CLI 命令语义对齐**

不急于改 CLI 行为。先修 help、README 和 workflow 对 CLI 的解释。如果需要行为改动，优先做兼容 alias 而非删除：例如 `task create` 是否映射到 `task start`，`dijiang review` 是否迁移到 `dijiang check --mode`。

验证：CLI e2e 覆盖 `start/task start/finish-work/task archive/mem/review` 的兼容路径，旧命令若 deprecated 必须输出明确替代命令。

**Batch E — 生成链路与重复产物清理**

明确 `PiConfigurator::write_skills` 负责 3 个 `dijiang-*` wrapper，`write_project_skills` 负责 `dj-*` core skills。修正注释、update report 和冲突检测；增加 `dj-dj-*` 重复目录检测，默认报告、不自动删除。

验证：新 init/update 不再生成 `dj-dj-*`；已有重复目录会在 `dijiang update` 或 `dijiang doctor` 中提示清理建议。

### 7A.7 完成标准

- README、AGENTS、workflow、prompt、agent、skill frontmatter 中的入口命名可以映射回 7A.2 的唯一层级。
- 没有文档把 `review` 描述成正式 task status。
- 没有模板把 `.trellis/*` 描述为 DiJiang 主路径。
- 没有模板引用不存在的 `/dj-muse`。
- CLI help、README、workflow 中列出的命令都真实存在，或明确标记为 deprecated alias。
- 每个 `dj-*` skill 都有明确的 category、primary use、non-goals。
- 所有平台模板使用相同的 lifecycle 摘要，只保留平台适配差异。



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

- **路径冲突**：历史 session 曾把 DiJiang 主路径描述为 `.trellis/tasks/`；当前代码与 7A canonical model 均以 `.dijiang/*` 为主路径，`.trellis/*` 仅作为 legacy compatibility fallback
- **Pi 0.80 API**：session 已确认使用 `@earendil-works/pi-coding-agent` factory function 模式，本计划不涉及 Pi 适配器代码（Pi 适配器深化不在本计划范围）
- **cargo build 不只是 cargo test**：session 已确认编辑 configurator 后必须 `cargo build -p dijiang` 而非仅 `cargo test`，本计划 §7 验证矩阵已包含
- **字段顺序契约**：Trellis 的 `TASK_RECORD_FIELD_ORDER`（TS 端）+ Python `TaskData`（Python 端）必须结构一致，本计划 §1.1 是把 DiJiang 也加入这个契约
- **新立场强化**：用户明确重申 DiJiang 必须保持完全独立，本计划 §0.1 已转化为三条设计硬约束 + §9 立场自检工具

---

> 计划完。本计划基于复核报告的 8 个事实修订，每个动作都有验证标准。优先级 P0 > P1 > P2 > P3，与复核报告的决策矩阵（§5）保持一致。核心立场（独立 Rust-native Agent Harness，零 Trellis runtime 依赖，兼容层可拆卸）已转化为 §0.1 的三条设计硬约束 + §9 PR 自检工具，确保未来不会被无意侵蚀。
