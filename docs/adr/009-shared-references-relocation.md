---
review_before: 2027-11-06
---

# 009 — 共读参考文档随技能迁徙（只装 skills 场景适配）

## 状态

accepted

复验：2027-11-06

## 上下文

DiJiang 的真实使用模型是**只装 skills**（README 安装指令即 `cp -r skills/`），用户不克隆整个仓库。但 21 份 docs/references/ 中有 13 份被 skills 引用（38 处，anti-patterns 被 11 个技能共读）——安装后这些引用全部变为幽灵路径，核心纪律文档（证据门/反模式/决策阶梯）随装随断。用户问「docs/ 目录现在是否合适」触发本决策。

## 决策

**按消费者归家（折中案 C）**：被 skills 消费的 13 份文档改为技能自带 references，随 `cp -r skills/` 完整走；纯开发面 8 份文档留 docs/。

### 归家映射（按主消费技能）

| 文档 | 归家 | 理由 |
|------|------|------|
| anti-patterns | dj-meta | 11 消费者——体系治理者持有 |
| decision-ladder | dj-ponytail | 9 消费者——决策阶梯本源是 ponytail 哲学 |
| code-task-contract | dj-implement | 5 消费者——契约的执行主面 |
| evidence-gate / design-modes / output-markers / intensity-levels | dj-check | 完成判定与门禁面 |
| coverage-ledger | dj-audit | 与 git-archaeology 同域（bug 聚类） |
| three-asset-boundaries / loop-design / loop-runtime-checklist | dj-loop | loop 基座三件 |
| api-doc-template | dj-output | 文档产出技能 |
| read-pattern | dj-read | 唯一消费者 |

### 引用规范

- **技能内自引用相对路径**（`references/X.md`）——同技能内最短且安装后有效
- **跨技能引用用仓库相对完整路径**（`skills/engineering/<skill>/references/X.md`）——安装场景下技能目录同根（`cp -r skills/` 保持整体结构），完整路径仍然可达；单技能摘装场景接受断链（罕见，dj-meta 模板已用完整路径惯例）

### 替代方案（三态记录)

- **否决**：B 全搬（21 份全随技能走）——version-management 等开发面文档无技能消费者，跟着走是污染
- **否决**：A 保留 + 安装器复制 docs——多一个安装步骤与状态面，且 8 份开发面文档本就不该装
- **暂缓**：安装脚本（cp 时校验引用完整性）——等真实用户报告安装后再断链的失败案例时复活

## 明确不做

- 不改 docs/adr、docs/guide、docs/architecture——开发面，仓库自身决策记录
- 不为跨技能引用建缩写机制（如 `@ref:X`）——多一层间接层违反 YAGNI

## 影响

- 正面：只装 skills 后纪律文档不再断链；单权威源不变（每文档一个真身）
- 负面：跨技能引用路径变长（仓库相对完整形态）；未来新共读文档必须先选「归家技能」，映射决策成本前移
- 顺手修复：dj-wait-what 引用不存在的 CONTEXT-MAP.md（pi 时代残留）已清

## 关联

- README 安装节同步（强调 skills/ 整体复制）
-`scripts/verify_skills.py` phantom-scripts 门禁不查 md 引用，跨技能引用漂移靠审核
