#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def find_dijiang_root(start: Path) -> Path | None:
    current = start.resolve()
    while True:
        if (current / ".dijiang").is_dir():
            return current
        if current == current.parent:
            return None
        current = current.parent


def main() -> int:
    root = find_dijiang_root(Path.cwd())
    if root is None:
        return 0

    try:
        stdin_data = sys.stdin.read()
    except OSError:
        stdin_data = ""

    try:
        result = subprocess.run(
            ["dijiang", "workflow-state", "--json", "--hook-event", "UserPromptSubmit"],
            input=stdin_data,
            text=True,
            cwd=root,
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError):
        return 0

    if result.returncode == 0 and result.stdout.strip():
        print(result.stdout.strip())
    return 0


if __name__ == "__main__":
    sys.exit(main())
