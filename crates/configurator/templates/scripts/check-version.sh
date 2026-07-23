#!/usr/bin/env bash
# check-version.sh — 版本一致性检查（ADR 004）
# 验证根 VERSION（若存在）与 Cargo.toml [workspace.package].version 一致。
# 退出码：0 一致，1 不一致或权威版本缺失。

set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

CARGO_TOML="Cargo.toml"
VERSION_FILE="VERSION"

if [ ! -f "$CARGO_TOML" ]; then
    echo "FAIL: $CARGO_TOML not found (workspace version authority)"
    exit 1
fi

WS_VER=$(awk '
  /^\[workspace\.package\]/ { in_pkg=1; next }
  /^\[/ { in_pkg=0 }
  in_pkg && $1 == "version" {
    gsub(/"/, "", $3)
    print $3
    exit
  }
' "$CARGO_TOML")

if [ -z "${WS_VER:-}" ]; then
    echo "FAIL: could not parse [workspace.package].version from $CARGO_TOML"
    exit 1
fi

if [ ! -f "$VERSION_FILE" ]; then
    echo "OK: workspace version=$WS_VER (no VERSION file to compare)"
    exit 0
fi

FILE_VER=$(tr -d ' \n' < "$VERSION_FILE")

if [ "$WS_VER" != "$FILE_VER" ]; then
    echo "FAIL: workspace=$WS_VER, VERSION=$FILE_VER"
    exit 1
fi

echo "OK: VERSION=$FILE_VER = workspace=$WS_VER"
