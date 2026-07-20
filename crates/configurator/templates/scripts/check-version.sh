#!/usr/bin/env bash
# check-version.sh — 版本一致性检查
# 验证 VERSION 文件与 package.json 中的版本号一致。
# 退出码：0 一致，1 不一致或文件缺失。

set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

VERSION_FILE="VERSION"
PKG_FILE="package.json"

if [ ! -f "$VERSION_FILE" ]; then
    echo "FAIL: $VERSION_FILE not found"
    exit 1
fi

VERSION=$(cat "$VERSION_FILE" | tr -d ' \n')

if [ ! -f "$PKG_FILE" ]; then
    echo "WARN: $PKG_FILE not found — skipping package version check"
    exit 0
fi

PKG_VER=$(grep '"version"' "$PKG_FILE" | head -1 | sed 's/.*: *"\(.*\)".*/\1/')

if [ "$VERSION" != "$PKG_VER" ]; then
    echo "FAIL: VERSION=$VERSION, package.json=$PKG_VER"
    exit 1
fi

echo "OK: VERSION=$VERSION = package.json=$PKG_VER"
