#!/usr/bin/env python3
"""核对「瘦身」是搬走还是弄丢了指令行。

把 git 基线修订里的指令文件逐行与当前树（或指定 --head 修订）比较：同一行文本
（空白归一化后）只要仍出现在任一指令文件中就算保留——整行搬进 references、拆到
子文件、挪进另一份文档都算搬。标题与纯标记行不计数。每一条被报"消失"的行，
必须能对上瘦身台账里已确认的删除或改写。

用法：
  python3 check_moved_lines.py <base-rev> [root] [--head <rev>] [--include GLOB ...]

加 --head 时比较两个已提交修订，用于对**已经提交**的瘦身做事后审计；
不加则比较基线修订与当前工作区（含未跟踪、排除 ignore）。

判定：exit 0 无缺失行；1 有缺失行（逐行列出）；2 用法或 git 错误（**不得读作通过**）。

来源：Waza 3.42.0 `skills/health/scripts/check_moved_lines.py`（MIT, © 2026 Tw93），
DiJiang 版改为本仓指令文件布局并保留同一判定语义。上游依据：30 个瘦身提交复查
抓到 5 条被误删的约束，故瘦身流程以本脚本收尾。

配合：dj-meta/references/context-diet.md 步骤 11；dj-gov 清理与 live 同步前的体积瘦身。
只读：仅执行 git ls-tree / git show / git ls-files，不写任何文件。
"""

from __future__ import annotations

import argparse
import fnmatch
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

# DiJiang 的指令面：技能正文、技能参考、仓库级 agent 文档、治理文档。
INSTRUCTION_GLOBS = (
    "AGENTS.md",
    "*/AGENTS.md",
    "CLAUDE.md",
    "*/CLAUDE.md",
    "skills/*/SKILL.md",
    "skills/*/references/*.md",
    "skills/*/scripts/*.md",
    "docs/*.md",
    "docs/*/*.md",
)
MAX_FILE_BYTES = 2_000_000
MAX_REPORTED = 200
# 标题与纯标记不承载约束；整节搬走时它们自然消失。
MARKUP_ONLY = re.compile(r"^(?:#.*|---+|[|:\-\s]+|```\w*|[-*]|\d+\.)$")


def is_instruction(path: str, extra: list[str]) -> bool:
    # fnmatch 的 `*` 跨 `/`，所以 `*/AGENTS.md` 覆盖任意深度的同名文档。
    return any(fnmatch.fnmatch(path, pattern) for pattern in (*INSTRUCTION_GLOBS, *extra))


def normalize(line: str) -> str:
    return " ".join(line.split())


def countable(text: str) -> bool:
    return len(text) >= 3 and not MARKUP_ONLY.match(text)


def git(root: Path, *args: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(root), *args], capture_output=True, check=False
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git {' '.join(args[:2])} failed: {detail}")
    return result.stdout


def tree_files(root: Path, base: str, extra: list[str]) -> dict[str, list[str]]:
    names = git(root, "ls-tree", "-r", "-z", "--name-only", base).split(b"\0")
    files: dict[str, list[str]] = {}
    for raw in names:
        name = raw.decode("utf-8", errors="replace")
        if name and is_instruction(name, extra):
            content = git(root, "show", f"{base}:{name}")[:MAX_FILE_BYTES]
            files[name] = content.decode("utf-8", errors="replace").splitlines()
    return files


def current_files(root: Path, extra: list[str]) -> dict[str, list[str]]:
    names = git(
        root, "ls-files", "-z", "--cached", "--others", "--exclude-standard"
    ).split(b"\0")
    files: dict[str, list[str]] = {}
    resolved_root = root.resolve()
    for raw in names:
        name = raw.decode("utf-8", errors="replace")
        if not name or not is_instruction(name, extra):
            continue
        path = root / name
        if path.is_symlink() or not path.is_file():
            continue
        if resolved_root not in path.resolve().parents:
            continue
        with path.open("rb") as handle:
            content = handle.read(MAX_FILE_BYTES)
        files[name] = content.decode("utf-8", errors="replace").splitlines()
    return files


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("base", help="瘦身前的 git 修订，如 HEAD 或某个 commit")
    parser.add_argument("root", nargs="?", default=".", help="仓库根（默认当前目录）")
    parser.add_argument(
        "--head",
        metavar="REV",
        help="与这个已提交修订比较，而不是工作区",
    )
    parser.add_argument(
        "--include",
        action="append",
        default=[],
        metavar="GLOB",
        help="额外的指令路径 glob，如 'docs/agent/*.md'（可重复）",
    )
    args = parser.parse_args()
    root = Path(args.root)
    if shutil.which("git") is None:
        print("moved_lines_status: ERROR\nmoved_lines_error: git not found", file=sys.stderr)
        return 2
    try:
        before = tree_files(root, args.base, args.include)
        after = (
            tree_files(root, args.head, args.include)
            if args.head
            else current_files(root, args.include)
        )
    except RuntimeError as error:
        print(f"moved_lines_status: ERROR\nmoved_lines_error: {error}", file=sys.stderr)
        return 2

    remaining = Counter(
        text for lines in after.values() for text in map(normalize, lines) if countable(text)
    )
    missing: list[tuple[str, int, str]] = []
    base_count = 0
    for name in sorted(before):
        for number, line in enumerate(before[name], 1):
            text = normalize(line)
            if not countable(text):
                continue
            base_count += 1
            if remaining[text] > 0:
                remaining[text] -= 1
            else:
                missing.append((name, number, text))

    print("=== MOVED LINES ===")
    print(f"moved_lines_status: {'WARN' if missing else 'PASS'}")
    print(f"moved_lines_base: {args.base}")
    print(f"moved_lines_head: {args.head or 'working tree'}")
    print(f"moved_lines_base_files: {len(before)}")
    print(f"moved_lines_current_files: {len(after)}")
    print(f"moved_lines_base_lines: {base_count}")
    print(f"moved_lines_missing: {len(missing)}")
    if missing:
        print("missing_lines (每条都必须对上台账里已确认的删除或改写):")
        for name, number, text in missing[:MAX_REPORTED]:
            print(f"  {name}:{number} {text[:160]}")
        if len(missing) > MAX_REPORTED:
            print(f"  ... {len(missing) - MAX_REPORTED} more")
    return 1 if missing else 0


if __name__ == "__main__":
    raise SystemExit(main())