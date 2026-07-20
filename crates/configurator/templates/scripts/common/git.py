#!/usr/bin/env python3
"""
Git operation helpers for DiJiang.

Provides safe, staged commit and add operations that respect
the ``session_auto_commit`` config flag.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from .paths import get_repo_root


# ── Helpers ──────────────────────────────────────────────────────────────

def git(*args: str, cwd: Path | None = None) -> subprocess.CompletedProcess:
    """Run a git command and return the result."""
    try:
        return subprocess.run(
            ["git"] + list(args),
            cwd=cwd,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        print("Error: git not found in PATH.", file=sys.stderr)
        raise SystemExit(1)


def check_git_available(cwd: Path | None = None) -> bool:
    """Check if git is available and the current dir is a git repo."""
    result = git("rev-parse", "--git-dir", cwd=cwd)
    return result.returncode == 0


# ── Status ───────────────────────────────────────────────────────────────

def get_status(cwd: Path | None = None) -> str:
    """Return ``git status --short --branch``."""
    result = git("status", "--short", "--branch", cwd=cwd)
    return result.stdout.strip()


def get_diff(cwd: Path | None = None) -> str:
    """Return ``git diff`` (unstaged changes)."""
    result = git("diff", cwd=cwd)
    return result.stdout.strip()


def get_diff_staged(cwd: Path | None = None) -> str:
    """Return ``git diff --staged`` (staged changes)."""
    result = git("diff", "--staged", cwd=cwd)
    return result.stdout.strip()


def get_changed_files(cwd: Path | None = None) -> list[str]:
    """Return list of changed (unstaged + untracked) file paths."""
    result = git("status", "--porcelain", cwd=cwd)
    if result.returncode != 0 or not result.stdout.strip():
        return []
    files: list[str] = []
    for line in result.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        # porcelain format: XY filename
        files.append(line[3:].strip())
    return files


def get_staged_files(cwd: Path | None = None) -> list[str]:
    """Return list of staged file paths."""
    result = git("diff", "--staged", "--name-only", cwd=cwd)
    if result.returncode != 0 or not result.stdout.strip():
        return []
    return result.stdout.splitlines()


# ── Add / Commit ─────────────────────────────────────────────────────────

def add_files(files: list[str], *, force: bool = False, cwd: Path | None = None) -> int:
    """Stage specific files for commit.

    Shows what will be added before executing.
    When *force* is True, use ``git add -f`` (for gitignored files).

    Returns 0 on success, 1 on error.
    """
    if not files:
        print("No files specified. Use '.' or a file path.")
        return 1

    # Show what will be added
    print("Files to stage:")
    for f in files:
        indicator = " [FORCED]" if force else ""
        print(f"  + {f}{indicator}")

    # Confirm
    cmd = ["add", "-f" if force else "-N"] if force else ["add"]
    cmd.extend(files)
    result = git(*cmd, cwd=cwd)
    if result.returncode != 0:
        print(f"Error: git add failed:\n{result.stderr.strip()}")
        return 1
    return 0


def add_all(*, dry_run: bool = False, cwd: Path | None = None) -> int:
    """Stage all changed files with preview.

    When *dry_run* is True, only show what would be staged.
    """
    changed = get_changed_files(cwd)
    if not changed:
        print("No changes to stage.")
        return 0

    print("Changes staged:")
    unstaged = get_status(cwd)
    print(unstaged)

    if dry_run:
        print("\n(Dry run — no changes made)")
        return 0

    result = git("add", "-A", cwd=cwd)
    if result.returncode != 0:
        print(f"git add failed:\n{result.stderr.strip()}")
        return 1
    return 0


def commit(message: str, *, allow_empty: bool = False, cwd: Path | None = None) -> int:
    """Create a commit with the given message.

    Returns 0 on success, 1 if nothing to commit or error.
    """
    staged = get_staged_files(cwd)
    if not staged:
        if allow_empty:
            result = git("commit", "--allow-empty", "-m", message, cwd=cwd)
        else:
            print("Nothing to commit (stage files first with `git add`).")
            return 1

    result = git("commit", "-m", message, cwd=cwd)
    if result.returncode != 0:
        print(f"git commit failed:\n{result.stderr.strip()}")
        return 1
    print(f"Committed: {message}")
    return 0
