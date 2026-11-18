#!/usr/bin/env python3
"""验证「修复附带的新测试」是否真的锁定 bug：把候选测试冻结到基线版本上跑。

Adapted from petrkindlmann/qa-skills skills/bug-reproduction/scripts/regression-check.mjs (MIT)。
原版绑定 JS 测试输出解析；DiJiang 版改为语言无关：只看退出码 + 输出正则，
判定的是同一件事——这些测试在没有修复的基线上会不会红。

为什么需要（配合 dj-regression-guard S2.5 与 dj-hunt「没有好 seam 锁 bug」）：
候选绿 ≠ 测试抓到了 bug；测试可能在基线上也绿（没抓住）。本脚本把候选的测试文件
搬到基线 worktree 上跑，红/绿对比才证明修复被测试锁定。

用法：
  # 修复在工作区（未提交），基线 = 上一个提交
  python3 regression_check.py --base-ref HEAD --test-cmd "pytest tests/test_fix.py -q" --symptom "AssertionError|422"
  # 修复已提交，基线 = 父提交；自动发现本次变更的测试文件
  python3 regression_check.py --base-ref HEAD~1 --test-cmd "go test ./..."
  # 测试依赖修复新增的辅助模块：显式带上
  python3 regression_check.py --base-ref HEAD --test-cmd "pytest -q" \
      --test-path tests/test_fix.py --extra-path tests/helpers.py

判定（exit 0 仅第一种）：
  shows_the_bug       基线红（命中 --symptom）+ 候选绿 → 修复被测试锁定
  green_at_base       基线也绿 → 测试没抓住 bug（测试写错或 bug 本不存在）
  fails_at_candidate  候选也红 → 修复未生效
  flaky_at_base       基线多次结果不定 → 无结论（--repeat 才可能触发）
  tampered            运行期间测试文件被改写 → 无结论
  red_without_symptom 基线红但不命中 symptom → 视为红（exit 0）但告警复核
exit 0 = shows_the_bug（或 red_without_symptom）；1 = 判定为未锁定/未修复；2 = 无结论或用法错误。

注意：基线 worktree 建在系统临时目录，跑完即删（--keep 保留排查）。
测试命令在基线 worktree 与当前工作区各跑一次，测试文件以候选版本为准（并做 sha256 防篡改）。
"""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

TEST_NAME = re.compile(
    r"[._-](test|spec)\.[cm]?[jt]sx?$|(^|/)test_[^/]+\.py$|_test\.(py|go|rb)$|Tests?\.(java|kt|cs)$",
    re.I)
TEST_DIR = re.compile(r"(^|/)(tests?|__tests__|specs?|spec|testing|e2e)(/|$)", re.I)


def run(cmd, cwd, timeout):
    start = time.time()
    try:
        p = subprocess.run(cmd, shell=True, cwd=str(cwd), capture_output=True,
                           text=True, errors="replace", timeout=timeout)
        return p.returncode, (p.stdout or "") + (p.stderr or ""), time.time() - start
    except subprocess.TimeoutExpired:
        return 124, f"(超时 {timeout}s)", time.time() - start


def git(args, cwd):
    p = subprocess.run(["git", *args], cwd=str(cwd), capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} 失败: {p.stderr.strip()}")
    return p.stdout


def sha(path: Path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def discover_changed_tests(root: Path, base_ref: str):
    """基线以来变更 + 未跟踪的文件里挑测试文件。"""
    changed = set()
    for line in git(["diff", "--name-only", base_ref, "--"], root).splitlines():
        if line.strip():
            changed.add(line.strip())
    for line in git(["ls-files", "--others", "--exclude-standard"], root).splitlines():
        if line.strip():
            changed.add(line.strip())
    picked = [p for p in sorted(changed)
              if TEST_NAME.search(p) or (TEST_DIR.search(p) and p.endswith(".py"))]
    return picked


def collect(args, root: Path):
    if args.test_path:
        files = []
        for raw in args.test_path:
            p = (root / raw).resolve()
            if p.is_dir():
                files.extend(str(f.relative_to(root)) for f in p.rglob("*")
                             if f.is_file() and (TEST_NAME.search(str(f)) or f.suffix in (".py", ".rs")))
            elif p.is_file():
                files.append(raw)
            else:
                raise SystemExit(f"--test-path 不存在: {raw}")
        return sorted(set(files))
    if args.test_glob:
        return sorted(str(f.relative_to(root)) for f in root.glob(args.test_glob)
                      if f.is_file() and TEST_NAME.search(str(f)))
    found = discover_changed_tests(root, args.base_ref)
    if not found:
        raise SystemExit("未发现变更的测试文件——用 --test-path 或 --test-glob 显式指定")
    return found


def main() -> int:
    ap = argparse.ArgumentParser(description="验证新测试是否在基线版本上复现 bug",
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base-ref", default="HEAD", help="基线版本（默认 HEAD；修复未提交时即 pre-fix）")
    ap.add_argument("--test-cmd", required=True, help="测试命令（在仓库根执行）")
    ap.add_argument("--test-path", action="append", help="测试文件/目录（可重复；默认自动发现变更测试）")
    ap.add_argument("--test-glob", help="用 glob 选测试（如 'tests/**/test_*.py'）")
    ap.add_argument("--extra-path", action="append", default=[],
                    help="一并冻结到基线的辅助文件（可重复）")
    ap.add_argument("--symptom", help="基线失败消息应匹配的正则（如 'AssertionError|422|Traceback'）")
    ap.add_argument("--repeat", type=int, default=1, help="基线运行次数（>1 可检出 flaky，默认 1）")
    ap.add_argument("--timeout", type=int, default=300, help="单次运行超时秒数（默认 300）")
    ap.add_argument("--keep", action="store_true", help="保留基线 worktree（排查用，路径会打印）")
    ap.add_argument("--json", action="store_true", help="输出 JSON")
    args = ap.parse_args()

    root = Path(git(["rev-parse", "--show-toplevel"], Path.cwd()).strip())
    base_sha = git(["rev-parse", "--short", args.base_ref], root).strip()

    try:
        test_files = collect(args, root)
    except SystemExit as e:
        print(e, file=sys.stderr)
        return 2

    frozen = test_files + [p for p in args.extra_path]
    for rel in frozen:
        if not (root / rel).is_file():
            print(f"冻结源不存在: {rel}", file=sys.stderr)
            return 2
    hashes = {rel: sha(root / rel) for rel in frozen}

    tmp = Path(tempfile.mkdtemp(prefix="dj-regression-base-"))
    worktree = tmp / "base"
    verdict, reason, base_runs, cand_rc, cand_out = None, "", [], None, ""
    try:
        git(["worktree", "add", "--detach", str(worktree), args.base_ref], root)
        existed = {rel: (worktree / rel).exists() for rel in frozen}  # 复制前判新/改
        # 冻结：候选测试文件覆盖到基线 worktree（保持相对路径）
        for rel in frozen:
            dst = worktree / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(root / rel, dst)

        print(f"基线 {args.base_ref} ({base_sha}) | 冻结 {len(frozen)} 个文件到临时 worktree")
        for rel in frozen:
            print(f"  · {rel}{'' if existed[rel] else '  [新增于候选]'}")

        for i in range(max(1, args.repeat)):
            rc, out, secs = run(args.test_cmd, worktree, args.timeout)
            base_runs.append({"rc": rc, "output": out, "seconds": round(secs, 1)})
            state = "通过" if rc == 0 else "失败"
            print(f"基线运行 {i + 1}/{args.repeat}: {state} (rc={rc}, {secs:.1f}s)")

        # 防篡改：运行后测试文件不得被改写
        for rel in frozen:
            if sha(worktree / rel) != hashes[rel]:
                verdict, reason = "tampered", f"运行期间 {rel} 被改写（测试自改让结果失真）"

        if verdict is None:
            rcs = [r["rc"] for r in base_runs]
            if len(set(rcs)) > 1:
                verdict, reason = "flaky_at_base", f"基线重复运行结果不定: {rcs}"
            elif rcs[0] == 0:
                verdict, reason = "green_at_base", "基线通过——这些测试没有抓住 bug"
            else:
                cand_rc, cand_out, cand_secs = run(args.test_cmd, root, args.timeout)
                print(f"候选运行: {'通过' if cand_rc == 0 else '失败'} (rc={cand_rc}, {cand_secs:.1f}s)")
                if cand_rc != 0:
                    verdict, reason = "fails_at_candidate", "候选仍失败——修复未生效"
                elif args.symptom and not re.search(args.symptom, base_runs[0]["output"], re.I):
                    verdict, reason = "red_without_symptom", \
                        "基线红但输出未命中 --symptom——确认失败原因确是此 bug"
                else:
                    verdict, reason = "shows_the_bug", "基线红、候选绿——修复被测试锁定"

        if not args.json:
            print("\n判定:", verdict)
            print("  " + reason)
            if verdict == "red_without_symptom" and base_runs:
                tail = "\n".join(base_runs[0]["output"].splitlines()[-8:])
                print("  基线失败输出尾部:\n    " + tail.replace("\n", "\n    "))
            if cand_rc not in (None, 0) and cand_out:
                tail = "\n".join(cand_out.splitlines()[-8:])
                print("  候选失败输出尾部:\n    " + tail.replace("\n", "\n    "))
        else:
            print(json.dumps({
                "base_ref": args.base_ref, "base_sha": base_sha,
                "frozen_files": frozen, "verdict": verdict, "reason": reason,
                "base_runs": [{"rc": r["rc"], "seconds": r["seconds"]} for r in base_runs],
                "candidate_rc": cand_rc,
                "candidate_tail": cand_out.splitlines()[-8:] if cand_out else [],
            }, ensure_ascii=False, indent=2))

        return 2 if verdict in ("tampered", "flaky_at_base") else (0 if verdict in
                                                                  ("shows_the_bug", "red_without_symptom") else 1)
    finally:
        try:
            git(["worktree", "remove", "--force", str(worktree)], root)
        except RuntimeError:
            pass
        if args.keep:
            print(f"（--keep）基线 worktree 残留: {worktree}")
        else:
            shutil.rmtree(tmp, ignore_errors=True)
            subprocess.run(["git", "worktree", "prune"], cwd=str(root), capture_output=True)


if __name__ == "__main__":
    sys.exit(main())