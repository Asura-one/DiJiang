---
name: dj-check
description: >
  交付质量闸门：验证 diff 质量、功能完整性、安全性和回归风险，输出 finish-work 收口证据。
  Use when the user needs a delivery quality gate, completion verification, release-blocking check, or finish-work readiness review.
  触发词：check、质量闸门、交付前检查、完成检查、收口检查、finish-work readiness、合并前检查。
---

# Check: 质量闸门

## 职责

`dj-check` 是 DiJiang 的 canonical quality gate。用于交付前验证 diff 质量、功能完整性、安全性、回归影响，并输出 finish-work 所需的收口证据。


## 检查职责

1. **代码质量审查** — 逻辑、风格、安全性
2. **功能完整性核对** — 对照 PRD/issue 逐项确认
3. **回归影响检查** — 搜索引用点，确认调用方同步更新
4. **过度工程检查** — dj-ponytail 视角：有没有可以删的；dj-karpathy 视角：代码是否最简
5. **多视角审查** — 高风险改动按 correctness、security、performance、architecture、docs 拆分检查
6. **finish-work handoff** — 只输出版本、验证、阻塞项和交接结论，不执行发布动作

## 原则

- **Simplicity First**（dj-karpathy）：审查时问自己"这段代码能更简单吗？"如果 200 行能缩到 50 行，标记为过度工程
- **Define Verifiable Success**（dj-karpathy）：每个功能是否有可验证的测试或检查点？没有则标记为缺失
- **阶梯决策**（dj-ponytail）：审查时检查依赖引入是否遵循阶梯（stdlib → 已有依赖 → 最少代码）
- **Source Fidelity**：重构、迁移和修 bug 时优先保持原系统事实，不能因为命名“不准确”就擅自改字段、界面文案或业务术语。遇到乱码或非 UTF-8 文件，先确认编码再下结论。
- **Memory Hygiene**：审查写入 memory 的内容时，检查 source、scope、confidence、freshness、conflict、actionability；没有行动价值的内容留在 task artifact。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | 当前 diff、PRD/issue/task 摘要、验证命令、受影响文件、git-safety 上下文 |
| 输出 | findings-first 质量报告、验证证据、风险结论、版本结论、finish-work handoff 数据 |
| 非目标 | 不在 `dj-check` 中修代码、commit、push、merge、打 tag 或删除 worktree |

## 工作流

### 1. 质量闸门约定

审查前先确认：

```text
范围：<working tree / staged / branch range>
需求来源：<PRD / issue / task / user request>
基准引用：<main / HEAD / other>
验证计划：<typecheck/test/lint/manual>
是否修改代码：no
是否 commit/push/merge：no
```

### 2. 读 diff

```bash
git status --short --branch
git diff main...HEAD --stat
git diff main...HEAD
git diff --cached --stat
git diff --cached
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

缺失或偏离的功能 → 标记为 blocking，并在结论中写明需要实现或排障后续；不要在 `dj-check` 内自行切换 skill。

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

### 3.5 对抗式安全审查

> 仅在变更触及权限、输入处理、外部调用、持久化、并发、依赖或安全边界时执行；普通文案/文档改动可写 `n/a`。

检查攻击面：
- Input validation：恶意输入是否能穿透边界？
- Injection：是否存在 SQL、command、XSS 或模板注入？
- Authentication / authorization：是否可能绕过身份或权限检查？
- Data exposure：日志、错误信息、返回值是否泄露 secret 或敏感数据？
- Denial of service：极端输入、资源耗尽、无限循环是否会拖垮系统？
- Supply chain：新增依赖、脚本、下载路径是否可信且可锁定？
- Race conditions / resource leaks：并发访问、清理失败、句柄泄漏是否会破坏状态？

发现问题时按格式输出：
```text
<file:line> [severity] <attack scenario>. fix: <required change>
```

### 4. 过度工程检查（dj-ponytail 视角）

```
L<行号>: <tag> <问题>. <替代>.  net: -<N> lines
```

标签：
- `delete:` 死代码、未使用扩展点。替代：无
- `stdlib:` 手写了标准库已有的东西。替代：指定函数名
- `native:` 平台原生功能就能做的。替代：指定功能
- `yagni:` 只有一个实现的抽象。替代：内联
- `shrink:` 同样逻辑更少行。替代：更短写法

结尾统计：`net: -<N> lines, -<M> deps possible.`

### 5. 收尾门禁（version/finish-work handoff）

审查通过后，必须给 `dijiang-finish-work` 明确结论和证据；`dj-check` 不执行 commit/push/merge/tag/worktree cleanup。

```bash
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
```

检查项：
- 当前目录必须是任务 worktree，不能是主 checkout。
- diff 只能包含当前任务相关修改，不能混入无关文件。
- 文档/spec/task artifact 已按实际行为同步，或明确说明无需更新。
- 验证命令和结果必须逐项列出；未运行的验证写 `not run` 和原因。
- 版本决策已给出：`major` / `minor` / `patch` / `none`。
- commit type/scope 候选已给出，message 写行为变化，不堆文件名。
- finish-work 阻塞项已列出：未修问题、未跑验证、权限/CI/remote 限制。

**版本号规范（语义化版本）：**
- Major：不兼容的 API 修改。
- Minor：向下兼容的功能性新增。
- Patch：向下兼容的问题修正。
- None：仅内部流程、测试、文档或未发布包的变化，不更新版本。

发布动作交给 `dijiang-finish-work`。如果用户要求在 check 阶段发布，停止并带着审查报告输出 finish-work 后续项。
## 输出格式

```text
## 审查报告
### Findings
[CRITICAL/HIGH/MEDIUM/LOW findings first, each with file:line and fix recommendation]

### 功能完整性
[逐项核对结果]

### 验证证据

Minimum validation evidence:

```text
Typecheck: <command or not run + reason> => <result>
Relevant tests: <command or not run + reason> => <result>
Full tests: <command or not run + reason> => <result>
Manual checks: <steps or n/a> => <result>
```

所有未运行项必须写 `not run` 和原因，不能暗示通过。

### 回归影响
[引用点 / sibling path / source fidelity result]

### 过度工程
[dj-ponytail 视角的发现]

### Finish-work 交接
Version: <major/minor/patch/none>
Commit type/scope: <type(scope)>
Blocking issues: <none or list>
Release actions: delegated to dijiang-finish-work
发布动作：交给 dijiang-finish-work

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
| diff 太大看不完 | 按模块分批审查，先看核心文件 | 只审 CRITICAL/HIGH，标注剩余风险 |
| 功能完整性核对发现缺失 | 标记缺失项，输出需要实现后续 | 记录为 blocking issue，不给通过结论 |
| 安全问题无法确定严重程度 | 按最高合理严重程度处理 | 标注 `待安全确认`，不合并 |
| 合并冲突 | 展示冲突文件，按 git-safety 处理 | 停止 check，输出需要排障或实现后续 |
| 审查标准和用户期望不一致 | 明确质量闸门优先级 | 功能完整性和安全性不能降级 |
| 回归检查发现引用点未更新 | 列出所有受影响的引用点 | 标记为 🔴 回归风险，必须修复才能通过 |
| 回归检查范围太大 | 按模块分批检查，先看核心文件 | 只检查直接引用，间接引用标注 `需人工确认` |
| 验证命令无法运行 | 记录命令、错误和原因 | 标注 `not run`，给出 residual risk，不说通过 |
## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不看 PRD 就审代码 | 先对照需求逐项核对 |
| 只看代码风格不管功能完整性 | 功能完整性是第一优先级 |
| 审查完自动合并 | 只给 finish-work handoff，发布动作另走 `dijiang-finish-work` |
| 发现问题直接修 | 先报告，再输出实现/排障后续项 |
| 忽略过度工程 | 用 dj-ponytail 视角扫一遍 |
| 不检查安全性 | 安全问题是硬伤 |
| 没跑验证却写成通过 | 写 `not run`、原因和 residual risk |
| 把缺失功能记到 issue 后放行 | 缺失需求是 blocking issue |
