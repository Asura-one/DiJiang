#!/usr/bin/env python3
"""
Safe commit helper — staged, controlled git operations.

Respects ``session_auto_commit`` from ``.dijiang/config.toml``.
Automatically stages all files when the flag is enabled (after preview).

Usage:
    python3 .dijiang/scripts/safe_commit.py status          # Show repo state
    python3 .dijiang/scripts/safe_commit.py add <file>...   # Stage specific files
    python3 .dijiang/scripts/safe_commit.py add-all         # Stage all (respects auto_commit)
    python3 .dijiang/scripts/safe_commit.py commit <msg>    # Commit staged changes
    python3 .dijiang/scripts/safe_commit.py auto "<msg>"    # add-all + commit in one step
"""

from __future__ import annotations

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from common.git import (
    check_git_available,
    get_status,
    get_diff,
    get_diff_staged,
    get_changed_files,
    get_staged_files,
    add_files,
    add_all,
    commit,
)
from common.paths import get_repo_root
from common.config import load_config


def cmd_status(root: str) -> int:
    """Show repo state."""
    status = get_status(root)
    staged = get_staged_files(root)
    diff = get_diff(root)
    diff_staged = get_diff_staged(root)

    print("Git status:")
    print(status or "  (clean)")
    print()

    if staged:
        print(f"Staged ({len(staged)}):")
        for f in staged:
            print(f"  {f}")
        print()
    if diff:
        print(f"Unstaged diff ({len(diff.splitlines())} lines):")
        # Show first 20 lines
        for line in diff.splitlines()[:20]:
            print(f"  {line}")
        lines_total = len(diff.splitlines())
        if lines_total > 20:
            print(f"  ... ({lines_total - 20} more lines)")
        print()
    if diff_staged:
        print(f"Staged diff ({len(diff_staged.splitlines())} lines):")
        for line in diff_staged.splitlines()[:20]:
            print(f"  {line}")
        lines_total = len(diff_staged.splitlines())
        if lines_total > 20:
            print(f"  ... ({lines_total - 20} more lines)")
    return 0


def cmd_add(files: list[str], root: str) -> int:
    """Stage specific files."""
    if not files:
        print("Usage: safe_commit.py add <file> [file...]")
        return 1
    return add_files(files, cwd=root)


def cmd_add_all(root: str) -> int:
    """Stage all changes (respects auto_commit flag)."""
    cfg = load_config(root)
    auto_commit = cfg.session_auto_commit

    print(f"Auto-commit config: {'enabled' if auto_commit else 'disabled'}")
    if auto_commit:
        return add_all(cwd=root)
    else:
        print("Auto-commit is disabled. Doing dry run only.")
        return add_all(dry_run=True, cwd=root)


def cmd_commit(root: str, msg: str) -> int:
    """Commit staged changes."""
    if not msg:
        print("Usage: safe_commit.py commit <message>")
        return 1
    return commit(msg, cwd=root)


def cmd_auto(root: str, msg: str) -> int:
    """add-all + commit in one step."""
    cfg = load_config(root)
    auto_commit = cfg.session_auto_commit

    if not auto_commit:
        print("Auto-commit is disabled in config. Use explicit add + commit instead.")
        print(f"  python3 .dijiang/scripts/safe_commit.py add <file>")
        print(f"  python3 .dijiang/scripts/safe_commit.py commit '{msg}'")
        return 1

    if not msg:
        print("Usage: safe_commit.py auto <commit-message>")
        return 1

    # Stage all
    r1 = add_all(cwd=root)
    if r1 != 0:
        return r1

    # Commit
    return commit(msg, cwd=root)


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(
        description="Safe commit helper",
    )
    parser.add_argument("command", nargs="?", default="status",
                        choices=["status", "add", "add-all", "commit", "auto"])
    parser.add_argument("args", nargs="*", default=[],
                        help="Arguments for subcommand")

    args = parser.parse_args()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    if not check_git_available(root):
        print("Error: not a git repository.")
        return 1

    cmd = args.command
    extra = args.args

    if cmd == "status":
        return cmd_status(root)
    elif cmd == "add":
        return cmd_add(extra, root)
    elif cmd == "add-all":
        return cmd_add_all(root)
    elif cmd == "commit":
        return cmd_commit(root, " ".join(extra) if extra else "")
    elif cmd == "auto":
        return cmd_auto(root, " ".join(extra) if extra else "")
    else:
        print(f"Unknown command: {cmd}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
