---
name: check
description: >
  代码审查：检查 diff 质量、功能完整性、安全性，并按 git-safety 规范合并。
  Use when the user wants to review code, check changes before merge, or verify a task is complete.
  触发词：review、审查、检查代码、看看有没有问题、合并前检查、check。
---

# Check: 代码审查

## 职责

1. **代码质量审查** — 逻辑、风格、安全性
2. **功能完整性核对** — 对照 PRD/issue 逐项确认
3. **过度工程检查** — dj-ponytail 视角：有没有可以删的；dj-karpathy 视角：代码是否最简
4. **合并流程** — 遵守 git-safety 合并规范

## 原则

- **Simplicity First**（dj-karpathy）：审查时问自己"这段代码能更简单吗？"如果 200 行能缩到 50 行，标记为过度工程
- **Define Verifiable Success**（dj-karpathy）：每个功能是否有可验证的测试或检查点？没有则标记为缺失
- **阶梯决策**（dj-ponytail）：审查时检查依赖引入是否遵循阶梯（stdlib → 已有依赖 → 最少代码）

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

缺失或偏离的功能 → 调用 `hunt` 或回到 `implement` 修复。

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

### 5. 发布跟进（release/publish/push）

审查通过后，如果用户要求发布：
```bash
# 确认当前状态
git status --short --branch
git rev-parse HEAD

# 发布流程
git checkout main && git merge --no-ff <分支名>
git tag v<版本号>
git push origin main --tags
```

发布后检查项：
- CI 是否通过
- 包管理器版本是否同步
- issue 是否需要关闭
- changelog 是否更新

**版本号规范（语义化版本）：**
- 格式：`Major.Minor.Revision`
- 递增规则：
  - Major：不兼容的 API 修改
  - Minor：向下兼容的功能性新增
  - Revision：向下兼容的问题修正
### 6. 合并

```
🔴 CHECKPOINT · 合并确认
```

审查通过后：
1. **创建备份 tag**（合并前必须执行）
   ```bash
   git tag backup/$(date +%Y%m%d-%H%M%S) HEAD
   ```
2. 展示变更摘要（文件数、commit 数、关键改动）
3. 询问用户：
   ```
   审查通过。变更摘要：
   - 分支：<分支名>
   - 提交数：N
   - 变更文件：M
   - 备份 tag：backup/xxx
   
   请确认：
   [1] 合并到主分支
   [2] 暂不合并，我先测试
   [3] 需要修改
   ```
4. 用户选 [1] 后：
   ```bash
   git checkout main && git merge --no-ff <分支名>
   ```
5. 用户说"合并吧"等明确指令 → 直接合并，跳过确认对话框
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
[通过 / 需要修改 / 缺失功能]
```

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| diff 太大看不完 | 按模块分批审查，先看核心文件 | 只审 CRITICAL 级别的变更 |
| 功能完整性核对发现缺失 | 标记缺失项，回到 implement 补充 | 记录到 issue，不阻塞合并 |
| 安全问题无法确定严重程度 | 按最高严重程度处理 | 标注"待安全团队确认" |
| 合并冲突 | 展示冲突文件，按 git-safety 处理 | `git merge --abort`，让用户手动解决 |
| 审查标准和用户期望不一致 | 问用户优先级（功能/质量/速度） | 先保证功能完整性，再看质量 |

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 不看 PRD 就审代码 | 先对照需求逐项核对 |
| 只看代码风格不管功能完整性 | 功能完整性是第一优先级 |
| 审查完自动合并 | 必须用户确认 |
| 发现问题直接修 | 先报告，再决定谁修 |
| 忽略过度工程 | 用 dj-ponytail 视角扫一遍 |
| 不检查安全性 | 安全问题是硬伤 |
