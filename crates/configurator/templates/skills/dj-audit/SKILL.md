---
name: dj-audit
description: >
  全仓扫描：过度工程检查 + 安全性扫描。只报告，不修改。
  Use when the user wants to audit the codebase for over-engineering, bloat, dead code, or security issues.
  触发词：审计、扫一下、过度工程、安全扫描、audit、代码大检查。
summary: 全仓扫描：过度工程检查 + 安全性扫描。只报告，不修改。
phases: [check, finish]
risk: low
  全仓扫描：过度工程检查 + 安全性扫描。只报告，不修改。
  Use when the user wants to audit the codebase for over-engineering, bloat, dead code, or security issues.
  触发词：审计、扫一下、过度工程、安全扫描、audit、代码大检查。
---

参考规范：`.dijiang/references/decision-ladder.md`（扫描时评估代码必要性）。

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 全仓扫描报告（过度工程 + 安全扫描）|
| **Done when** | 全部 git 文件扫描完成，报告输出 |
| **Evidence** | 扫描日志、grep 结果 |
| **Output** | 结构化审计报告（可删除内容列表 + 安全发现 + 每项理由） |

# Audit: 全仓扫描

扫描代码库的过度工程和安全问题。只报告，不修改。

## 工作流

### 1. 过度工程扫描

检查这些信号：
- **死代码** — 从未被引用的导出、不会触发的条件分支
- **过早抽象** — 只有一个调用方的接口/工厂/策略模式
- **过度配置** — 永远只会用默认值的配置项
- **冗余层面** — 可以直接调用的中间层
- **过大文件** — 300 行以上的单一文件

### 2. 安全扫描

搜索这些模式：
- 敏感信息硬编码（密码、token、API key）
- 未校验的外部输入（SQL 拼接、shell 命令拼接）
- 危险的导入（eval、exec、pickle 不安全使用）

### 3. 输出

每项发现包含：
```text
文件：<路径:行号>
类型：<过度工程/安全>
严重度：<高/中/低>
描述：<问题说明>
建议：<修复方向>
```

## 🟢 审计范围确认

```text
扫描目标：<整个代码库 / 指定模块 / 指定类型>
排除：<第三方库 / 生成代码 / 测试代码>
```

参考 `references/HTML-REPORT.md` 查看审计报告 HTML 模板。

## 边界

- 只检查已追踪的文件（git tracked）
- 不检查第三方依赖
- 对 false positive 标注为"疑似"而非"确认"

参考规范：`.dijiang/references/anti-patterns.md`（跨技能行为约束）。

## Hard Rules

1. 只检查 git tracked 文件，不碰第三方依赖
2. 过度工程信号和安全隐患分两个阶段独立扫描
3. false positive 必须标注"疑似"，不能标"确认"
4. 每项发现必须包含：文件位置、类型、严重度、描述、建议

## Gotchas

| Gotcha | 后果 | 预防 |
|---|---|---|
| 把第三方依赖也扫进去 | 报告噪音太大 | 排除 vendor/node_modules |
| 报了问题不给建议方向 | 用户不知道怎么做 | 每项发现给修复方向 |
| 过度工程和安全一起扫 | 报告混杂，难以决策 | 分两个阶段独立扫描 |
