# edit 工具 mtime 冲突：升级为 Python 补丁脚本

## 症状

read 后立即 edit/multiedit 仍报 `file has been modified since it was last read`，且 Last modification 与 Last read 只差 1~2 秒——环境 hook/监控进程在每次读取后 touch 文件，read→edit 循环追不上，原样重试大概率继续失败。

**升级路径**：重试 1~2 次仍失败即切换 Python 补丁脚本，不要恋战。附带坑：不要用 PowerShell 内联 `python -c` 做含引号的文本替换（引号转义破坏 Python 字符串），一律落成 `.temp/patch_xxx.py` 脚本执行（用项目 venv：`.venv\Scripts\python.exe`）。

## 补丁脚本四要点

1. **锚点选不含全角标点的行**——AIGC hook 会把 .md 中的全角引号等标点改写为半角，含引号的锚点静默失配（报 OLD NOT FOUND 但文件完好）；不确定时先打印目标行 `repr()` 探测实际字节再定锚
2. **幂等检测**：`new in content and old not in content` → ALREADY APPLIED 跳过（multiedit 失败可能已部分落盘，必须容忍中间态）
3. 任一 old 未命中 → **整体报错不写盘**，防止半套补丁
4. 每处改动用精确 `(old, new)` 字符串对 `content.replace(old, new, 1)`

## 补丁报 OLD NOT FOUND 时

先怀疑文件已被之前的失败 multiedit 截断/损坏（grep 关键锚点无匹配、行数骤减——实测 ssh.go 被截成 43 行残片），而不是怀疑补丁脚本写错——用 `git checkout -- <file>` 恢复后重跑补丁即可。

## 为什么每条 edit 前要重新 read

每次成功写入后 mtime 可能被外部进程更新，下一条 edit 就会报 modified since last read；multiedit 失败可能已部分写入，留下不一致中间态——轻则变量已删/引用还在（NameError），重则整文件被截断成碎片。同一文件多处改动优先**整文件重写**。