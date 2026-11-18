---
review_before: 2027-11-06
---

# Live 副本同步验收清单

> 摘要：finish-work 第 11 步 live 同步后的三条验收命令——死技能引用零命中、误删技能补回、漂移清零；全绿才报告同步完成。repo→live 单向，永不反向。

## 前提

- 方向红线：repo（DiJiang 仓库）是权威演进版；live（`~/.config/TeleAgent/users/<id>/skills/` 的 dj-* 系）是投放快照。**只 repo→live，永不 live→repo**——2026-11-05 双向同步误删 4 技能事故 + forgejo 旧快照覆盖事件双重验证。
- live 目录属于用户 skmanager 域：写入前用户已知晓；第三方技能（mattpocock/ponytail/平台域，非 dj 系）**零触碰**。
- 跳过条件：本批次纯文档变更（无技能增删改，如仅动 docs/、scripts/、tests/）→ 跳过整步并在 gate 报告里写明。

## 第 0 步：同步前对账（rsync 前必跑）

rsync `--delete` 是单向覆盖：live 侧若有本批次没进 repo 的内容（典型：平台 skill-evolution 直接改 live 不改 repo），会被抹掉。先跑：

```bash
for s in $(ls "$LIVE" | grep -E "^(dj-|dijiang-)"); do
  if   [ -d "$REPO/engineering/$s" ]; then src="$REPO/engineering/$s"
  elif [ -d "$REPO/productivity/$s" ]; then src="$REPO/productivity/$s"
  else continue; fi
  diff -rq --exclude=".DS_Store" "$src" "$LIVE/$s" >/dev/null 2>&1 || echo "⚠️ 与 repo 不同：$s"
done
```

有差异先判性质：live 比 repo 多内容（平台进化/热修）→ **先回灌 repo 再同步**；live 只是旧版落后 → 直接同步。跳过回灌直接 rsync = 抹掉平台工作。

## 同步命令

```bash
REPO="${DIJIANG_SKILLS_ROOT:?请先 export DIJIANG_SKILLS_ROOT=<DiJiang 仓库根>/skills（如 $PWD/skills）}"  # 环境变量可移植，勿硬编码个人绝对路径
LIVE=~/.config/TeleAgent/users/<id>/skills   # 按当前用户 id 替换

for src in "$REPO"/engineering/* "$REPO"/productivity/*; do
  rsync -a --delete "$src/" "$LIVE/$(basename "$src")/"
done
```

`--delete` 的安全性：只作用于 40 个 dj 系目录内部（源是 repo 枚举），清掉的是已退役技能的尸块残留（如旧 issue-sync 残页）；第三方目录不在 rsync 目标里。

## 三条验收（全绿才算完）

```bash
# 1) 死技能引用零命中（退役/已并入技能；名单以 .out-of-scope/ 退役档案为准，dj-teach 已于 a70c01c 收编复活、不在死名单）
grep -rn "dj-pattern\|dj-triage\|dj-dispatch\|dj-review\|dj-spec-bootstrap" \
  "$LIVE"/dj-ask/SKILL.md "$LIVE"/dj-ask/references/*.md \
  && echo "❌ 仍有残留" || echo "✅ 零残留"

# 2) 全量技能在位：live 的 dj/dijiang 系数量 = repo 的 engineering+productivity 之和
echo "repo: $(ls "$REPO"/engineering "$REPO"/productivity | grep -v ':$' | wc -l) live: $(ls "$LIVE" | grep -cE '^(dj-|dijiang-)')"
# 两数相等即 ✅（不等=有缺/有多，逐个 diff 定位）

# 3) 漂移清零（应无 ❌ 输出）
for s in $(ls "$LIVE" | grep -E "^(dj-|dijiang-)"); do
  if   [ -d "$REPO/engineering/$s" ]; then src="$REPO/engineering/$s"
  elif [ -d "$REPO/productivity/$s" ]; then src="$REPO/productivity/$s"
  else echo "❌ live 多余: $s"; continue; fi
  diff -rq --exclude=".DS_Store" "$src" "$LIVE/$s" >/dev/null 2>&1 || echo "❌ 仍漂移: $s"
done; echo "检查完毕"
```

## 历史教训（为什么有这清单）

- **2026-11-05 早**：skmanager 侧误删 4 个活跃技能（questionnaire/teach/wait-what/write），自动同步把它传播到 live——productivity 桶两连击，恢复靠 git 历史（父提交检出）
- **2026-11-05 晚**：live 全量漂移 32/37——近六批结构变动（吸收/退役/融合/迁徙）都没到 live，dj-ask 用着 8 天前的旧路由图，catalog 还指着已退役技能的坟头
- 共因：**双向同步 + 无方向权威声明**。本清单把方向红线和验收固化进 finish-work，发版后同步不再是记在人脑里的口头习惯
- **2026-11-09**：两轮 dj-absorb 平台进化（只写 live）被 finish-work 第 11 步 rsync 覆盖——平台进化 × 单向同步的盲区。靠 `.backup` + 会话记录回灌修复，本文件「第 0 步」由此而生

来源：finish-work 收尾批新增（2026-11-05 拍板「干吧」），承接 live 漂移全量审计结论
