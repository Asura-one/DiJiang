#!/usr/bin/env bash
# verify-skills.sh — 技能结构完整性检查
# 检查每个 dj-* SKILL.md 的必要结构和交叉引用。
# 退出码：0 全部通过，1 有失败项。

set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo "/Users/cimer/Project/DiJiang")"

SKILL_DIR=".pi/skills"
PASS=0
FAIL=0
ERRORS=""

check_skill() {
    local file="$1"
    local name
    name=$(basename "$(dirname "$file")")
    local basename="${name}/SKILL.md"
    local issues=0

    # 1. 文件存在
    if [ ! -f "$file" ]; then
        echo "  FAIL: $basename — file not found"
        return 1
    fi

    # 2. Frontmatter: --- 开闭
    if ! head -1 "$file" | grep -q '^---$'; then
        echo "  FAIL: $basename — missing opening ---"
        issues=1
    fi

    # 3. name: 字段
    if ! grep -q '^name:' "$file"; then
        echo "  FAIL: $basename — missing name: in frontmatter"
        issues=1
    fi

    # 4. 必要章节
    for section in "## Outcome Contract" "## Hard Rules" "## Gotchas"; do
        if ! grep -Fq "$section" "$file"; then
            echo "  FAIL: $basename — missing section '$section'"
            issues=1
        fi
    done

    if [ "$issues" -eq 0 ]; then
        echo "  PASS: $basename"
        return 0
    fi
    return 1
}

echo "[verify-skills] Checking all dj-* skills..."
echo ""

for skill_file in "$SKILL_DIR"/dj-*/SKILL.md; do
    if check_skill "$skill_file"; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS $skill_file"
    fi
done

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo "Failed files:$ERRORS"
    exit 1
fi
