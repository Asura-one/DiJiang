# Phase 2 Clarification: Language and Review Relationship

Generated: 2026-07-01T01:44:09+00:00

## User Clarification
- skill 是原子能力；CLI 兼容入口不应写入 skill 能力定义，避免污染职责边界。
- 面向人的 skill 文案不应不必要地中英混用；英文保留给命令、skill 名、代码标识符、状态字面量和固定 spec 章节。
- 澄清 `dijiang review` 与 `dj-review` 的关系，以及 `dijiang review` 是否有真实用途。

## Decision
- `dijiang review` 已从 CLI 中删除。它原本只生成 adversarial / first-principles 审查 prompt 并记录 tactic，不改变 task 状态，不运行审查，不调用 `dj-review` 或 `dj-check`。
- 原 `adversarial` prompt 的有效内容不再只归属单一入口：第一落点是 `dj-check` 的「对抗式安全审查」，同时按职责融合到 `dj-review`、`dj-audit`、`dj-output`、`dj-write` 等验证/审查/论证类 skill。
- 原 `first-principles` prompt 的有效内容不再只归属单一入口：第一落点是 `dj-karpathy` 的「第一性原理审查」，同时按职责融合到 `dj-grill`、`dj-design`、`dj-implement`、`dj-hunt`、`dj-tdd`、`dj-output`、`dj-write`、`dj-audit` 等生成/推导/排障类 skill。
- `dj-review` 保留为轻量只读 diff / PR 审查能力；`dj-check` 保留为交付质量闸门能力。
- skill 模板只描述原子能力，不记录已删除 CLI 命令的关系说明。

## Changes
- 删除 `crates/cli/src/main.rs` 中的 `review` 子命令、dispatch 分支和 `cmd_review` 实现。
- 删除 `crates/cli/tests/e2e.rs` 中的 `dijiang review` e2e 测试。
- 删除 `crates/mem/src/memory.rs` 中内置 `review-adversarial` / `review-first-principles` tactics。
- 在 `dj-check` 中加入对抗式安全审查 lens：输入校验、注入、鉴权、数据泄露、DoS、供应链、竞态和资源泄漏。
- 在 `dj-review` 中加入轻量 diff 对抗式审查：恶意输入、异常数据、资源耗尽、重试/缓存/worker、并发和幂等路径。
- 在 `dj-audit` 中加入全局对抗式审查和第一性原理扫描：系统不变量、攻击路径、异常数据、资源放大器、维护风险。
- 在 `dj-karpathy` 中加入第一性原理审查 lens：fundamental problem、basic facts、hidden assumptions、derived solution、simpler approach、trade-offs。
- 在 `dj-grill`、`dj-design`、`dj-implement`、`dj-hunt`、`dj-tdd` 中加入第一性原理推导，用于需求对齐、设计方向、实现方案、根因排查和测试切片。
- 在 `dj-output`、`dj-write` 中加入非代码场景的第一性原理与对抗式审查，用于文档方案、论证文本、逻辑漏洞和事实断点。
- 将新增的人读标题、契约标签和检查点字段统一为中文表达。
- 保留 `Contracts` 作为 spec 模板中的固定合约章节名。

## Ratchet Check
- `dj-review`: 79.0 -> 79.0 (keep); dim1=9, dim2=8, dim3=9, dim4=9, dim5=6, dim6=7, dim7=8, dim8=9, dim9=6
- `dj-check`: 93.6 -> 95.3 (keep); dim1=9, dim2=10, dim3=10, dim4=9, dim5=10, dim6=7, dim7=9, dim8=10, dim9=10
- `dj-health`: 90.6 -> 90.6 (keep); dim1=9, dim2=9, dim3=10, dim4=9, dim5=9, dim6=7, dim7=9, dim8=10, dim9=7
- `dj-karpathy`: 85.9 -> 86.5 (keep); dim1=9, dim2=10, dim3=8, dim4=9, dim5=8, dim6=7, dim7=8, dim8=10, dim9=7
## Validation
- `cargo check -p dijiang` 通过；仅保留既有 warning（mem 解析结构字段未读、CLI channel helper 未用等），无新增编译错误。
- `cargo test -p dijiang --test e2e` 通过：26 passed。
- 修复 `test_e2e_update_refreshes_existing_platform_hooks` 的跨测试污染：该测试现在用临时 `HOME` 初始化并更新，避免真实全局 skill cache 影响临时项目。
- 所有生成的 `test-prompts.json` 均可解析。
- 所有 template `SKILL.md` 的 Markdown fence 均为偶数。
- Runtime neutrality 红灯扫描无输出。
- 旧 `dijiang review` CLI 引用扫描无命中：`review-adversarial`、`review-first-principles`、`cmd_review`、`Commands::Review`、`test_e2e_review`、`review-*` 均已从 `crates` 中移除。
- 横切 lens 分布扫描确认命中：`dj-grill`、`dj-design`、`dj-implement`、`dj-hunt`、`dj-tdd`、`dj-output`、`dj-write`、`dj-karpathy`、`dj-review`、`dj-check`、`dj-audit`。
- `dj-review` / `dj-check` skill 模板中不再出现 `dijiang review`。
- Shell quoting lesson: `rg` pattern 含反引号时使用单引号，避免 shell command substitution。
