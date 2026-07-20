#!/usr/bin/env bash
# verify-references.sh — 交叉引用有效性检查
# 检查 SKILL.md 中引用的 `.dijiang/references/` 和 `references/` 文件路径是否存在。
# 退出码：0 全部通过，1 有失败项。

set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo "/Users/cimer/Project/DiJiang")"

SKILL_DIR=".pi/skills"
PASS=0
FAIL=0

echo "[verify-references] Checking cross-references in all dj-* skills..."
echo ""

for skill_file in "$SKILL_DIR"/dj-*/SKILL.md; do
    name=$(basename "$(dirname "$skill_file")")
    issues=0

    # 提取所有 `.dijiang/references/` 反引号引用
    while IFS= read -r ref; do
        # 提取反引号内的路径
        ref_dir=$(dirname "$skill_file")
        ref=$(echo "$ref" | sed 's/.*`\(\.[^`]*\)`.*/\1/')
        [ -z "$ref" ] && continue

        # 判断是绝对路径（.dijiang/xxx）还是相对路径（references/xxx）
        case "$ref" in
            .dijiang/*)
                # 绝对路径：相对于项目根
                target="$ref"
                ;;
            references/*)
                # 相对路径：相对于 skill 目录
                target="$ref_dir/$ref"
                ;;
            *)
                # 非引用跳过
                continue
                ;;
        esac

        if [ ! -f "$target" ]; then
            echo "  FAIL: $name — referenced '$ref' not found at $target"
            issues=1
        fi
    done < <(grep -o '`\(\.dijiang/references/[^`]*\|references/[^`]*\)`' "$skill_file" || true)

    if [ "$issues" -eq 0 ]; then
        echo "  PASS: $name"
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
echo ""

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
