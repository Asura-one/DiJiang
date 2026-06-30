---
name: dj-check
description: >
  代码审查：检查 diff 质量、功能完整性、安全性，并按 git-safety 规范合并。
  Use when the user wants to review code, check changes before merge, or verify a task is complete.
  触发词：review、审查、检查代码、看看有没有问题、合并前检查、check。
---

# Check: 质量闸门

## 职责

`dj-check` 是 DiJiang 的 canonical quality gate。用于交付前验证 diff 质量、功能完整性、安全性、回归影响，并在需要发布/合并时遵守 git-safety 规范。

1. **代码质量审查** — 逻辑、风格、安全性
2. **功能完整性核对** — 对照 PRD/issue 逐项确认
3. **回归影响检查** — 搜索引用点，确认调用方同步更新
4. **过度工程检查** — dj-ponytail 视角：有没有可以删的；dj-karpathy 视角：代码是否最简
5. **多视角审查** — 高风险改动按 correctness、security、performance、architecture、docs 拆分检查
6. **合并流程** — 仅在用户明确要求发布/合并时执行 git-safety 流程

## 原则

- **Simplicity First**（dj-karpathy）：审查时问自己"这段代码能更简单吗？"如果 200 行能缩到 50 行，标记为过度工程
- **Define Verifiable Success**（dj-karpathy）：每个功能是否有可验证的测试或检查点？没有则标记为缺失
- **阶梯决策**（dj-ponytail）：审查时检查依赖引入是否遵循阶梯（stdlib → 已有依赖 → 最少代码）
- **Source Fidelity**：重构、迁移和修 bug 时优先保持原系统事实，不能因为命名“不准确”就擅自改字段、界面文案或业务术语。遇到乱码或非 UTF-8 文件，先确认编码再下结论。
- **Memory Hygiene**：审查写入 memory 的内容时，检查 source、scope、confidence、freshness、conflict、actionability；没有行动价值的内容留在 task artifact。

## 工作流

### 1. 读 diff

```bash
git diff main...HEAD --stat   # 概览
git diff main...HEAD          # 详情
```

### 2. 功能完整性核对

对照 PRD / issue / 需求描述，逐项检查：
```
功能完整性：
- [x] 功能 A — 已实现
- [x] 功能 B — 已实现
- [ ] 功能 C — 缺失 ← 标记
- [x] 功能 D — 已实现，但行为偏离设计 ← 标记
```

缺失或偏离的功能 → 调用 `hunt` 或回到实现阶段修复（`implement` 或 `dj-tdd`，取决于任务级别）。

### 2.5 回归检查（改A不坏B）

**LLM 改代码的高频问题：改了 A 文件导致 B 文件出问题。** 每次审查必须跑这步。

```bash
# 1. 找出本次改动涉及的所有文件
git diff main...HEAD --name-only

# 2. 对每个改动文件，搜索其他文件对它的引用
grep -r "被改的函数名/类名/变量名" --include="*.ts" --include="*.py" --include="*.js" . | grep -v node_modules | grep -v .git

# 3. 检查引用点是否受本次改动影响
# - 函数签名变了？→ 调用方是否已更新
# - 类型定义变了？→ 使用方是否已更新
# - 配置项变了？→ 读取方是否已更新
# - 接口返回值变了？→ 前端是否已更新
```

发现不一致 → 标记为 🔴 回归风险，必须修复后才能合并。

### 2.6 源事实核对（防 AI 自作主张）

AI 生成代码常见风险是把旧系统里的“不好命名”当成可修正对象，或在读取失败时用推测补全事实。涉及迁移、重构、文案、字段名、报表列、搜索项时必须核对源事实：

- 原界面/原 API/原数据库字段是什么，当前 diff 是否保持一致？
- 是否存在编码问题（GBK、Latin-1、损坏字符）导致源文案被误读？
- 如果改了业务术语，是否有 PRD、ADR 或用户确认支撑？
- 没有证据时，结论标记为「待澄清」，不能用“命名更合理”作为通过理由。

### 2.7 记忆质量核对

当 diff、task artifact 或 finish-work 准备写入 `dijiang mem findings` / `dijiang mem learn` 时，检查：

- source：这条记忆来自用户、代码、测试、事故复盘还是外部资料？
- scope：它适用于当前项目、某个 package、某类任务，还是所有 DiJiang 项目？
- confidence：它是已验证事实、用户偏好、推断，还是待验证假设？
- freshness：什么时候应该重看、过期或删除？
- conflict：是否和现有 spec、ADR、代码、任务记录或 memory 冲突？
- actionability：未来 agent 会因此改变哪个决策、检查或执行路径？

不满足 actionability 的内容不得进入 durable memory。

### 3. 代码质量审查

逐文件检查：
- 逻辑正确性
- 错误处理是否完整
- 边界条件是否覆盖
- 命名是否清晰
- 是否有安全隐患（注入、越权、信息泄露）

### 4. 过度工程检查（dj-ponytail 视角）

```
L<行号>: <tag> <问题>. <替代>.  net: -<N> lines
```

标签：
- `delete:` 死代码、未使用的灵活性。替代：无
- `stdlib:` 手写了标准库已有的东西。替代：指定函数名
- `native:` 平台原生功能就能做的。替代：指定功能
- `yagni:` 只有一个实现的抽象。替代：内联
- `shrink:` 同样逻辑更少行。替代：更短写法

结尾统计：`net: -<N> lines, -<M> deps possible.`

### 5. 收尾门禁（version/commit/push/merge）

审查通过后，必须给 `dijiang-finish-work` 明确结论：

```bash
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
```

检查项：
- 当前目录必须是任务 worktree，不能是主 checkout。
- diff 只能包含当前任务相关修改，不能混入无关文件。
- 文档/spec/task artifact 已按实际行为同步，或明确说明无需更新。
- 版本决策已给出：`major` / `minor` / `patch` / `none`。
- commit type/scope 已给出，message 写行为变化，不堆文件名。

**版本号规范（语义化版本）：**
- Major：不兼容的 API 修改。
- Minor：向下兼容的功能性新增。
- Patch：向下兼容的问题修正。
- None：仅内部流程、测试、文档或未发布包的变化，不更新版本。

push/merge 规则：如果 remote、权限、CI 状态允许，任务完成后应 push 任务分支，合并到主分支，push 主分支和 tag，然后删除任务 worktree。任何一步不可执行时，报告具体阻塞并保留 worktree。
## 输出格式

```
## 审查报告

### 功能完整性
[逐项核对结果]

### 代码质量
[发现列表，按严重程度排序]

### 过度工程
[dj-ponytail 视角的发现]

### 结论
[通过 / 需要修改 / 缺失功能 / 待澄清]
```

### 「待澄清」是合法结论

**当证据不足时，显式标记为「待澄清」，不硬给结论。**

| 结论类型 | 条件 | 示例 |
|---------|------|------|
| ✅ 通过 | 代码质量好，功能完整 | "审查通过，无问题" |
| ⚠️ 需要修改 | 发现明确问题 | "第 42 行有逻辑错误" |
| ❌ 缺失功能 | PRD 中的功能未实现 | "功能 C 未实现" |
| ❓ 待澄清 | 证据不足，无法判断 | "这段代码的意图不明确，需要作者解释" |

**🛑 禁止行为**：
- ❌ 不确定时硬给「通过」结论
- ❌ 不确定时硬给「需要修改」结论
- ❌ 只看代码风格就下结论（需要看功能完整性）

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| diff 太大看不完 | 按模块分批审查，先看核心文件 | 只审 CRITICAL 级别的变更 |
| 功能完整性核对发现缺失 | 标记缺失项，回到实现阶段补充（`implement` 或 `dj-tdd`） | 记录到 issue，不阻塞合并 |
| 安全问题无法确定严重程度 | 按最高严重程度处理 | 标注"待安全团队确认" |
| 合并冲突 | 展示冲突文件，按 git-safety 处理 | `git merge --abort`，让用户手动解决 |
| 审查标准和用户期望不一致 | 问用户优先级（功能/质量/速度） | 先保证功能完整性，再看质量 |
| 回归检查发现引用点未更新 | 列出所有受影响的引用点，逐一修复 | 标记为 🔴 回归风险，必须修复才能合并 |
| 回归检查范围太大（改动文件太多） | 按模块分批检查，先看核心文件 | 只检查直接引用，间接引用标注"需人工确认" |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不看 PRD 就审代码 | 先对照需求逐项核对 |
| 只看代码风格不管功能完整性 | 功能完整性是第一优先级 |
| 审查完自动合并 | 必须用户确认 |
| 发现问题直接修 | 先报告，再决定谁修 |
| 忽略过度工程 | 用 dj-ponytail 视角扫一遍 |
| 不检查安全性 | 安全问题是硬伤 |
