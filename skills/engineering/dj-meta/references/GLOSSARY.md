# 技能写作术语表

> 摘要：dj-meta 的写作术语唯一真相源。仓库根 `CONTEXT.md` 是项目级术语（Skill/Bucket/Task 等），**不随技能投放**；本文件随技能目录分发，设备上只有这一份，写技能时以本文件为准。

| 术语 | 定义 |
|---|---|
| SKILL.md | 技能核心指令文件，与技能目录同名；正文守 3000 字符红线，超出的深度细节外溢 references/ |
| references/ | 本技能的条件加载配套文档（深度论证/模板/清单），由 SKILL.md 挂指针后才算可达 |
| scripts/ | 本技能的确定性脚本，随技能目录投放；与仓库根 `scripts/`（仓库级工具，不随技能走）严格区分 |
| frontmatter | SKILL.md 顶部 YAML 元数据块：`name`（= 目录名）、`description`（含触发词，供路由）、可选 `disable-model-invocation` |
| disable-model-invocation | frontmatter 标记：**禁止模型自动调用，只能由用户显式触发**（user-invoked skill，负责编排和入口） |
| flow | 多技能串联的工作路径（如 idea→ship 主流程）；本库旧称 pipeline，现已统一为 flow |
| user-invoked skill | 只能由用户显式调用的技能；不能调用另一个 user-invoked skill |
| model-invoked skill | 可由模型自动触发或用户调用的技能，承载可复用的工程纪律 |