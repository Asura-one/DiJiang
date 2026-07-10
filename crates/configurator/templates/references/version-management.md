# 版本管理流程

DiJiang 项目版本管理遵循 VERSION 单一真相源原则。

## VERSION 文件

项目根目录的 `VERSION` 文件是版本号的唯一权威来源：

```
3.0.0
```

不含前缀 `v`，不包含其他元数据。

## 版本同步步骤

当发布新版本时：

1. 更新 `VERSION` 文件
2. 运行 `dijiang update` 同步到关联元数据
3. 运行 `.dijiang/spec/scripts/check-version.sh` 验证各引用文件的版本一致性（如果存在）
4. 提交含 `chore: bump to <version>` 的 commit

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
