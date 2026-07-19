---
name: dj-health
description: >
  综合代码库健康检查：覆盖构建、测试、Git、依赖、lint、工作区和
  agent 配置。每当用户对项目状态不确定、或想确认修改没有引入
  问题时使用。触发词：健康检查、状态报告、有没有问题、检查一切正常吗、
  全面检查。
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 完整的代码库健康检查报告，覆盖 6+ 维度 |
| **Done when** | 全部检查项执行完成，输出 Pass/Fail 汇总 |
| **Evidence** | 各检查项的实际执行结果 |
| **Output** | 健康检查报告（维度总结 + 每个检查项 Pass/Fail/Warn + 修复建议） |

# Health: 综合代码库健康检查

只报告，不修复。每个检查项输出 Pass/Fail/Warn + 修复建议。

## 检查维度

### 1. 构建健康

```bash
# 增量检查（较快）
cargo check 2>&1

# 完整构建
cargo build 2>&1

# Release 构建
cargo build --release 2>&1
```

检查项：
- [ ] `cargo check` 无错误/警告
- [ ] `cargo build` 成功
- [ ] 没有过期的编译缓存（`cargo clean` 不需要吗？）
- [ ] 特性标志交叉编译正常（如适用多个 feature 组合）

### 2. 测试健康

```bash
# 全部测试
cargo test 2>&1 | tail -20

# 指定 crate 测试
cargo test -p <crate> 2>&1 | tail -10
```

检查项：
- [ ] 所有测试通过
- [ ] 已知的预失败测试已标记（如 `#[ignore]` 或已有 issue 跟踪）
- [ ] 测试覆盖率未明显下降（如可用 `cargo tarpaulin` 或 `cargo llvm-cov`）
- [ ] 集成测试（e2e）可运行

### 3. Git 健康

```bash
# 工作区状态
git status

# 未推送提交
git log --oneline @{upstream}..HEAD

# 过时分支（已合并到 main，未清理）
git branch --merged main | grep -v "main\|*"

# 大 diff 警告
git diff --stat
```

检查项：
- [ ] 工作区干净或改动可预期
- [ ] 没有未推送的提交堆积过多
- [ ] 没有大量已合并但未清理的本地分支
- [ ] 单次 diff 不超过 500 行（过大提示拆分 commit）

### 4. 依赖健康

```bash
# 检查过时依赖
cargo outdated 2>/dev/null || cargo install cargo-outdated

# 安全审计
cargo audit 2>/dev/null || cargo install cargo-audit

# 重复依赖检查
cargo tree -d 2>&1 | head -30
```

检查项：
- [ ] 没有已知的安全漏洞
- [ ] 依赖版本没有严重过时
- [ ] 没有不必要的重复依赖
- [ ] dev-dependencies 和正常依赖分开合理

### 5. Lint / 格式健康

```bash
# 格式检查
cargo fmt --check 2>&1

# Clippy（默认）
cargo clippy -- -D warnings 2>&1 | tail -30

# 如果项目有额外 lint 配置
cargo clippy --all-targets --all-features 2>&1 | tail -20
```

检查项：
- [ ] `cargo fmt --check` 通过
- [ ] `cargo clippy` 无警告（至少无 error）
- [ ] 项目自定义 lint 配置（clippy.toml / .clippy.toml）未退化

### 6. Agent 配置健康

```bash
# 配置目录
ls -la .dijiang/ 2>/dev/null || echo "MISSING .dijiang/"

# skill 列表
dijiang skills 2>&1

# 活跃 task 状态
dijiang status 2>&1

# 工作流状态
dijiang workflow-state --json 2>&1 | head -20
```

检查项：
- [ ] `.dijiang/` 目录存在且结构完整
- [ ] 所有必需 skill 在 `dijiang skills` 中列出（dj-grill、dj-implement、dj-check 等）
- [ ] 活跃 task 状态正常，task.json 可解析
- [ ] 配置（.dijiang/config.toml、workflow.md 等）文件完整

### 7. CI / 集成健康（如可用）

```bash
# Github Actions 状态
gh run list --limit 5 --json conclusion,headBranch,displayTitle 2>/dev/null
```

检查项：
- [ ] 最近 CI 运行全绿
- [ ] 当前分支的 CI 无失败

## 报告格式

```markdown
## 健康检查报告

### 构建 ✅/❌/⚠️
- cargo check: Pass — 无错误，N 个警告
- cargo build: Pass
- 建议: （如有需要记录的建议）

### 测试 ✅/❌/⚠️
- 单元测试: 25/25 Pass
- E2E 测试: 45/48 Pass（3 Known Failures: ...）
- 覆盖率: N/A
- 建议: ...

### Git ✅/❌/⚠️
- 工作区: 未提交更改 N 个文件
- 未推送: N 个 commit
- 建议: ...

### 依赖 ✅/❌/⚠️
- cargo audit: Pass
- cargo outdated: N 个可用更新
- 建议: ...

### Lint ✅/❌/⚠️
- cargo fmt --check: Pass
- cargo clippy: Pass
- 建议: ...

### 配置 ✅/❌/⚠️
- .dijiang/: 存在
- Skills: N/N 正常
- 建议: ...

**综合结论**: 健康 / 轻微关注 / 需处理
```

## 边界

- 不修改配置、代码或依赖
- 不安装缺少的工具——只建议安装
- 只报告，不修复

## Hard Rules

1. 不修改任何文件
2. 不安装缺少的工具——只标记缺失并给出安装命令
3. 6 个维度全部覆盖，不跳过
4. 每个维度先列出命令再检查，执行结果输出后再给判断
5. 报告格式统一：维度标题 + 项目列表 + 建议

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 只检查构建不检查测试 | 测试失败未发现 | 6 个维度必须全部覆盖 |
| 没看 CI 结果 | 远程构建失败本地不知道 | 能访问 gh CLI 时检查 |
| 只检查 main 忽略当前分支 | 分支可能有编译问题 | 切换或指定分支再检查 |
| 依赖检查命令不在 PATH | 直接跳过 | 提示用户安装再跑 |
