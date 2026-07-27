#!/usr/bin/env python3
"""Compatibility wrapper for the Rust-owned DiJiang dispatch registry.

The `dijiang dispatch` command owns prompt classification, skill metadata, and
route-gate evaluation. Keeping this script as a thin launcher prevents its
legacy Python keyword table from drifting from the runtime registry.
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="DiJiang dispatch compatibility wrapper")
    parser.add_argument("prompt", nargs="?", default="", help="User prompt to classify")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of text")
    parser.add_argument("--force-new", action="store_true", help="Create a new task despite an active task")
    parser.add_argument("--active-task", help="Retained for CLI compatibility; Rust dispatch uses the active session task")
    parser.add_argument("--hook-event", default="UserPromptSubmit", help="Hook event name")
    args = parser.parse_args()

    prompt = args.prompt
    if not prompt and not sys.stdin.isatty():
        prompt = sys.stdin.read().strip()
    if not prompt:
        parser.error("a prompt is required")

    executable = shutil.which("dijiang")
    if executable is None:
        print("dijiang executable was not found on PATH", file=sys.stderr)
        return 127

    command = [
        executable,
        "dispatch",
        prompt,
        "--hook-event",
        args.hook_event,
        "--classify-only",
    ]
    if args.active_task:
        command.extend(["--active-task", args.active_task])
    if args.json:
        command.append("--json")
    if args.force_new:
        command.append("--force-new")
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
