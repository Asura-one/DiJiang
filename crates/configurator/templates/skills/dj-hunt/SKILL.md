---
name: dj-hunt
description: >
  系统化排查 bug：先定位根因，再修复。尤其擅长回归和"以前好现在坏"的情况。
  Use when the user reports errors, crashes, regressions, failing tests,
  or something that used to work and now fails.
  触发词：bug、报错、崩溃、不工作、以前好的、回归、debug、排查、出问题了、hunt。
---

# Hunt: 排查 Bug

## 职责

在修复之前找到根因。一个症状的 patch 往往在别处制造新 bug。

## 铁律

**在能用一句话说出根因之前，不要碰代码。**

> "我认为根因是 [X]，因为 [证据]。"

## 代码定位（找代码在哪里）

开始调查前，先找到症状对应的代码位置。以下策略按优先级排列。

### 1. 错误驱动的定位

利用错误消息本身找到代码：

```bash
# 1. 用错误消息精确搜索
grep -rn "错误消息内容" src/ --include="*.rs"

# 2. 如果错误消息不完全匹配，搜索关键词
grep -rn "partial\|keyword\|相关关键词" src/ --include="*.rs"

# 3. 搜索错误代码/编号
grep -rn "E1234\|ERR_BAD_RESPONSE" src/ --include="*.rs"

# 4. 如果有 stack trace：
#    - 第一行是异常抛出点 → 搜索类名/函数名
#    - 项目内部调用 → 从下往上看，第一个项目内部调用通常是起点
```

### 2. 语义搜索（当精确匹配不够时）

错误消息可能包含变量/动态内容，不完全匹配字符串：

```bash
# 1. 搜索包含该错误消息的函数/模块
grep -rn "error\|Error\|invalid\|failed\|unexpected" <可疑模块>/ --include="*.rs" | head -20

# 2. 搜索生成该错误消息的引用模式
#    很多错误由 format!()/concat!() 拼接而成——搜索各段
grep -rn "固定段" src/ --include="*.rs"

# 3. 如果错误包含变量名或类型名，直接搜
grep -rn "变量名\|类型名" src/ --include="*.rs"
```

### 3. 调用链追踪

从已知点（入口、API、UI 组件）沿调用路径找到 bug 点：

```bash
# 1. 从入口函数开始，追踪调用链
grep -rn "fn [a-z_]*入口名" src/ --include="*.rs"

# 2. 找出所有调用该函数的地方（找到调用者链）
grep -rn "可疑函数名(" src/ --include="*.rs"

# 3. 沿数据流方向：检查输入从哪里来、输出到哪里去
grep -rn "输入参数名\|输出变量名" src/ --include="*.rs"
```

调用链追查方向选择：
| 症状（知道结果） | 方向 | 目标 |
|-----------------|------|------|
| UI 显示错误 | 从 UI 组件向服务层回溯 | 找到数据源头和转换链
| API 返回错误 | 从路由/controller 向业务逻辑回溯 | 找到验证/处理链
| 数据不一致 | 从存储层向上游追踪 | 找到写入/读取的路径
| 测试失败 | 从测试用例向被测试代码追踪 | 找到断言对应实现

### 4. Git 历史定位

当代码库较大，不知道从哪里开始时：

```bash
# 1. 找最近修改过相关文件的 commit
git log --oneline --all -- <相关文件或目录> | head -10

# 2. 用 git log -S 找特定字符串的变更历史
git log -p --all -S '关键字' -- '*.rs' | head -60

# 3. 用 git log -G 找正则匹配的变更
git log -p --all -G '正则表达式' -- '*.rs' | head -60

# 4. 用 git blame 找某行是谁什么时候改的
git blame <文件> -L <行号>,<行号>

# 5. git bisect（已知好坏的二分查找）
git bisect start
git bisect bad   # 当前版本 bad
git bisect good <已知好的版本>
# 重复测试直到找到第一个坏的 commit
git bisect reset
```

### 5. 分层交叉引用

bug 的根因往往不在症状表现那一层：

```
症状层         搜索策略
───────         ────────
UI 层出错   →  先查 UI 层找到用户操作对应的调用
API 层出错  →  查 API handler，然后沿数据流进业务逻辑
服务层出错  →  查模型/状态管理代码
数据层出错  →  查数据库迁移/映射/序列化
```

每层间的映射关系：

```bash
# UI → API：搜索 API 端点字符串
grep -rn "/api/endpoint" src/ --include="*.rs"

# API → Service：搜索 handler 调用的 service 方法
grep -rn "可疑handler名" src/ --include="*.rs" | grep "service\|use_case\|domain"

# Service → Data：搜索 Repository/DAO 调用
grep -rn "repo\|db\|query" src/ --include="*.rs" | grep "可疑函数名"
```

### 6. 特征搜索

当没有错误消息只有描述时，用功能特征缩小范围：

```bash
# 1. 搜索功能相关的术语
grep -rn "功能名\|特性名\|模块名" src/ --include="*.rs" -i | head -20

# 2. 搜索相关类型和结构体
grep -rn "struct.*相关类型\|trait.*相关特性" src/ --include="*.rs"

# 3. 搜索测试中描述的功能
grep -rn "被测功能描述" tests/ --include="*.rs"

# 4. 搜索配置/路由/注册表中的引用
grep -rn "功能key\|feature_flag\|路由路径" src/ --include="*.rs"
```

### 7. 失败回退

| 方法跑完仍找不到 | 降级策略 |
|-----------------|---------|
| 代码库太大无从下手 | 用 find + wc + sort 按行数排序定位核心文件
| 跨语言/跨项目时 | 搜索日志/输出关键字定位入口
| 关键词太常见误报多 | 缩小范围到具体模块或类型
| 项目有多个入口 | 从 test 文件或 main.rs 开始追踪

> "我认为根因是 [X]，因为 [证据]。"

必须指出具体文件、函数、行号或条件。"状态管理有问题"不可测试。"`useUser` 在 `src/hooks/user.ts:42` 缺少 `userId` 依赖"可测试。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Symptom, reproduction signal, expected behavior, observed behavior, environment, and changed range if known |
| 输出 | Root cause with evidence, minimal fix, regression verification, sibling-path check, and learning/spec recommendation |
| 非目标 | Do not patch before root cause, hide failed hypotheses, or use destructive git commands as normal recovery |

## 工作流

### Phase 0：排障约定

改代码前先定义：

```text
Symptom: <what fails>
Expected: <what should happen>
Observed: <what happened>
Reproduction: <test/command/manual steps or unknown>
Search entry: <error/log/UI/API/test/git history>
Will modify code before root cause: no
```

If reproduction is unknown, Phase 1 is to build or approximate the feedback loop. Do not skip to a fix.

### Phase 1：构建反馈回路

**这是核心技能。** 有了精确的 pass/fail 信号，找到原因只是时间问题。

反馈回路的方式（按优先级）：
1. **失败测试** — 在能触达 bug 的任何层面
2. **curl / HTTP 脚本** — 对着开发服务器
3. **CLI 调用** — fixture 输入，对比已知正确的快照
4. **浏览器脚本** — Playwright/Puppeteer 驱动 UI
5. **重放捕获的 trace** — 存真实请求到磁盘，隔离重放
6. **最小 harness** — 启动系统子集，单函数调用
7. **属性/fuzz 循环** — 随机输入找失败模式
8. **二分 harness** — `git bisect run`
9. **人工可复核清单** — 当无法自动化时，列出输入、操作、期望输出和证据截图/日志

Record:

```text
Feedback loop: <command or manual checklist>
RED/Repro evidence: <exact failure, observed output, or screenshot/log reference>
GREEN command: <same command/checklist that must pass after fix>
Regression scope: <sibling paths, related callers, full/relevant tests>
False positive risk: <low/medium/high>
Exception: <none, or why the reproduction cannot be automated and what replaces it>

反馈回路必须先证明关键命题，再扩大覆盖面。不要一开始追求全量自动化；一个可靠的 CLI fixture、截图对比或日志断言，比“看起来应该对”的代码审查更有价值。

### Phase 2：最小化

- 从最小复现开始，逐步扩大
- 一次只改一个变量
- 每步记录输入、输出、期望

### Phase 3：假设 + 验证

**🔴 证据-推理分离原则**：先收集所有证据，再推理结论。不允许带着结论去找证据。

1. **取证阶段**（只收集，不判断）
   - 收集所有相关日志、错误信息、代码片段
   - 记录时间线：什么时候开始出错、什么时候正常
   - 记录环境：操作系统、依赖版本、配置差异
   - 记录复现步骤：怎么触发、触发条件是什么

2. **推理阶段**（基于证据下结论）
   - 提出假设（一句话）——必须基于收集到的证据
   - 对复杂或反复复发的问题做第一性原理追问：
     ```text
     问题本质：系统本来必须保证什么不变量？
     硬事实：哪些输入、状态、时序、依赖行为已被证据证明？
     隐藏假设：之前的解释是否只修表层症状？
     根因推导：从硬事实能否推出真正破坏不变量的位置？
     治本修复：什么改动能防止同类路径再次出错？
     ```
   - 设计验证方式
   - 执行验证
   - 假设成立 → 进入 Phase 4
   - 假设不成立 → 回到取证阶段补充证据
**🛑 禁止行为**：
- ❌ 带着结论去找证据（确认偏误）
- ❌ 推理阶段发现证据不足时回去补充新证据
- ❌ 只收集支持自己假设的证据

### Phase 4：修复 + 回归
1. **在 worktree 中修复（git-safety）**
   ```bash
   # 确认在 worktree 中
   pwd  # 应在 ../<项目名>-<分支名> 中
   git status --short --branch
   ```
2. **回滚操作（如需回滚）**
   ```bash
   # 步骤1：创建备份 tag
   git tag backup/$(date +%Y%m%d-%H%M%S) HEAD
   
   # 步骤2：优先用 git revert 创建可审计撤销提交
   git revert <commit-hash>
   
   # 步骤3：如果只是撤销本轮未提交改动，先列出文件并请求确认
   git diff --stat
   # 经用户明确确认后，只还原本轮触碰的文件
   ```
3. 修复前确认 **Code Task TDD Contract**：根因明确，RED/Repro evidence 已记录，GREEN command 已定义，Regression scope 已列出，Exception 为 `none` 或有可审查理由。
4. 确认反馈回路从红变绿：同一 RED/Repro 对应的 GREEN command 必须通过。
5. 检查是否有兄弟路径存在同类问题，并按 Regression scope 跑相关回归。
6. 写回归测试（如果还没有自动化覆盖，且不是已记录的 Exception）。
7. 分析 AI 为什么会写出这个 bug：源事实缺失、编码/乱码、术语误判、过度推断、上下文截断、提示词约束不足。
8. 选择沉淀位置：prompt/skill/spec/ADR/task artifact/memory。只有具备 source、scope、confidence、freshness、conflict、actionability 的经验才写入 memory。
9. 跑全量测试确认没有引入新问题。

"以前好现在坏"的特殊处理：
1. 找到最后一个正常的 commit：`git bisect`
2. 对比正常和异常的 diff
3. 确定引入变更的意图
4. 修复时保留原意图

## 截图回归专项

UI 相关的回归 bug：
1. 获取截图（用户提供或自动化截图）
2. 对比正常状态的截图
3. 定位视觉差异对应的代码变更
4. 修复时检查响应式和跨浏览器

## 缓存问题专项

"有时好有时坏"的常见原因：
1. 检查是否有 stale cache（过期缓存返回旧数据）
2. 检查 cache key 是否正确（命中了不该命中的缓存）
3. 检查 runtime boundary（前后端/微服务之间的状态不一致）
4. 清缓存重试，确认是否是缓存问题

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 构建不了反馈回路 | 降级到手动复现步骤 | 文字描述复现步骤，让用户手动验证 |
| git bisect 找不到引入 commit | 缩小范围，手动二分 | 用最近相关 commit 逐个测试 |
| 假设全部被否定 | 回到 Phase 1 重新构建回路 | 扩大搜索范围（换模块/换数据/换环境） |
| 修复引入了新 bug | 回滚本轮修复，保留证据 | 新建更小 worktree 或更小 patch 重来 |
| 无法复现（间歇性 bug） | 加日志/监控，等下次出现 | 代码审查找竞态条件/缓存问题并标注风险 |
| 用户要求先改再说 | 输出当前证据缺口 | 拒绝猜修，先建立最小反馈回路 |
## 🔴 CHECKPOINT · 根因确认

修复前必须确认：
```
根因：<一句话>
证据：<文件:行号 / 测试 / 日志>
修复方案：<一句话>
RED/Repro evidence: <修复前失败信号>
GREEN command: <修复后必须通过的命令或检查>
Regression scope: <相关回归范围>
Exception: <none 或无法自动化原因>

确认修复？(Y/n)
```

## 🛑 修复失败计数器（3次换思路）

**每次修复尝试后，如果反馈回路仍然红灯，计数器 +1。**

```
fix_attempts = 0

修复后跑反馈回路：
├── 绿灯 → 成功，进入回归检查
└── 红灯 → fix_attempts += 1
    ├── fix_attempts < 3 → 回到 Phase 3 换假设
    └── fix_attempts >= 3 → 🛑 STOP: 强制换思路
```

### 3次失败后必须执行（不能继续同一思路）

| 优先级 | 换思路方式 | 具体动作 |
|--------|-----------|---------|
| 1 | **质疑根因** | 你认为的根因可能不是真正的根因。回到 Phase 1 重新构建反馈回路，用不同的复现路径 |
| 2 | **扩大搜索范围** | 从单文件/单函数扩展到调用链上下游。检查依赖版本、环境变量、运行时配置 |
| 3 | **二分隔离** | `git bisect` 找引入 commit；或写最小独立测试排除项目其他部分干扰 |
| 4 | **换工具** | 换调试方式：加日志 → 用 debugger → 用 strace/dtrace → 看 core dump |
| 5 | **降级处理** | 如果 bug 不阻塞核心功能，标记为 `known-issue`，记录复现步骤和影响范围，先推进其他任务 |
| 6 | **向用户求助** | 描述已尝试的所有路径、每次的假设和结果、当前卡点。用户可能有你没有的上下文 |

### 计数器记录格式

在 progress.md 中记录每次尝试：
```
- [HUNT-ATTEMPT-1] 假设: X, 修复: 改了Y, 结果: 仍然失败, 原因: Z
- [HUNT-ATTEMPT-2] 假设: A, 修复: 改了B, 结果: 仍然失败, 原因: C
- [HUNT-ATTEMPT-3] 🛑 触发 break-loop 回顾
```

### 🛑 Break-Loop 回顾（3次失败后强制执行）

**3次失败不只是换思路——必须写根因分析，防止同类问题复发。**

#### Step 1: 分类根因

| 根因类型 | 特征 | 典型场景 |
|---------|------|---------|
| **错误假设** | 你以为的根因不是真正的根因 | 症状在 A，根因在 B |
| **信息不足** | 缺少关键上下文 | 环境差异、隐式依赖 |
| **方法不当** | 用错了调试工具或策略 | 日志不够、没用 debugger |
| **架构问题** | 根因在设计层面 | 耦合太紧、职责不清 |
| **间歇性** | 无法稳定复现 | 竞态条件、缓存、时序 |

#### Step 2: 写 Break-Loop 报告

```
## Break-Loop 报告

**Bug**: <一句话描述>
**尝试次数**: 3+
**根因分类**: <错误假设/信息不足/方法不当/架构问题/间歇性>

### 每次尝试回顾
| # | 假设 | 修复 | 结果 | 失败原因 |
|---|------|------|------|---------|
| 1 | ... | ... | 失败 | ... |
| 2 | ... | ... | 失败 | ... |
| 3 | ... | ... | 失败 | ... |

### 根因分析
<为什么前3次都失败？真正的根因是什么？>

### 预防措施
<怎么防止同类问题再次发生？需要更新哪些 spec/文档？>
```

#### Step 3: 写入 DiJiang memory + 沉淀到 spec

```bash
# 写入 DiJiang project learning
dijiang mem learn --lesson "[break-loop] <bug描述>: <根因与预防措施>"

# 需要防止复发时，输出 spec 更新后续项；不要在 dj-hunt 内切换到文档 skill
```

### 3.5 将 Break-Loop 发现晋升为 spec 合约

当 Break-Loop 发现需要沉淀为 spec 时，必须按以下合约格式输出，并标记 `.dijiang/spec/` 更新后续项：

```markdown
### 1. Scope / Trigger
- 触发条件：<什么场景下会触发此类 bug>

### 2. Signatures
- 涉及的函数签名、API 合约（如有变更）

### 3. Contracts
- 输入/输出约束：<字段名: 类型, 约束>

### 4. Validation & Error Matrix
| 条件 | 期望结果 |
|------|---------|
| <触发条件1> | <期望行为1> |

### 5. Good/Base/Bad Cases
- Good: <无 bug 的正确行为>
- Base: <典型正常输入>
- Bad: <复现本 bug 的输入>

### 6. Tests Required
- [ ] <测试描述> — 断言：<具体断言>

### 7. Wrong vs Correct
#### Wrong
<导致本 bug 的错误写法>
#### Correct
<修复后的正确写法>
```

### 写入路径

- 技术规范 → `.dijiang/spec/{layer}/`（backend / frontend / meta）
- 通用经验 → `~/.config/muse/strategic/memories.md`（跨项目复用）
- 思维检查表 → `.dijiang/spec/guides/`（防止复发）

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 看到报错直接改 | 先构建反馈回路 |
| "可能是 X 问题"就去改 | 必须有证据支持假设 |
| 修完不写回归测试 | 回归测试是修复的一部分 |
| 只修报告的那个点 | 检查兄弟路径是否有同类问题 |
| 在 main 上修 bug | 在 worktree 中修 |
| 把破坏性重置当常规恢复手段 | 优先 `git revert` 或只还原本轮触碰文件，且需明确确认 |
| 一次改多个可能原因 | 一次改一个，确认效果 |
| 修了3次还没好继续硬试 | 🛑 3次必须换思路，不能在同一死胡同里打转 |
