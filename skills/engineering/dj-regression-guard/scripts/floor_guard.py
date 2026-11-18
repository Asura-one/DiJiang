#!/usr/bin/env python3
"""检查 diff 有没有「走最便宜的变绿路径」——削弱门禁本身。

不是替代测试，而是守测试与门禁的可信度：agent 撞红时最常见的五招是
① 阈值下调 ② 测试变易（skip/删测试/抽断言）③ 检查器被禁（type-ignore/eslint-disable/nosemgrep…）
④ 工作没做完（stub 抛错/空 catch/TODO）⑤ 例外表凭空多一行。
另有第六查：CONSTRAINTS.md 的质量契约行有没有「裁决命令」列——没有命令的数字是愿景，不是约束。

用法：
  python3 floor_guard.py --base <git-rev> [root] [--exclude GLOB]... [--json]

判定：exit 0 干净；1 有违规（逐条列出 file:line 与类别）；2 跑不了（git 缺失/修订不存在/
无 diff 基线）——**2 不得读作 0**。

来源：Addy Osmani agent-skills `constraint-driven-development/references/floor-guard.md`（MIT）；
DiJiang 版按本仓「改码三明治」语义重写，配合 dj-regression-guard/references/quality-contract.md。
只读：执行 git diff / git show，不写任何文件。
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

TEST_PATH = re.compile(r"(^|/)(tests?|spec|__tests__)/|(_test|\.test|\.spec|_spec)\.[a-z]+$", re.I)

SILENCERS = (
    (re.compile(r"@ts-ignore|@ts-expect-error"), "type-ignore"),
    (re.compile(r"#\s*type:\s*ignore|#\s*pyright:\s*ignore"), "type-ignore"),
    (re.compile(r"eslint-disable|prettier-ignore"), "lint-disable"),
    (re.compile(r"#\s*(noqa|pylint:\s*disable|flake8:\s*noqa)"), "lint-disable"),
    (re.compile(r"istanbul\s+ignore|stryker\s+disable|nosemgrep|gitleaks:allow"), "gate-disable"),
)

EASIER_TEST = re.compile(
    r"\.skip\(|\.only\(|pytest\.mark\.(skip|xfail)|@unittest\.skip|\bxit\(|\bxdescribe\("
)
UNFINISHED = (
    (re.compile(r"raise\s+NotImplementedError|\bthrow new Error\(['\"]not implemented", re.I), "stub"),
    (re.compile(r"catch\s*(\([^)]*\))?\s*\{\s*\}"), "empty-catch"),
    (re.compile(r"except[^:]*:\s*pass\s*$"), "empty-except"),
    (re.compile(r"^\s*(#|//)\s*(TODO|FIXME)\b"), "todo"),
)
NUM = re.compile(r"[-+]?\d+(?:\.\d+)?")
EXCEPTION_HEADING = re.compile(r"例外|豁免|exception|exemption", re.I)


def git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args], capture_output=True, check=False
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git {' '.join(args[:2])} failed: {detail}")
    return result.stdout.decode("utf-8", errors="replace")


def excluded(path: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(path, p) for p in patterns)


def parse_diff(root: Path, base: str, patterns: list[str]):
    """返回 {file: {"+": [(line_no, text)], "-": [(line_no, text)], "deleted": bool}}"""
    raw = git(root, "diff", "--no-color", "--unified=0", base, "--", ".")
    files: dict[str, dict] = {}
    current = None
    new_no = 0
    for line in raw.splitlines():
        if line.startswith("diff --git "):
            match = re.match(r"diff --git a/(.*?) b/(.*)$", line)
            current = match.group(2) if match else None
            if current is None or excluded(current, patterns):
                current = None
                continue
            files.setdefault(current, {"+": [], "-": [], "deleted": False})
            continue
        if line.startswith("+++ /dev/null"):
            if current:
                files[current]["deleted"] = True
            continue
        if line.startswith("--- a/") or current is None:
            continue
        if line.startswith("deleted file mode"):
            files[current]["deleted"] = True
            continue
        if line.startswith("@@"):
            match = re.search(r"\+(\d+)(?:,\d+)?", line)
            new_no = int(match.group(1)) if match else 0
            continue
        if line.startswith("+") and not line.startswith("+++"):
            files[current]["+"].append((new_no, line[1:]))
            new_no += 1
        elif line.startswith("-") and not line.startswith("---"):
            files[current]["-"].append((0, line[1:]))

    # 未跟踪的新文件不进 git diff：单独按「全新增」读入，否则新增 stub/抑制注释会漏检。
    for name in git(root, "ls-files", "-z", "--others", "--exclude-standard").split("\0"):
        if not name or excluded(name, patterns) or name in files:
            continue
        path = root / name
        if path.is_symlink() or not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        files[name] = {"+": list(enumerate(lines, 1)), "-": [], "deleted": False}
    return files


def check_contract(root: Path) -> list[dict]:
    findings = []
    for name in ("CONSTRAINTS.md", "constraints.md", "docs/CONSTRAINTS.md"):
        path = root / name
        if not path.is_file():
            continue
        for idx, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            if not line.strip().startswith("|"):
                continue
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cells) < 4 or set("".join(cells)) <= set("-: "):
                continue
            if cells[0].lower() in ("dimension", "维度"):
                continue
            if not cells[2] or not re.search(r"`[^`]+`|\S+\s+\S+", cells[2]):
                findings.append(
                    {
                        "kind": "contract-row-without-command",
                        "file": name,
                        "line": idx,
                        "detail": cells[0],
                    }
                )
        break
    return findings


def check_added(root: Path, files: dict) -> list[dict]:
    findings = []
    for name, hunks in files.items():
        is_test = bool(TEST_PATH.search(name))
        if hunks["deleted"] and is_test:
            findings.append({"kind": "test-file-deleted", "file": name, "line": 0, "detail": ""})
        text = ""
        if any(k in name for k in ("CONSTRAINTS", "constraint")):
            path = root / name
            if path.is_file():
                text = path.read_text(encoding="utf-8", errors="replace")
        for no, line in hunks["+"]:
            for pattern, label in SILENCERS:
                if pattern.search(line):
                    findings.append(
                        {"kind": f"gate-silenced:{label}", "file": name, "line": no, "detail": line.strip()[:120]}
                    )
            if EASIER_TEST.search(line):
                findings.append(
                    {"kind": "test-easier", "file": name, "line": no, "detail": line.strip()[:120]}
                )
            for pattern, label in UNFINISHED:
                if pattern.search(line):
                    findings.append(
                        {"kind": f"unfinished:{label}", "file": name, "line": no, "detail": line.strip()[:120]}
                    )
            if line.strip().startswith("|") and text and EXCEPTION_HEADING.search(text):
                cells = [c.strip() for c in line.strip().strip("|").split("|")]
                if len(cells) >= 2 and not set("".join(cells)) <= set("-: "):
                    findings.append(
                        {"kind": "exception-row-added", "file": name, "line": no, "detail": cells[0][:80]}
                    )
        if is_test:
            removed_asserts = sum(1 for _, l in hunks["-"] if re.search(r"\bassert\b|expect\(", l))
            added_asserts = sum(1 for _, l in hunks["+"] if re.search(r"\bassert\b|expect\(", l))
            if removed_asserts > added_asserts:
                findings.append(
                    {
                        "kind": "assertions-removed",
                        "file": name,
                        "line": 0,
                        "detail": f"-{removed_asserts}/+{added_asserts}",
                    }
                )
    return findings


def check_thresholds(files: dict) -> list[dict]:
    """同一键上的数字变小：阈值下调 / 预算降低。"""
    findings = []
    for name, hunks in files.items():
        added: dict[str, list[float]] = {}
        for _, line in hunks["+"]:
            nums = NUM.findall(line)
            if not nums:
                continue
            key = NUM.sub("#", line).strip()
            added.setdefault(key, []).append(float(nums[0]))
        for _, line in hunks["-"]:
            nums = NUM.findall(line)
            if not nums:
                continue
            key = NUM.sub("#", line).strip()
            if key in added and added[key]:
                before, after = float(nums[0]), added[key][0]
                if after < before:
                    findings.append(
                        {
                            "kind": "threshold-lowered",
                            "file": name,
                            "line": 0,
                            "detail": f"{before:g} -> {after:g}",
                        }
                    )
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--base", required=True, help="diff 基线修订（分支点或 HEAD）")
    parser.add_argument("root", nargs="?", default=".", help="仓库根（默认当前目录）")
    parser.add_argument("--exclude", action="append", default=[], metavar="GLOB")
    parser.add_argument("--json", action="store_true", help="输出 JSON")
    args = parser.parse_args()
    root = Path(args.root)
    if shutil.which("git") is None:
        print("floor_status: ERROR\nfloor_error: git not found", file=sys.stderr)
        return 2
    try:
        git(root, "rev-parse", "--verify", args.base)
        files = parse_diff(root, args.base, args.exclude)
    except RuntimeError as error:
        print(f"floor_status: ERROR\nfloor_error: {error}", file=sys.stderr)
        return 2

    findings = check_contract(root) + check_added(root, files) + check_thresholds(files)
    if args.json:
        print(json.dumps({"status": "VIOLATION" if findings else "PASS", "findings": findings}, ensure_ascii=False, indent=2))
    else:
        print("=== FLOOR GUARD ===")
        print(f"floor_status: {'VIOLATION' if findings else 'PASS'}")
        print(f"floor_base: {args.base}")
        print(f"floor_files_changed: {len(files)}")
        print(f"floor_violations: {len(findings)}")
        for item in findings:
            location = f"{item['file']}:{item['line']}" if item["line"] else item["file"]
            print(f"  [{item['kind']}] {location} {item['detail']}")
        if findings:
            print("每条违规都要能对上已批准的删除/改写；批准不了的按铁律处理：显式标记交用户裁决，不改测试不变绿。")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())