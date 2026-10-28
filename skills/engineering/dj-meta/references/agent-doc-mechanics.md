# Agent 文档机制（invocation / 拆分 / 路由技能）

> 由 DJ-012 中文化迁移（原独立 agent 文档技能的机制参考）。文档是 skill 时的特化分支：frontmatter、invocation 选择、路由技能；其余写作规范见 dj-meta 正文「为 agent 写文档」节。

## Invocation 选择

两种选择，交换两种负载：

- **model-invoked**：保留 description，agent 可自主触发，其他技能也可到达。model-invocation 恒包含用户可达——description 只增加 agent 发现能力，从不剥夺人的入口。description 是技能的顶级上下文指针，被迫常驻加载：用常驻上下文负载换取可发现性。机制：省略 `disable-model-invocation`，写一条面向模型的 description 承载触发分支。
- **user-invoked**：从 agent 视野剥离 description（设 `disable-model-invocation: true`），只有人点名才能触发。零上下文负载，但花认知负载——你就是必须记得它存在的索引。description 转为面向人的一句话摘要，剥离触发词列表。

只在 agent 必须自主到达、或另一技能必须到达时选 model-invocation。若只靠手动触发，做 user-invoked，不付上下文负载。

第三判据臂：**纯参考型 model-invoked 技能是多技能共享参考的单一存放地**——多个技能都需要的参考放进一个 model-invoked 技能，一处维护处处可达（技能融合的「并入 references/」正是此判据的实践）。

## 失败模式（agent 文档特有）

| 触发条件 | 一线修复 | 仍失败兜底 |
|---|---|---|
| 指针措辞弱 agent 不到达 | 按触发场景补前置词/触发词 | 内联关键材料 |
| 信息层级放错（重要信息深藏） | 「始终需要」提到前置词区 | 结构重排，可按需取的下沉 |
| 参考文件被删或改名 | 修复指针路径 | 内联必要内容，不依赖外链 |

两个 user-invoked 技能都需要的共享参考两边都放不下（无 description 互不可达）：推到技能系统外的普通文件，任何技能都可指向的外部参考。

## 按 invocation 拆分

拆分时（序列切分见 dj-meta 正文）：有独立引导词应自主触发、或另一技能必须到达时，拆出 model-invoked 技能。新技能的 description 恒占上下文负载——独立可达性必须值回这个价。

## 路由技能

user-invoked 技能多到记不住时，用**路由技能**治理：一个 user-invoked 技能点名其余技能及各自适用时机，人只记一个技能。它只能提示不能触发——user-invoked 无 description，除了人没有谁能到达它们。
