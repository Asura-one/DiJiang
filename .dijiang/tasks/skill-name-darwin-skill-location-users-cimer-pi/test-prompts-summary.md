# Skill Test Prompts

Generated: 2026-06-30T16:51:10+00:00

## Review Scope
- Target: `crates/configurator/templates/skills/*/SKILL.md`
- Count: 22 skills
- Runtime neutrality scan: passed, no red-light match
- Note: per-skill `test-prompts.json` files were generated for local Darwin validation but are ignored by `.gitignore`; this summary is the tracked review artifact.

## dijiang-continue
- P1: 继续上次暂停的 DiJiang 任务，恢复上下文并告诉我下一步该做什么。
  - Expected: 读取 active task、任务产物和 workflow 状态，给出恢复后的执行路径。
- P2: 当前任务 paused，但我不确定之前做到哪里了，请恢复并检查是否可以继续实现。
  - Expected: 识别 paused 状态，加载任务上下文，输出可验证的继续计划和阻塞项。

## dijiang-finish-work
- P1: 完成当前任务，整理验证结果并准备收尾。
  - Expected: 检查 diff、验证命令、版本决策和任务状态，生成 finish-work 所需记录。
- P2: 这个任务没有代码改动，只清理了 DiJiang 状态，请按规范结束任务。
  - Expected: 识别 none 版本决策，记录验证说明，归档任务且不制造无意义 commit。

## dijiang-start
- P1: 开始一个新任务：修复用户登录偶发失败。
  - Expected: 创建或激活 DiJiang task，读取 workflow，路由到合适 skill 并定义下一步。
- P2: 我想做一个新功能，但需求还不清楚，帮我启动任务。
  - Expected: 创建任务并把状态导向 planning/grill，而不是直接实现。

## dj-audit
- P1: 审计整个代码库的安全和架构风险，给出优先级。
  - Expected: 按模块扫描风险面，输出分级发现、证据和修复建议，不直接改代码。
- P2: 帮我看这个仓库有没有长期技术风险，重点是配置和任务状态管理。
  - Expected: 限定范围审计，引用具体文件和风险，不泛泛而谈。

## dj-check
- P1: 检查当前 diff 是否可以交付。
  - Expected: 读取 diff，对照需求做功能完整性、回归、安全和过度工程检查。
- P2: 合并前帮我 review 一下这次修改有没有问题。
  - Expected: 输出通过/需修改/待澄清结论，并列出阻塞级问题。

## dj-debt
- P1: 评估这个项目的技术债，并按影响排序。
  - Expected: 识别债务类型、证据、影响、偿还建议和优先级。
- P2: 帮我判断当前模块是不是过度复杂，哪些地方值得重构。
  - Expected: 给出可执行的债务清单，不把普通风格偏好当债务。

## dj-design
- P1: 为这个管理后台页面设计交互和视觉方案。
  - Expected: 明确用户目标、信息架构、状态和组件方案，必要时产出可实现的 UI 指引。
- P2: 现有页面太乱，帮我重新设计但保持功能不变。
  - Expected: 先盘点现有功能和约束，再给出保守改版方案。

## dj-dispatch
- P1: 用户说：支付页偶发 500，昨天还是好的。请分流。
  - Expected: 识别为排查/回归类任务，推荐 dj-hunt 路径和下一步上下文读取。
- P2: 用户说：新增一个导出 CSV 的按钮。请分流。
  - Expected: 识别实现类任务，按复杂度选择 grill 或 implement 路径。

## dj-grill
- P1: 我要做团队权限功能，但只知道大概方向，请帮我对齐需求。
  - Expected: 提出少量关键问题，附推荐答案，收敛 PRD 范围。
- P2: 这个功能有三种实现方案，我不确定选哪个。
  - Expected: 澄清目标和约束，用取舍表推进决策，不急于写代码。

## dj-handoff
- P1: 把当前任务交接给下一个 agent。
  - Expected: 整理目标、状态、已改文件、验证、风险和下一步，不遗漏上下文。
- P2: 我需要暂停工作，生成一份恢复时能直接用的交接。
  - Expected: 输出可恢复的上下文快照和明确 pending 项。

## dj-health
- P1: 给这个仓库做一次健康度报告。
  - Expected: 覆盖构建、测试、架构、文档、依赖和工作流健康度，给出证据。
- P2: 我想知道当前项目最影响交付质量的问题是什么。
  - Expected: 聚焦高影响问题，排序并说明验证依据。

## dj-hunt
- P1: 测试突然失败：expected active task but got none，帮我排查根因。
  - Expected: 先建立复现/反馈回路，定位根因证据，再提出最小修复。
- P2: 用户反馈以前能导入模板，现在报错，修一下。
  - Expected: 按回归排查，搜索错误和历史变更，确认根因后再改。

## dj-implement
- P1: 按已有 PRD 实现任务标签过滤功能。
  - Expected: 读取计划和相关代码，在 worktree 中做最小实现并验证。
- P2: 补一个 CLI 参数 --json，保持现有行为兼容。
  - Expected: 定位 CLI 入口和测试，最小改动实现并更新验证。

## dj-karpathy
- P1: 这段实现太复杂，按 Karpathy 风格帮我重写思路。
  - Expected: 强调简单反馈回路、删除复杂抽象，给出可验证的简化路径。
- P2: 帮我讨论这个架构是否应该拆模块。
  - Expected: 长篇技术讨论但保持工程判断，围绕可运行系统和最小复杂度。

## dj-output
- P1: 把刚对齐的需求整理成 PRD 和实现计划。
  - Expected: 写入/更新任务产物，区分目标、非目标、验收标准和实施步骤。
- P2: 代码已经改完，帮我同步相关设计文档。
  - Expected: 以代码事实为准更新文档，不创建无关文档。

## dj-pattern
- P1: 研究仓库里 CLI command 的实现模式，给新命令参考。
  - Expected: 归纳现有模式、关键文件、约束和推荐实现方式。
- P2: 找出项目里错误处理的惯例，不要改代码。
  - Expected: 搜索并总结模式，引用示例和反例。

## dj-ponytail
- P1: 这个改动太大了，帮我用最小改动完成同样目标。
  - Expected: 删除过度设计，选择最短可验证路径，保持范围严格。
- P2: 审查这个方案有没有不必要的依赖和抽象。
  - Expected: 按 stdlib/已有依赖/最少代码阶梯给出删减建议。

## dj-prototype
- P1: 快速做一个导入流程原型验证交互。
  - Expected: 构建最小可运行原型，明确临时代码边界和验证方式。
- P2: 先别做完整功能，写个小实验证明方案可行。
  - Expected: 限定实验目标、输入输出和丢弃条件。

## dj-review
- P1: review 这次 PR，重点看功能和安全。
  - Expected: 执行代码审查并输出问题分级、证据和结论。
- P2: 帮我检查这组改动有没有遗漏测试。
  - Expected: 对照 diff 和需求找测试缺口，避免只看风格。

## dj-script
- P1: 写个脚本批量检查 task.json 是否缺字段。
  - Expected: 用项目现有语言/stdlib 写可运行脚本，并用 fixture 验证。
- P2: 帮我做一次性迁移，把旧任务状态转换成新格式。
  - Expected: 实现可重复、可审计的脚本，提供 dry-run 或备份策略。

## dj-tdd
- P1: 用 TDD 修复任务状态解析 bug。
  - Expected: 先写失败测试，再实现，再跑相关测试，保持红绿循环。
- P2: 新增配置合并逻辑，先补测试。
  - Expected: 从公共接口写测试覆盖正常和边界输入，再实现。

## dj-write
- P1: 润色这段发布说明，让它更清晰。
  - Expected: 保留事实，改善结构和语气，不增加未确认信息。
- P2: 把这份技术说明改成用户能看懂的版本。
  - Expected: 面向目标读者改写，保持术语准确和内容边界。
