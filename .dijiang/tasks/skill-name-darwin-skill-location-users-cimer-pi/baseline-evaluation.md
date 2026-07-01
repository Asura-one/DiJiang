# Darwin Baseline Evaluation

Generated: 2026-06-30T16:57:48+00:00

## Scope
- Target: `crates/configurator/templates/skills/*/SKILL.md`
- Skills: 22
- Runtime neutrality scan: passed (`runtime_warn=0`)
- Eval mode: `dry_run` for dim8 because no independent sub-agent execution interface is available in this run.
- Warning: dry_run ratio is 100%; dim8 scores are directional and must be rechecked with full_test before trusting fine-grained ranking.

## Scorecard

| Skill | Score | Weakest dimensions | Structure shortfall | Effect note |
|---|---:|---|---|---|
| `dijiang-start` | 52.6 | dim4=2, dim9=2, dim2=3 | dim4:2, dim9:2, dim2:3 | dry_run显示典型prompt下约束力有限 |
| `dijiang-continue` | 57.7 | dim3=2, dim4=2, dim9=2 | dim3:2, dim4:2, dim9:2 | dry_run显示典型prompt下约束力有限 |
| `dj-ponytail` | 63.0 | dim2=3, dim9=5, dim5=6 | dim2:3, dim9:5, dim5:6 | dry_run显示典型prompt下约束力有限 |
| `dj-handoff` | 63.9 | dim5=3, dim9=5, dim2=6 | dim5:3, dim9:5, dim2:6 | dry_run显示典型prompt下约束力有限 |
| `dj-karpathy` | 65.5 | dim5=5, dim2=5, dim9=6 | dim5:5, dim2:5, dim9:6 | dry_run显示典型prompt下约束力有限 |
| `dijiang-finish-work` | 66.6 | dim3=3, dim4=3, dim9=3 | dim3:3, dim4:3, dim9:3 | dry_run显示能稳定约束典型prompt |
| `dj-design` | 66.7 | dim5=5, dim2=6, dim9=6 | dim5:5, dim2:6, dim9:6 | dry_run显示典型prompt下约束力有限 |
| `dj-pattern` | 67.1 | dim4=1, dim9=5, dim8=7 | dim4:1, dim9:5 | dry_run显示典型prompt下约束力有限 |
| `dj-audit` | 67.2 | dim5=5, dim2=5, dim9=5 | dim5:5, dim2:5, dim9:5 | dry_run显示能稳定约束典型prompt |
| `dj-script` | 67.8 | dim9=5, dim5=6, dim2=6 | dim9:5, dim5:6, dim2:6 | dry_run显示典型prompt下约束力有限 |
| `dj-review` | 69.6 | dim4=4, dim9=5, dim5=6 | dim4:4, dim9:5, dim5:6 | dry_run显示典型prompt下约束力有限 |
| `dj-write` | 69.6 | dim5=5, dim9=5, dim7=6 | dim5:5, dim9:5, dim7:6 | dry_run显示能稳定约束典型prompt |
| `dj-grill` | 71.5 | dim3=2, dim4=2, dim9=5 | dim3:2, dim4:2, dim9:5 | dry_run显示能稳定约束典型prompt |
| `dj-health` | 71.5 | dim5=4, dim9=5, dim3=7 | dim5:4, dim9:5, dim3:7 | dry_run显示能稳定约束典型prompt |
| `dj-tdd` | 72.6 | dim2=5, dim5=6, dim8=7 | dim2:5, dim5:6 | dry_run显示典型prompt下约束力有限 |
| `dj-implement` | 72.9 | dim2=5, dim7=6, dim9=6 | dim2:5, dim7:6, dim9:6 | dry_run显示能稳定约束典型prompt |
| `dj-debt` | 75.1 | dim9=5, dim2=6, dim4=7 | dim9:5, dim2:6, dim4:7 | dry_run显示能稳定约束典型prompt |
| `dj-prototype` | 81.2 | dim9=6, dim3=7, dim7=7 | dim9:6, dim3:7, dim7:7 | dry_run显示能稳定约束典型prompt |
| `dj-output` | 85.8 | dim9=6, dim3=7, dim4=7 | dim9:6, dim3:7, dim4:7 | dry_run显示能稳定约束典型prompt |
| `dj-dispatch` | 87.0 | dim9=6, dim6=7, dim3=8 | dim9:6, dim6:7, dim3:8 | dry_run显示能稳定约束典型prompt |
| `dj-hunt` | 88.8 | dim7=6, dim6=7, dim8=9 | dim7:6, dim6:7 | dry_run显示能稳定约束典型prompt |
| `dj-check` | 91.2 | dim6=7, dim3=8, dim5=9 | dim6:7, dim3:8, dim5:9 | dry_run显示能稳定约束典型prompt |

- Average: 71.6

## Dimension Details

### dijiang-start
- Score: 52.6
- Dimensions: dim1=8, dim2=3, dim3=3, dim4=2, dim5=8, dim6=7, dim7=6, dim8=6, dim9=2
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dijiang-continue
- Score: 57.7
- Dimensions: dim1=8, dim2=8, dim3=2, dim4=2, dim5=8, dim6=8, dim7=4, dim8=7, dim9=2
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-ponytail
- Score: 63.0
- Dimensions: dim1=9, dim2=3, dim3=7, dim4=7, dim5=6, dim6=7, dim7=7, dim8=7, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-handoff
- Score: 63.9
- Dimensions: dim1=9, dim2=6, dim3=7, dim4=7, dim5=3, dim6=7, dim7=9, dim8=7, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-karpathy
- Score: 65.5
- Dimensions: dim1=9, dim2=5, dim3=8, dim4=7, dim5=5, dim6=7, dim7=7, dim8=7, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dijiang-finish-work
- Score: 66.6
- Dimensions: dim1=7, dim2=10, dim3=3, dim4=3, dim5=9, dim6=7, dim7=5, dim8=8, dim9=3
- dim1: frontmatter触发词或使用场景不完整
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-design
- Score: 66.7
- Dimensions: dim1=9, dim2=6, dim3=7, dim4=7, dim5=5, dim6=7, dim7=8, dim8=7, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-pattern
- Score: 67.1
- Dimensions: dim1=9, dim2=7, dim3=7, dim4=1, dim5=7, dim6=7, dim7=8, dim8=7, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-audit
- Score: 67.2
- Dimensions: dim1=9, dim2=5, dim3=7, dim4=7, dim5=5, dim6=7, dim7=8, dim8=8, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-script
- Score: 67.8
- Dimensions: dim1=9, dim2=6, dim3=7, dim4=7, dim5=6, dim6=7, dim7=8, dim8=7, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-review
- Score: 69.6
- Dimensions: dim1=9, dim2=8, dim3=8, dim4=4, dim5=6, dim6=7, dim7=8, dim8=7, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 有显式失败处理和兜底路径
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-write
- Score: 69.6
- Dimensions: dim1=9, dim2=9, dim3=7, dim4=7, dim5=5, dim6=7, dim7=6, dim8=8, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-grill
- Score: 71.5
- Dimensions: dim1=8, dim2=10, dim3=2, dim4=2, dim5=9, dim6=7, dim7=9, dim8=8, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 缺少🔴/🛑/CHECKPOINT显性停点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-health
- Score: 71.5
- Dimensions: dim1=9, dim2=9, dim3=7, dim4=9, dim5=4, dim6=7, dim7=8, dim8=8, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-tdd
- Score: 72.6
- Dimensions: dim1=9, dim2=5, dim3=10, dim4=9, dim5=6, dim6=7, dim7=8, dim8=7, dim9=7
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 执行细节偏少或软化措辞偏多
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示典型prompt下约束力有限
- dim9: 缺少独立反例或危险动作黑名单

### dj-implement
- Score: 72.9
- Dimensions: dim1=9, dim2=5, dim3=9, dim4=7, dim5=8, dim6=7, dim7=6, dim8=8, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-debt
- Score: 75.1
- Dimensions: dim1=9, dim2=6, dim3=8, dim4=7, dim5=8, dim6=8, dim7=8, dim8=8, dim9=5
- dim1: frontmatter含name/description/触发信息
- dim2: 步骤存在但输入/输出或阶段边界不足
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-prototype
- Score: 81.2
- Dimensions: dim1=9, dim2=10, dim3=7, dim4=9, dim5=8, dim6=7, dim7=7, dim8=9, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-output
- Score: 85.8
- Dimensions: dim1=9, dim2=10, dim3=7, dim4=7, dim5=10, dim6=7, dim7=9, dim8=9, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 失败分支不足或缺少if-then兜底表
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-dispatch
- Score: 87.0
- Dimensions: dim1=9, dim2=10, dim3=8, dim4=9, dim5=10, dim6=7, dim7=8, dim8=9, dim9=6
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 缺少独立反例或危险动作黑名单

### dj-hunt
- Score: 88.8
- Dimensions: dim1=9, dim2=10, dim3=10, dim4=9, dim5=10, dim6=7, dim7=6, dim8=9, dim9=9
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 疑似断链资源 1 个
- dim7: 结构偏薄、偏长或存在冗余表达
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 有反例/禁止动作清单

### dj-check
- Score: 91.2
- Dimensions: dim1=9, dim2=10, dim3=8, dim4=9, dim5=9, dim6=7, dim7=9, dim8=10, dim9=10
- dim1: frontmatter含name/description/触发信息
- dim2: 流程分步清晰
- dim3: 有显式失败处理和兜底路径
- dim4: 有显式检查点
- dim5: 命令/格式/示例具体
- dim6: 资源引用少且无明显断链
- dim7: 结构层次基本完整
- dim8: dry_run显示能稳定约束典型prompt
- dim9: 有反例/禁止动作清单

## Baseline Conclusion
- Lowest-scoring cluster is mostly dim3 failure-mode encoding, dim4 explicit checkpoint design, and dim9 counterexamples/blacklists.
- P0 runtime drift is not present, so Phase 2 should start with the lowest score skill and focus on the weakest non-dim8 structural dimension.
- Because dim8 is dry_run-only, improvements must be treated as validation-gated but low-confidence until at least one full_test judge run is available.

🔴 CHECKPOINT · 🛑 STOP: confirm this baseline scorecard before entering Phase 2 optimization.
