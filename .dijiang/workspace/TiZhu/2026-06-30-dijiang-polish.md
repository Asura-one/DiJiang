# 2026-06-30 DiJiang 完善 Session

## 完成内容

### 1. Channel 子命令（Agent Harness 核心）
- `dijiang channel spawn` - 创建 agent 通道
- `dijiang channel list` - 列出活跃通道
- `dijiang channel send` - 发送消息到通道
- `dijiang channel status` - 查看通道状态
- `dijiang channel execute` - 执行 agent（支持超时、实时输出）
- `dijiang channel execute-all` - 并行执行所有活跃通道
- `dijiang channel stop` - 停止通道

### 2. Review 命令
- `dijiang review --mode adversarial` - 对抗式安全审查（7 个攻击向量）
- `dijiang review --mode first-principles` - 第一性原理架构审查（6 步分析）

### 3. 内置 Tactics
- 6 个默认策略：cargo-test, typecheck, review-adversarial, review-first-principles, lint-fix, doc-update
- 初始化时自动填充

### 4. Agent 恢复
- 从 Trellis 历史恢复完整的 dijiang-check.md 和 dijiang-implement.md

### 5. Workflow 集成
- 添加 Phase 6: Review 到工作流模板
- 更新 CLI Commands 表格
- 更新 Skill Routing 表格

### 6. Tests
- 16 个 e2e 测试全部通过
- 覆盖 review、channel、mem 命令

### 7. Makefile
- build/release/install/uninstall/test/clean/fmt/check/ci 目标
- install 安装到 ~/.local/bin

### 8. 中文本地化
- 所有 CLI 命令描述改为中文
- 所有输出信息改为中文

### 9. Update 命令
- `dijiang update` - 更新已初始化项目的 dj-* 技能

## 关键决策
- Agent 通过 stdin 管道传递给 pi --print 执行
- 超时使用 try_wait + sleep 轮询实现
- 并行执行使用 std::thread::spawn
- 默认超时 300 秒

## 下一步
- 优化 channel execute（更智能的超时处理）
- 添加更多 agent 类型（audit, design, grill 等）
- 完善 progress 查看功能
