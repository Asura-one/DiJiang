---
name: dj-audit
description: >
  全仓扫描：过度工程检查 + 安全性扫描。只报告，不修改。
  Use when the user wants to audit the codebase for over-engineering, bloat,
  security issues, or general code health.
  触发词：audit、审计、过度工程、安全扫描、代码体检、find bloat、全仓扫描。
---

# Audit: 全仓扫描

## 职责

扫描整个代码库，输出两份报告：
1. **过度工程报告** — 可以删的、可以简化的
2. **安全扫描报告** — 潜在的安全问题

只报告，不修改。一次性执行。

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | Audit scope, audit type, language/framework signals, dependency files, and user risk focus |
| 输出 | Sorted findings with evidence, severity/impact, suggested fix, and no code changes |
| 非目标 | Do not fix findings, refactor code, or turn audit into implementation |

## 工作流

1. **Confirm scope** → directory, whole repo, or named subsystem.
2. **Detect stack** → package managers, languages, frameworks, dependency manifests.
3. **Run over-engineering scan** → bloat, unused abstraction, duplicated wrappers, needless deps.
4. **Run security scan** → credentials, injection, permissions, dependency risk, sensitive logging.
5. **Rank findings** → severity for security; deletion/simplification impact for bloat.
6. **Report only** → include exact evidence and recommended follow-up type; do not start fixes inside audit.

## 过度工程扫描

### 扫描目标

- 标准库已有的功能被手写
- 平台原生功能被依赖替代
- 只有一个实现的抽象层
- 死代码、未使用的导出
- 可以内联的包装函数
- 没人设置的配置项

### 最小命令

```bash
git status --short --branch
find . -maxdepth 3 -type f \( -name 'package.json' -o -name 'Cargo.toml' -o -name 'go.mod' -o -name 'pyproject.toml' \)
grep -R "TODO\|FIXME\|deprecated\|unused\|eval\|exec\|password\|token\|secret" . --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules
```

Adapt commands to the stack. Use project-native tools when available, but keep the report evidence tied to file paths and lines.

### 输出格式

```text
OVER-ENGINEERING
<file>:<line> <tag> impact=<delete|simplify|replace>. <problem>. <replacement>.
```

Tags: `delete:` / `stdlib:` / `native:` / `yagni:` / `shrink:`

Sort by practical impact: deleted dependency, deleted file, deleted abstraction, deleted branch, then local line reduction.

End with: `net: -<N> lines, -<M> deps possible` when estimable; otherwise `net: not estimated`.

If the codebase is already lean: `Lean already. Ship.`

## 安全扫描

### 扫描目标

| 类别 | 检查项 |
|---|---|
| 凭证泄露 | .env 被 commit、硬编码 token/key/password、API key |
| 注入风险 | SQL 拼接、eval/exec、未转义的用户输入 |
| 依赖安全 | 已知漏洞的依赖版本、不必要的依赖 |
| 权限问题 | 过宽的文件权限、未验证的用户操作 |
| 信息泄露 | 错误信息暴露内部路径、调试模式未关闭 |
| 加密问题 | 弱算法、硬编码 IV/key、HTTP 传敏感数据 |
| 对抗式输入 | 超大输入、恶意文件、未来时间、乱码、空数据、重复事件 |
| 资源耗尽 | worker OOM 重试循环、缓存穿透、队列堆积、无限重试 |

### 全局对抗式审查

当用户要求全仓、架构、近期变更或上线前审计时，加入这轮反方视角：

```text
第一性原理：系统最核心的不变量是什么？哪些事实和边界必须成立？
攻击者视角：如果我要让系统崩溃、泄漏、污染数据或反复重试，会走哪条路径？
异常数据视角：未来时间、超大 HTML、损坏 JSON、重复 webhook、空响应会触发什么？
资源视角：内存、队列、缓存、文件句柄、网络超时是否会形成放大器？
维护者视角：哪些抽象、文档或依赖关系会让后续修复变成缝补？
```

输出仍然只报告，不修复。

### 输出格式

按严重程度排序：
```
[CRITICAL] <file>:<line> <问题>. <修复建议>.
[HIGH]     <file>:<line> <问题>. <修复建议>.
[MEDIUM]   <file>:<line> <问题>. <修复建议>.
[LOW]      <file>:<line> <问题>. <修复建议>.
```

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 扫描范围太大超时 | 缩小到 src/ 或核心目录 | 分批扫描，先扫最关键的 10 个文件 |
| 安全扫描工具不可用 | 用 grep 手动扫常见模式（硬编码密钥、eval、exec） | 只报告过度工程部分，安全扫描标注"未执行" |
| 扫描结果太多无法排序 | 按 tag 分类，每类取 top 5 | 只输出 CRITICAL + HIGH 级别 |
| 项目语言/框架不识别 | 跳过语言特定的检查项 | 用通用检查项（密钥、死代码、依赖） |

## 🔴 CHECKPOINT · 审计范围确认

扫描前先报告：

```text
Scope: <directory / whole repo / subsystem>
Audit type: <over-engineering / security / both>
Stack signals: <languages/frameworks/manifests>
Output limit: <top N findings per category>
Will modify code: no
```

🛑 STOP if the user asks for fixes during audit. Finish the report first, then output remediation follow-up type without switching skills inside `dj-audit`.

## 边界

- 只扫描，不修改代码
- 正确性 bug 和性能问题不在范围内（那是 `dj-hunt` 的事）
- 一次性报告，不持续监控
- 不读取或打印 secret 值；只报告疑似位置和 key 名

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 审计完直接改代码 | 只报告，用户决定修不修 |
| 把正确性 bug 也算过度工程 | 过度工程和 bug 是两回事 |
| 输出 secret 原文 | 只写 `[REDACTED]` 和文件位置 |
| 只查代码不查安全 | 过度工程 + 安全都要扫，除非用户限定范围 |
| 扫描结果不排序 | 按影响/严重程度排序 |
| 发现问题就创建大重构计划 | 给最小修复路径和后续工作类型 |
