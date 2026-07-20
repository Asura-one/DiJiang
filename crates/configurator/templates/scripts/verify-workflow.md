# 验证流程脚本使用说明

| 脚本 | 检查内容 | 运行方式 |
|------|---------|---------|
| `verify-skills.sh` | 技能结构完整性（frontmatter、Outcome Contract、Hard Rules、Gotchas） | `bash .dijiang/spec/scripts/verify-skills.sh` |
| `verify-references.sh` | 交叉引用有效性（spec 引用路径存在） | `bash .dijiang/spec/scripts/verify-references.sh` |
| `verify-index.sh` | 索引一致性（入口→文件存在，文件→入口存在） | `bash .dijiang/spec/scripts/verify-index.sh` |
| `verify_skills.py` | 高级验证：frontmatter 解析、引用图谱、Outcome Contract 完整性、Hard Rules/Gotchas 结构、JSON 输出 | `python3 .dijiang/spec/scripts/verify_skills.py` |
| `check-version.sh` | VERSION 文件与 package.json 版本一致性 | `bash .dijiang/spec/scripts/check-version.sh` |

## 运行全部检查

### 基础检查

```bash
bash .dijiang/spec/scripts/verify-skills.sh && \
bash .dijiang/spec/scripts/verify-references.sh && \
bash .dijiang/spec/scripts/verify-index.sh
```

### 高级检查（全面推荐）

```bash
python3 .dijiang/spec/scripts/verify_skills.py

# JSON 输出（CI 集成用）
python3 .dijiang/spec/scripts/verify_skills.py --json

# 只检查单个 skill
python3 .dijiang/spec/scripts/verify_skills.py --skill dj-hunt
```

## 集成到交付流程

这些脚本在 `dijiang finish-work` 前作为验证步骤运行。若任何一个脚本退出码非 0，表示当前修改破坏了某条规则，应在交付前修复。

| 脚本 | 检查内容 | 运行方式 |
|------|---------|---------|
| `verify-skills.sh` | 技能结构完整性（frontmatter、Outcome Contract、Hard Rules、Gotchas） | `bash .dijiang/spec/scripts/verify-skills.sh` |
| `verify-references.sh` | 交叉引用有效性（spec 引用路径存在） | `bash .dijiang/spec/scripts/verify-references.sh` |
| `verify-index.sh` | 索引一致性（入口→文件存在，文件→入口存在） | `bash .dijiang/spec/scripts/verify-index.sh` |

## 运行全部检查

```bash
bash .dijiang/spec/scripts/verify-skills.sh && echo "" && \
bash .dijiang/spec/scripts/verify-references.sh && echo "" && \
bash .dijiang/spec/scripts/verify-index.sh
```

## 集成到交付流程

这些脚本在 `dijiang finish-work` 前作为验证步骤运行。若任何一个脚本退出码非 0，表示当前修改破坏了某条规则，应在交付前修复。
