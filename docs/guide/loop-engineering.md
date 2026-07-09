# DiJiang Loop Engineering 使用指南

> 适用版本：dijiang >= 0.6.3
> 更新日期：2026-07-08

Loop Engineering 是将 AI 编程代理的使用模式从"手动写提示"升级为"设计自动化循环"的工程方法。DiJiang 实现了 loop-engineering 的核心能力，让项目可以自动评估自动化成熟度、记录工作模式、估算成本，并通过 MCP 协议与外部工具集成。

---

## 核心概念

### 循环（Loop）

循环是指代理围绕一个目标反复执行"发现→分诊→修复→验证→通知"的过程。DiJiang 通过以下机制管理循环：

- **断路器（Circuit Breaker）**：自动检测代理是否陷入死循环（同一错误重复 N 次、连续失败 N 次、Token 超预算、迭代超上限），触发时在 `workflow-state` 中输出 STOP 信号
- **记忆回写（Memory Writeback）**：每次 `finish-work` 将循环信号写回记忆系统
- **工作流状态（Workflow State）**：每次注入包含 `loop_state`（goal、mode、progress、retry）和 `circuit_breaker_status`

### L1 → L2 → L3 成熟度模型

| 等级 | 分数 | 行为 | 说明 |
|------|------|------|------|
| L0 | 0–37 | 未就绪 | 项目尚未配置任何循环自动化 |
| L1 | 38–57 | 只报告 | 可输出审计报告，但不做任何修改 |
| L2 | 58–77 | 辅助操作 | 小范围自动修复 + 验证者 |
| L3 | 78+ | 无人值守 | 全自动运行（需 budget + denylist + run log + human gates） |

---

## 快速开始

### 1. 运行循环就绪审计

```bash
# 在项目根目录执行
dijiang audit

# 示例输出：
# ═══════════════════════════════════════
#   DiJiang Loop Readiness Audit
# ═══════════════════════════════════════
#   总分: 72/100
#   等级: L2
# ───────────────────────────────────────
#   信号 (8/13 项检测):
#     .dijiang/: 10/10 (100%)
#     active_task: 0/18 (0%)
#     route_gate: 14/14 (100%)
#     ...
# ───────────────────────────────────────
#   L0 (0-37): 未就绪  | L1 (38-57): 只报告
#   L2 (58-77): 辅助操作 | L3 (78+): 无人值守

# 查看改进建议
dijiang audit --suggest

# 生成 shields.io badge URL
dijiang audit --badge
```

#### 评分信号说明

| 信号 | 权重 | 检测内容 | 如何提升 |
|------|------|---------|---------|
| `.dijiang/` 目录 | 10 | 项目已初始化 | `dijiang init` |
| active task | 18 | 存在活跃任务 | `dijiang start <name>` |
| route gate | 14 | route_gate 已编译 | 内置，无需操作 |
| git gate | 14 | git_gate 已编译 | 内置，无需操作 |
| 2+ dj-* skills | 14 | 至少 2 个技能 | 安装 dj-grill、dj-check 等 |
| verifier skill | 14 | 存在 dj-check | 安装 dj-check |
| AGENTS.md | 9 | 项目根目录存在 | 创建 AGENTS.md |
| workflow.md | 9 | `.dijiang/` 下存在 | 内置模板 |
| tactic 记录 | 6 | 全局记忆有策略 | `dijiang mem tactic` |
| pattern 记录 | 6 | 项目记忆有模式 | `dijiang mem pattern` |
| circuit breaker | 6 | 断路器已实现 | 内置，无需操作 |
| run log | 3 | 存在 `loop-run-log.json` | 由自动化生成 |
| budget | 3 | 存在 budget 文档 | 创建 budget.md |

### 2. 注册工作模式

```bash
# 基本模式（带 metadata）
dijiang mem pattern \
  --name "ci-sweeper" \
  --description "监控 CI 构建失败并自动分诊" \
  --tags "ci,build,test" \
  --cadence "15m" \
  --risk "medium" \
  --week_one_mode "L2" \
  --token_cost "high" \
  --phases "discover,triage,fix,verify,notify" \
  --human_gates "生产环境部署" \
  --steps "检查 CI 状态,分析失败原因,创建修复 PR,验证通过后通知"

# 查看所有模式
dijiang mem patterns

# 导出为 JSON 注册表
dijiang mem recommend --registry
```

#### Pattern 元数据字段

| 字段 | 示例 | 说明 |
|------|------|------|
| `--cadence` | `"15m"`, `"1h"`, `"6h"`, `"1d"` | 运行节奏 |
| `--risk` | `"low"`, `"medium"`, `"high"` | 风险等级 |
| `--week_one_mode` | `"L1"`, `"L2"` | 首周推出模式 |
| `--token_cost` | `"low"`, `"medium"`, `"high"` | 预估 Token 消耗 |
| `--phases` | `"discover,triage,fix,verify,notify"` | 包含的阶段 |
| `--human_gates` | `"生产部署审批"` | 需要人工介入的环节 |

### 3. 估算 Token 成本

```bash
# 估算所有模式的每日开销（默认 L1）
dijiang cost

# 估算指定模式的 L3 开销
dijiang cost --pattern ci-sweeper --level L3

# 示例输出：
#   Token 成本估算 (等级: L2):
# ──────────────────────────────────────────
#   ci-sweeper:
#     节奏: 15m | 成本: high | 次/天: 96.0
#     单次: 240000 tok | 每日: 23040000 tok | 每月: 691200000 tok
```

#### 成本模型

| token_cost | 单次 Token | 说明 |
|-----------|-----------|------|
| `low` | 5,000 | 轻量检查（如 changelog 草稿） |
| `medium` | 20,000 | 常规分诊（如 daily triage） |
| `high` | 80,000 | 深度分析（如 CI sweeper / PR review） |

级别倍率：L1 = 1x, L2 = 3x, L3 = 5x

#### 节奏换算

| cadence | 次/天 |
|---------|-------|
| `5m` / `15m` | 96 |
| `30m` | 48 |
| `1h` | 24 |
| `2h` | 12 |
| `6h` | 4 |
| `12h` | 2 |
| `1d` | 1 |

### 4. 推荐模式

```bash
# 根据用例推荐
dijiang mem recommend --use-case "watch CI"

# 输出格式：
#   推荐模式 (查询: "watch CI"):
#     ci-sweeper (匹配度: 80%)
#       监控 CI 构建失败并自动分诊
#       节奏: 15m
#       风险: medium

# 其他用例示例：
dijiang mem recommend --use-case "monitor PR"
dijiang mem recommend --use-case "daily triage"
dijiang mem recommend --use-case "dependency check"

# 查看完整注册表
dijiang mem recommend --registry
```

### 5. 启动 MCP 服务器

MCP（Model Context Protocol）服务器让外部工具（Claude Desktop、VS Code 等）按需查询 DiJiang 的工作流状态、模式和策略。

```bash
# 启动 MCP 服务器（stdio 模式）
dijiang mcp

# 服务器监听 stdin，接收 JSON-RPC 2.0 请求
# 可用的 resources：
#   dijiang://workflow-state  — 当前工作流状态
#   dijiang://patterns        — 所有模式注册表
#   dijiang://tactics         — 所有贝叶斯策略
#   dijiang://audit           — 循环就绪审计报告

# 可用的 tools：
#   list_patterns     — 列出所有模式
#   get_pattern       — 按名称获取模式详情
#   recommend_pattern — 按用例推荐模式
#   estimate_cost     — 估算 Token 成本
#   run_audit         — 运行审计
```

#### MCP 客户端配置示例（Claude Desktop）

```json
{
  "mcpServers": {
    "dijiang": {
      "command": "dijiang",
      "args": ["mcp"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

### 6. 理解断路器自动行为

断路器在后台自动运行，无需手动触发。当代理循环出现以下情况时会自动注入 STOP 信号：

```bash
# 查看断路器状态
dijiang workflow-state

# 输出中包含：
#   Circuit Breaker：STOP — stagnation（相同错误连续3次）
#   或
#   Circuit Breaker：CONTINUE — 循环健康

# 或通过 JSON 查看
dijiang workflow-state --json | jq '.workflowState.circuitBreakerStatus'
```

#### 断路器触发条件

| 触发条件 | 说明 | 阈值 |
|---------|------|------|
| stagnation（停滞） | 同一错误签名连续出现 | 默认 3 次 |
| no-progress（无进展） | 连续失败无成功 | 默认 5 次 |
| token-budget（超预算） | 累计 Token 超过预算 | 可配置 |
| max-iterations（超上限） | 迭代次数超过上限 | 默认 20 次 |

---

## MCP 集成场景

### 场景一：Claude Desktop 集成

配置 `claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "dijiang": {
      "command": "/usr/local/bin/dijiang",
      "args": ["mcp"],
      "cwd": "/Users/me/Project"
    }
  }
}
```

启动后 Claude 可直接查询：
- "当前项目的 loop readiness 评分是多少？"
- "查看注册的模式列表"
- "推荐适合监控 CI 的模式"

### 场景二：VS Code 扩展

通过支持 MCP 的 VS Code 扩展连接 `dijiang mcp`，在编辑器中直接查看项目工作流状态。

---

## 完整工作流示例

```bash
# 1. 审计当前项目成熟度
dijiang audit

# 2. 根据建议注册工作模式
dijiang mem pattern --name "daily-triage" --cadence "1d" --risk "low" --phases "triage,notify"

# 3. 估算成本
dijiang cost --level L2

# 4. 启动 MCP 服务器供外部工具集成
dijiang mcp

# 5. 日常使用：查看工作流状态
dijiang workflow-state
```

---

## 常见问题

**Q: 审计分数很低怎么办？**
A: 运行 `dijiang audit --suggest` 查看具体改进建议。通常从创建 AGENTS.md、安装 dj-grill/dj-check 技能、记录第一个 tactic 开始。

**Q: Token 成本估算准吗？**
A: 估算基于 cadence + token_cost + level 的线性模型，实际消耗因模型、prompt 长度而异。用于预算规划参考而非精确计费。

**Q: MCP 服务器需要一直运行吗？**
A: 是，`dijiang mcp` 是一个长时间运行的进程。建议在需要 MCP 集成时启动，用完关闭。可在终端或后台进程管理器中运行。

**Q: 断路器怎么配置阈值？**
A: 当前使用默认值（stagnation=3, no-progress=5, max-iterations=20）。可通过配置 `CircuitBreakerConfig` 在代码层面调整。
