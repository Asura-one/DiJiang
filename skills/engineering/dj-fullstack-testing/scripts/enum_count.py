#!/usr/bin/env python3
"""枚举完整性交叉校验器：代码侧独立计数 vs checklist 行数，数差即 FAIL。

扫描思路 Adapted from fishzjp/qa-skills core/scripts/scan_signals.py (MIT)：
确定性文件遍历（排序输出、跳过依赖/构建产物、体量封顶）。
适配 DiJiang dj-fullstack-testing 步骤 1.6a：把「每类 grep 独立计数，大于清单 → 回补」
从临时手拼命令固化为可复跑的机器闸门。

用法：
  python3 enum_count.py --preset api-fastapi --root .            # 只输出代码侧计数
  python3 enum_count.py --preset api-fastapi --root . \\
      --checklist docs/testing/enumeration.md --section "后端 API"   # 双侧比对
  python3 enum_count.py --pattern "@click|@submit" --glob "*.vue" --root src --expect 24

预设：routes-vue | routes-react | api-fastapi | api-flask | api-tauri | api-go |
      ui-vue | ui-react | ui-svelte（--list 看正则明细）

判定（dj-fullstack-testing 1.6a）：代码计数 > 清单条数 → 枚举不完整，FAIL（exit 1）；
清单 ≥ 代码计数 → PASS（清单可含人工补充条目）。不带比对时只打印计数，exit 0。
"""

import argparse
import re
import sys
from fnmatch import fnmatch
from pathlib import Path

SKIP_DIRS = {
    ".git", ".svn", ".hg", ".idea", ".vscode", "__pycache__", "node_modules",
    "dist", "build", "vendor", "target", "coverage", ".venv", "venv", ".tox",
    ".mypy_cache", ".pytest_cache", ".temp", "obj", "bin", "out",
    ".dijiang", ".out-of-scope", "referens", ".cache",
}
MAX_FILE_BYTES = 1_000_000
MAX_FILES = 30_000
MAX_HITS_SHOWN = 15

# preset: 正则（按行匹配，与 grep -E 行计数口径一致）, glob 列表, 说明
PRESETS = {
    "routes-vue":   (r"path:\s*['\"]", ["*.vue", "*.ts", "*.js"],
                     "Vue Router 路由定义（含 src/router）"),
    "routes-react": (r"(path|Route)\s*[=:]\s*[\"']", ["*.tsx", "*.jsx", "*.ts", "*.js"],
                     "React Router 路由定义"),
    "api-fastapi":  (r"@(app|router|api)\.(get|post|put|delete|patch)\(", ["*.py"],
                     "FastAPI/Starlette 端点装饰器"),
    "api-flask":    (r"@(app|bp|blueprint)\.route\(", ["*.py"],
                     "Flask 路由装饰器"),
    "api-tauri":    (r"#\[tauri::command\]", ["*.rs"],
                     "Tauri 命令"),
    "api-go":       (r"(HandleFunc|\.(GET|POST|PUT|DELETE|PATCH)\()", ["*.go"],
                     "net/http / gin 路由注册"),
    "ui-vue":       (r"@(click|submit|change|input)\b|v-on:(click|submit)", ["*.vue"],
                     "Vue 交互入口（@click/@submit/@change/@input）"),
    "ui-react":     (r"on(Click|Submit|Change|Input)\s*=", ["*.tsx", "*.jsx"],
                     "React 交互入口"),
    "ui-svelte":    (r"on:(click|submit|change|input)", ["*.svelte"],
                     "Svelte 交互入口"),
}


def iter_files(root: Path):
    count = 0
    stack = [root]
    while stack:
        d = stack.pop()
        for entry in sorted(d.iterdir()):
            if entry.name in SKIP_DIRS and entry.is_dir():
                continue
            if entry.is_dir():
                stack.append(entry)
                continue
            count += 1
            if count > MAX_FILES:
                raise ValueError(f"文件数超过 {MAX_FILES}，请用 --root 收窄范围")
            if entry.stat().st_size <= MAX_FILE_BYTES:
                yield entry


def scan(root: Path, pattern: str, globs):
    rx = re.compile(pattern)
    total = 0
    hits = []
    files_hit = 0
    for f in iter_files(root):
        if not any(fnmatch(f.name, g) for g in globs):
            continue
        try:
            text = f.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        n = 0
        for i, line in enumerate(text.splitlines(), 1):
            if rx.search(line):
                total += 1
                n += 1
                if len(hits) < MAX_HITS_SHOWN:
                    hits.append((str(f), i, line.strip()[:90]))
        if n:
            files_hit += 1
    return total, files_hit, hits


def count_checklist(path: Path, section: str):
    """统计 checklist 文件中指定章节下的条目行（bullet / 编号 / 表格数据行）。

    表格按连续块处理：每块首行是表头不计，分隔行 |---| 不计，数据行才计。
    """
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    heading_re = re.compile(r"^(#{1,6})\s+(.*)$")
    sep_row_re = re.compile(r"^\|[\s:|-]+\|$")
    level, collecting = None, False
    count = 0
    prev_table = False

    def is_bullet(s: str) -> bool:
        return bool(re.match(r"^[-*+]\s+", s) or re.match(r"^\d+[.、)]\s*", s))

    for line in lines:
        m = heading_re.match(line)
        if m:
            this_level = len(m.group(1))
            if collecting and this_level <= level:
                collecting = False  # 同级或更高级标题 = 章节结束
            if not collecting and section in m.group(2):
                level, collecting = this_level, True
            prev_table = False
            continue
        if not collecting:
            continue
        s = line.strip()
        if s.startswith("|"):
            if not sep_row_re.match(s) and prev_table:
                count += 1  # 表格数据行（首行是表头，不计）
            prev_table = True
        else:
            if is_bullet(s):
                count += 1
            elif not s and prev_table:
                pass
            prev_table = False
    return count


def main() -> int:
    parser = argparse.ArgumentParser(
        description="枚举交叉校验：代码 grep 计数 vs checklist 行数",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--preset", choices=sorted(PRESETS), help="预设类别")
    parser.add_argument("--pattern", help="自定义正则（与 --preset 二选一）")
    parser.add_argument("--glob", action="append", default=None, metavar="GLOB",
                        help="文件名 glob，可重复（默认按 preset；自定义 pattern 必须给）")
    parser.add_argument("--root", default=".", help="扫描根目录（默认 .）")
    parser.add_argument("--checklist", help="checklist 枚举文档路径（启用比对）")
    parser.add_argument("--section", help="checklist 里要比对的章节标题（启用比对）")
    parser.add_argument("--expect", type=int, help="直接指定期望条数比对（与 --checklist 互斥）")
    parser.add_argument("--list", action="store_true", help="列出预设明细后退出")
    args = parser.parse_args()

    if args.list:
        for name in sorted(PRESETS):
            rx, globs, desc = PRESETS[name]
            print(f"{name:14} {desc}\n               /{rx}/  glob={globs}")
        return 0
    if not args.preset and not args.pattern:
        parser.error("需要 --preset 或 --pattern（--list 看预设）")
    if args.preset and args.pattern:
        parser.error("--preset 与 --pattern 二选一")
    if args.pattern and not args.glob:
        parser.error("--pattern 需要配套 --glob（如 *.vue，可重复）")
    if args.expect is not None and args.checklist:
        parser.error("--expect 与 --checklist 互斥")

    pattern, globs = (PRESETS[args.preset][0], PRESETS[args.preset][1]) if args.preset \
        else (args.pattern, args.glob)
    root = Path(args.root)

    try:
        total, files_hit, hits = scan(root, pattern, globs)
    except (OSError, ValueError) as e:
        print(f"扫描错误: {e}", file=sys.stderr)
        return 2

    label = args.preset or "custom"
    print(f"[{label}] 代码侧命中 {total} 处（{files_hit} 个文件，glob={globs}）")
    for f, i, line in hits:
        print(f"  {f}:{i}  {line}")
    if total > MAX_HITS_SHOWN:
        print(f"  …（仅展示前 {MAX_HITS_SHOWN} 处）")

    if args.expect is not None:
        ok = total <= args.expect
        print(f"[expect] 期望 ≤ {args.expect}，实际 {total} → {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1

    if args.checklist:
        if not args.section:
            parser.error("--checklist 需要配套 --section")
        cl = Path(args.checklist)
        if not cl.exists():
            print(f"checklist 不存在: {cl}", file=sys.stderr)
            return 2
        n_cl = count_checklist(cl, args.section)
        ok = n_cl >= total
        print(f"[checklist] 章节「{args.section}」条目 {n_cl} 条")
        print(f"  代码 {total} {'≤' if ok else '>'} 清单 {n_cl} → "
              f"{'PASS' if ok else 'FAIL：枚举不完整，回补清单（dj-fullstack-testing 1.6a）'}")
        return 0 if ok else 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
