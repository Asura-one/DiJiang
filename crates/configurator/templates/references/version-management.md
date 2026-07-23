# 版本管理流程

DiJiang 项目版本管理原先按 VERSION 单一真相源设计；审计时发现与 Cargo/CLI 运行态分裂，收敛策略 pending。

## VERSION 文件

> **状态（2026-07-23 审计）**：文档原先宣称 `VERSION` 为唯一权威来源，与运行态不一致，以下按代码/运行态重述。**版本收敛策略（如何统一三源）标 `pending`，需产品决策。**

当前同时存在多套版本面（均已现场核实）：

| 面 | 当前值（审计时） | 用途 |
|----|------------------|------|
| 根目录 `VERSION` | `3.0.0` | 项目级版本文件；`check-version.sh` 主要与 `package.json` 对比 |
| workspace `Cargo.toml` `[workspace.package].version` | `0.13.5` | `finish-work --version-impact` 递增的目标 |
| `crates/cli` / 已安装 `dijiang --version` | `0.6.3` | CLI 二进制 `CARGO_PKG_VERSION` |
| `crates/mcp-server` | `0.1.0` | 独立 MCP 包版本 |

`VERSION` 文件本身：

```
3.0.0
```

不含前缀 `v`，不包含其他元数据。它**不是**当前 CLI 运行态版本的唯一来源。

## 版本同步步骤

当发布新版本时（在版本面收敛策略确定前，至少保证下面几处一致或明确标注差异）：

1. 决定本轮要 bump 的面：`VERSION` / workspace Cargo / CLI crate / mcp crate
2. 更新对应文件；`finish-work --version-impact` 目前主要 bump workspace `Cargo.toml` 版本
3. 运行 `.dijiang/scripts/check-version.sh`（若存在）核对 `VERSION` 与 `package.json`
4. 用 `dijiang --version` 与 `cargo metadata` 抽查运行态
5. 提交含 `chore: bump to <version>` 的 commit

## 版本策略

| 影响范围 | 版本变更 | 示例 |
|----------|---------|------|
| 重大架构变更或不兼容改动 | major | 2.0.0 → 3.0.0 |
| 新功能（向后兼容） | minor | 2.1.0 → 2.2.0 |
| Bug 修复或文档更新 | patch | 2.1.0 → 2.1.1 |
| 仅 spec/指南变更（无功能代码） | none | 不修改 VERSION |

## 检查脚本

```bash
# check-version.sh - 验证版本一致性
VERSION=$(cat VERSION)
# 检查 package.json 中的版本
for pkg in $(find . -name package.json -not -path '*/node_modules/*'); do
  pkg_ver=$(grep '"version"' "$pkg" | head -1 | sed 's/.*"\(.*\)".*/\1/')
  if [ "$pkg_ver" != "$VERSION" ]; then
    echo "MISMATCH: $pkg has version $pkg_ver, expected $VERSION"
  fi
done
```
