#!/usr/bin/env bash
# audit-inventory.sh — dj-gov 盘点专用只读脚本
# 产出四节：规则链存在性 / Markdown 清单 / Git 与 worktree 状态 / skills 软链状态
# 纯只读：不写不删不改任何文件（dj-gov 红线「只盘点不清理」在此脚本层固化）
# 兼容 macOS (bash 3.2+) 与 Linux；在任意 cwd 调用均以仓库根为基准。
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== 规则链 =="
for f in AGENTS.md CLAUDE.md CONTEXT.md SCOPE.md .dijiang/config.toml; do
  if [ -e "$f" ]; then echo "✅ $f"; else echo "—  $f (不存在)"; fi
done

echo
echo "== Markdown 清单（按目录计数）=="
for d in docs skills; do
  if [ -d "$d" ]; then
    n=$(find "$d" -name '*.md' -not -path '*/node_modules/*' 2>/dev/null | wc -l | tr -d ' ')
    echo "$d: $n 个 .md"
  fi
done

echo
echo "== Markdown 文件清单（相对路径）=="
find docs skills -name '*.md' -not -path '*/node_modules/*' 2>/dev/null | sort

echo
echo "== Git 状态 =="
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "非 git 仓库，跳过本节"
  exit 0
fi
echo "branch: $(git branch --show-current 2>/dev/null || echo '(detached)')"
echo "dirty:  $(git status --porcelain | wc -l | tr -d ' ') 个变更"
echo "HEAD:   $(git log -1 --oneline 2>/dev/null || echo '无提交')"

echo
echo "== worktree =="
git worktree list 2>/dev/null || echo "(无)"

echo
echo "== skills 软链状态（安装面探测）=="
linked=0
while IFS= read -r l; do
  tgt=$(readlink "$l" 2>/dev/null || echo '?')
  if [ -e "$l" ]; then echo "✅ $l -> $tgt"; else echo "❌ $l -> $tgt (断链)"; fi
  linked=1
done < <(find skills -name 'SKILL.md' -type l 2>/dev/null)
[ "$linked" -eq 0 ] && echo "(无软链——全部为仓库内实体文件)"
