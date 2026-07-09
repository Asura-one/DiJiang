---
name: dj-debt
description: >
  追踪被推迟的技术捷径：收集代码中所有 ponytail: 标记，生成债务台账。
  Use when the user wants to see what shortcuts were taken and what needs to be revisited.
  触发词：债务、技术债、debt、ponytail、标记、台账。
---

# Debt: 技术债追踪

收集代码中所有 `ponytail:` 标记，生成债务台账。定期清点才能有意识管理。

## 工作流

1. 搜索整个项目中所有 `ponytail:` 标记：
   ```bash
   grep -rn 'ponytail:' --include='*.py' --include='*.ts' --include='*.rs' --include='*.go' .
   ```
2. 按标记整理成债务台账
3. 汇报统计：总数、按严重度分布、按模块分布

## 债务台账

```markdown
## 技术债务台账（<日期>）
总数：N 项

| 文件 | 行号 | 标记内容 | 严重度 | 年龄 |
|---|---|---|---|---|
| path/file.py | 42 | 临时绕过，需重构 | 高 | 3 个月 |
```

## 标记约定

所有技术债用 `ponytail:` 标记，格式：
```python
ponytail: <描述> # <高/中/低> <日期>
```

## 边界

- 只搜索 `ponytail:` 标记，不做代码分析
- 仅报告，不标记新 debt
