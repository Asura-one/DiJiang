---
name: dj-debt
description: >
  技术债评估、追踪和管理。聚合多种来源（`ponytail:` 标记、TODO/FIXME/HACK、
  过时依赖、死代码模式），生成按模块/严重度/年龄归类的债务台账。
  Use when asked about technical debt, cleanup priorities, what shortcuts
  were taken, or pre-release debt review.
  触发词：债务、技术债、debt、ponytail、标记、台账、清理、重构优先级。
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 技术债评估报告（多源聚合 + 优先级排序 + 历史趋势） |
| **Done when** | 所有债务源扫描完毕，生成结构化台账 |
| **Evidence** | grep 搜索结果 + cargo audit/ls 等命令输出 |
| **Output** | 结构化债务报告（总数、按类型/严重度/模块/年龄分布） |

# Debt: 技术债评估与追踪

只报告，不修复。不标记新的 debt。

## 债务来源

### 1. `ponytail:` 标记（显式债务）

```bash
grep -rn 'ponytail:' --include='*.rs' --include='*.py' --include='*.ts' \
  --include='*.go' --include='*.js' --include='*.sh' . 2>/dev/null
```

格式：`ponytail: <描述> # <高/中/低> <可选日期>`

### 2. TODO/FIXME/HACK/XXX 注释（隐式债务）

```bash
# 排除 vendor/、target/、node_modules/
grep -rn 'TODO\|FIXME\|HACK\|XXX\|WORKAROUND\|TEMP\|HACK' \
  --include='*.rs' --include='*.py' --include='*.ts' --include='*.go' \
  --include='*.js' . 2>/dev/null | grep -v '/target/' | grep -v '/node_modules/' \
  | grep -v '/vendor/'
```

注意区分：
- **TODO** — 未完成的功能（Medium）
- **FIXME** — 已知但未修复的 bug（High）
- **HACK/WORKAROUND** — 绕过而非解决（High）
- **XXX/TEMP** — 不稳定或临时方案（Medium-High）

### 3. 废弃/弃用代码

```bash
# Rust: 弃用属性
grep -rn '#\[deprecated\]' --include='*.rs' . 2>/dev/null

# Python: DeprecationWarning
grep -rn 'DeprecationWarning\|deprecated' --include='*.py' . 2>/dev/null

# 仓库中的 dead 文件（>6个月未修改）
find . -name '*.rs' -o -name '*.py' -o -name '*.ts' | xargs -I{} stat -f '%Sm' {} 2>/dev/null | sort | tail -20
```

### 4. 依赖债务

```bash
# 过时依赖
cargo outdated 2>/dev/null | head -40

# 安全漏洞
cargo audit 2>/dev/null | head -40

# 重复依赖
cargo tree -d 2>/dev/null | head -30
```

### 5. 测试债务

```bash
# 被忽略的测试
grep -rn '#\[ignore\]' --include='*.rs' . 2>/dev/null

# 空测试
grep -rn 'fn .*test.* {}' --include='*.rs' . 2>/dev/null
```

### 6. 构建债务

```bash
# 编译警告
cargo check 2>&1 | grep 'warning'

# 未使用的依赖
cargo +nightly udeps 2>/dev/null | head -30
```

## 台账格式

```markdown
## 技术债务台账
扫描日期: <日期>
总债务项: N

### 按类型
| 类型 | 数量 | 高严重度 | 中严重度 | 低严重度 |
|---|---|---|---|---|
| ponytail 显式标记 | N | N | N | N |
| TODO/FIXME/HACK | N | N | N | N |
| 弃用/废弃 | N | N | N | N |
| 依赖债务 | N | N | N | N |
| 测试债务 | N | N | N | N |
| 构建债务 | N | N | N | N |

### 按模块（Top-10 严重模块）
| 模块 | 债务数 | 最高严重度 | 最老债务 |
|---|---|---|---|
| crates/cli/ | N | High | 6个月 |
| crates/task/ | N | Medium | 3个月 |

### 按年龄
- 🟢 <1 个月: N 项
- 🟡 1-3 个月: N 项
- 🟠 3-6 个月: N 项
- 🔴 >6 个月: N 项

### 优先级债务（严重度高 + 年龄大）
1. file.rs:42 — `ponytail: 临时绕过，需重构 #高 2026-01`
2. ...
```

## 严重度规则

| 级别 | 定义 | 示例 |
|---|---|---|
| **高** | 导致功能缺失、性能问题或 bug | `FIXME: 此路径会导致 panic`、`ponytail: 临时绕过 #高` |
| **中** | 未来可能出问题，或降低可维护性 | `TODO: 添加错误处理`、未使用的 import |
| **低** | 代码风格、文档过时、微小优化 | `TODO: 重命名变量以获得更好可读性` |

## 边界

- 只报告，不修复
- 不标记新的 debt
- 不做代码分析（仅搜索模式）
- 不做自动升级依赖

## Hard Rules

1. 只搜索模式，不做代码分析
2. 只报告，不标记新 debt，不修复
3. 输出顺序：高严重度先于低严重度，老 debt 先于新 debt
4. 总览（分类/模块/年龄统计）在前，详细清单在后
5. 每个债务项标注严重度和来源类型

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 顺手修了旧 debt | 改了不该改的 | 只报告不修改 |
| 只搜 `ponytail:` 漏了 TODO/FIXME | 债务规模被低估 | 至少覆盖 3 种来源 |
| TODO 全当同等严重度 | 高优先级被淹没 | 区分类型（FIXME > TODO 等） |
| 不标年龄 | 不知道哪些是长期遗留 | 按年龄分组输出 |
