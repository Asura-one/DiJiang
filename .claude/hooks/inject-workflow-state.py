#!/usr/bin/env python3
from __future__ import annotations

import json
import os
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


def visible_error(message: str) -> str:
    session = (
        os.environ.get("DIJIANG_CONTEXT_ID")
        or os.environ.get("CLAUDE_SESSION_ID")
        or os.environ.get("CLAUDE_CODE_SESSION_ID")
        or "unknown"
    )
    context = "\n".join(
        [
            "<dijiang-workflow-state>",
            "Platform: claude",
            f"Session hint: {session}",
            f"Hook error: {message}",
            "Active task: unknown",
            "Next: run `dijiang workflow-state` from the project root and check that `dijiang` is on PATH.",
            "</dijiang-workflow-state>",
        ]
    )
    return json.dumps({"hookEventName": "UserPromptSubmit", "additionalContext": context})


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
    except FileNotFoundError:
        print(visible_error("dijiang executable not found"))
        return 0
    except subprocess.TimeoutExpired:
        print(visible_error("dijiang workflow-state timed out"))
        return 0
    except subprocess.SubprocessError as exc:
        print(visible_error(str(exc)))
        return 0

    if result.returncode == 0 and result.stdout.strip():
        print(result.stdout.strip())
    elif result.returncode != 0:
        detail = (result.stderr or result.stdout or f"exit code {result.returncode}").strip()
        print(visible_error(detail))
    return 0


if __name__ == "__main__":
    sys.exit(main())
