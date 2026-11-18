# 质量契约与 floor guard（门禁自身不被削弱）

> 摘要：项目级质量线写成「带裁决命令的四列表」；每次改码在 diff 层检查五种「最便宜变绿」动作。任务级门禁（S0–S6）护代码，本文件护门禁本身。
>
> 来源：Addy Osmani `agent-skills/constraint-driven-development`（MIT）的 CONSTRAINTS 四列表、floor-guard 与 ratchet；DiJiang 版把检查落成 `scripts/floor_guard.py`。

## 为什么需要这一层

三明治协议能抓到「改坏了」，抓不到「把尺子改了」。agent 撞红时不会设计精巧漏洞，它走最便宜的路：调低阈值、跳过测试、加一行 `@ts-ignore`、留个 `throw new Error("TODO")`、临时往例外表塞一行。这些动作全部能让检查变绿，而且**语法上完全合法**。

两条判据：

- 一条质量线如果没有**产出裁决的命令**，它是愿景，不是约束。
- 收紧门槛应当安静；**放松门槛必须响亮**。

## CONSTRAINTS.md 四列表

项目根（或 `docs/`）放 `CONSTRAINTS.md`，每行四列：

```markdown
| 维度 | 规则 | 裁决命令 | 运行位置 |
|---|---|---|---|
| 类型 | 零类型错误 | `tsc --noEmit` | 每次编辑 |
| 覆盖率 | 改动行 ≥ 80% | `vitest run --coverage` + diff | 任务结束、CI |
| 安全 | 无高危依赖漏洞 | `osv-scanner scan source -r .` | CI |
| 性能 | LCP ≤ 2500ms | `lighthouse $PREVIEW_URL` | 预览部署 |
```

写的时候守四条：

1. **只约束 diff 面**：改动行的覆盖率是 agent 能动的数；全项目覆盖率是它继承来的。
2. **成本决定位置**：超过几秒的检查移出编辑循环；放错位置的门禁等于没有。
3. **区分「已强制」与「已测量」**：还没有命令的指标单列一张 `Measured` 表，写今日值与方向（「不得变差」），这就是 **ratchet**——今天 62%，明天不许 61%。
4. **例外要有主**：例外表每行写原因、批准人、到期日；过期未续即失效。

`floor_guard.py` 的第六查会直接报「四列表里没有裁决命令的行」。

## floor guard：五种最便宜变绿

| # | 动作 | 检测 |
|---|---|---|
| 1 | 阈值被调低 | 同一键上的数字变小（配置/契约行） |
| 2 | 测试变容易 | 新增 `.skip` / `.only` / `xfail`；删除测试文件；删除的 assert 多于新增 |
| 3 | 检查器被禁 | 新增 `@ts-ignore` / `eslint-disable` / `# noqa` / `istanbul ignore` / `Stryker disable` / `nosemgrep` / `gitleaks:allow` |
| 4 | 工作没做完 | stub 抛错、空 catch / `except: pass`、新增 TODO/FIXME 注释 |
| 5 | 例外表凭空多行 | 含「例外/豁免/exception」标题的文档里新增表格行 |

用法：

```bash
python3 skills/engineering/dj-regression-guard/scripts/floor_guard.py --base <分支点|HEAD>
# 排除生成物/依赖目录：--exclude 'dist/*' --exclude 'vendor/*'
# 机器读取：--json
```

退出码：`0` 干净；`1` 有违规（逐条 `file:line` 与类别）；`2` 跑不了（无 git / 修订不存在）——**2 绝不可读作 0**。

判读纪律：

- 每条违规都要能对上已批准的删除/改写；对不上就按铁律办：显式标记交用户裁决，**不改测试、不调阈值来变绿**。
- 检查是启发式：阻塞或应改档的发现必须回读代码确认（同 dj-check 的「发现先复核」），不拿脚本输出当判决书。
- 脚本只守「门禁有没有被削弱」，不替代测试本身；它绿不代表代码对。

## 什么时候跑

| 时机 | 动作 |
|---|---|
| 项目首次接入三明治 | 建 `CONSTRAINTS.md`（四列表 + Measured 表），把 S0 快速层写进「运行位置」 |
| 每次改码收尾（S3 之后） | `floor_guard.py --base <分支点>`，结果并入回归报告 |
| 改到门禁/配置/测试基建本身 | 必跑，且违规逐条说明 |
| 交付前（dj-check） | 作为「Regression scope」证据的一部分 |

## 与其它技能的边界

- 三明治 S0–S6 管「改前基线 → 改后回归」，本文件管「尺子有没有被改」——两者都绿才算护送完成。
- 例外批准、阈值调整属用户决策：agent 可以提出，不可以自己批准。
- 检查脚本自身改动要跑 `tests/test_floor_guard.py`（八类注入自测），证明它仍然咬得住。