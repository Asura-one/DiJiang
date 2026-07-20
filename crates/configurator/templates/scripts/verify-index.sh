#!/usr/bin/env bash
# verify-index.sh — 索引一致性检查
# 验证 guides/index.md 的每个入口都对应一个实际文件，
# 且每个 guide 文件都在索引中有入口。
# 退出码：0 全部通过，1 有失败项。

set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo "/Users/cimer/Project/DiJiang")"

GUIDES_DIR=".dijiang/spec/guides"
INDEX_FILE="$GUIDES_DIR/index.md"
LINKED=0
ORPHAN_INDEX=0
ORPHAN_FILE=0
FIX_LIST=""

echo "[verify-index] Checking guides/index.md consistency..."
echo ""

if [ ! -f "$INDEX_FILE" ]; then
    echo "FAIL: $INDEX_FILE not found"
    exit 1
fi

# 1. 收集索引中所有文件引用
echo "  Checking index entries point to existing files..."
while IFS= read -r link; do
    target=$(echo "$link" | sed 's/.*(\.\/\([^)]*\)).*/\1/')
    [ -z "$target" ] || [ "$target" = "$link" ] && continue
    full_path="$GUIDES_DIR/$target"
    if [ ! -f "$full_path" ]; then
        echo "  FAIL: index entry '$target' → file not found"
        ORPHAN_INDEX=$((ORPHAN_INDEX + 1))
        FIX_LIST="$FIX_LIST $target(index→missing)"
    else
        LINKED=$((LINKED + 1))
    fi
done < <(grep -o '\[[^]]*\](\./[^)]*)' "$INDEX_FILE" || true)

# 2. 每个 guide 文件都在索引中
echo "  Checking guide files have index entries..."
for guide_file in "$GUIDES_DIR"/*.md; do
    basename=$(basename "$guide_file")
    [ "$basename" = "index.md" ] && continue
    # 用文件名（不含扩展名）匹配索引
    name="${basename%.md}"
    if ! grep -qF "$name" "$INDEX_FILE" 2>/dev/null; then
        echo "  FAIL: $basename — no entry in index.md"
        ORPHAN_FILE=$((ORPHAN_FILE + 1))
        FIX_LIST="$FIX_LIST $basename(file→missing entry)"
    fi
done

echo ""
echo "=== Results: $LINKED linked, $ORPHAN_INDEX missing files, $ORPHAN_FILE missing entries ==="
echo ""

if [ "$ORPHAN_INDEX" -gt 0 ] || [ "$ORPHAN_FILE" -gt 0 ]; then
    echo "Issues:$FIX_LIST"
    exit 1
fi
echo "  All entries valid."
