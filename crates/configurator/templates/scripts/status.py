#!/usr/bin/env python3
"""
DiJiang status — project status query.

Usage:
    python3 .dijiang/scripts/status.py                          # Basic status
    python3 .dijiang/scripts/status.py --compat                 # With compatibility diagnostics
    python3 .dijiang/scripts/status.py --json                   # JSON output

Reads project state from `.dijiang/` and task files without requiring the
`dijiang` binary in PATH.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from common.paths import get_repo_root, get_tasks_dir, get_config_path
from common.config import load_config
from common.tasks import load_task
from common.git import git, get_status as get_git_status
from common.io import read_json


# ── Helpers ──────────────────────────────────────────────────────────────

def read_project_name(root: Path) -> str:
    """Read project name from config.toml or repo dir name."""
    config = load_config(root)
    if config.project_name:
        return config.project_name
    return root.name


def read_active_task(root: Path) -> str | None:
    """Read current active task from session file."""
    session_dir = root / ".dijiang" / ".runtime" / "sessions"
    if not session_dir.is_dir():
        return None
    for f in sorted(session_dir.iterdir()):
        if f.suffix == ".json":
            try:
                session = json.loads(f.read_text())
                task = session.get("current_task")
                if task:
                    return task
            except (json.JSONDecodeError, OSError):
                continue
    return None


def list_tasks(root: Path) -> list[dict]:
    """List all tasks from the tasks directory."""
    tasks_dir = get_tasks_dir(root)
    if not tasks_dir.is_dir():
        return []
    tasks: list[dict] = []
    for entry in sorted(tasks_dir.iterdir()):
        if not entry.is_dir() or entry.name.startswith("."):
            continue
        task_json = entry / "task.json"
        if task_json.is_file():
            try:
                data = read_json(task_json)
                if isinstance(data, dict):
                    tasks.append({
                        "name": entry.name,
                        "status": data.get("status", "?"),
                        "title": data.get("title", data.get("name", entry.name)),
                        "priority": data.get("priority", ""),
                    })
            except Exception:
                tasks.append({
                    "name": entry.name,
                    "status": "?",
                    "title": entry.name,
                    "priority": "",
                })
    return tasks


# ── Commands ─────────────────────────────────────────────────────────────

def cmd_status(compat: bool = False, json_output: bool = False) -> dict:
    """Build a status report."""
    root = get_repo_root()
    name = read_project_name(root)
    active = read_active_task(root)
    tasks = list_tasks(root)
    git_status = get_git_status(root)

    has_pi = (root / ".pi").is_dir()

    result = {
        "project": name,
        "active_task": active,
        "total_tasks": len(tasks),
        "tasks": [
            {
                "name": t["name"],
                "status": t["status"],
                "title": t["title"],
                "active": t["name"] == active,
            }
            for t in tasks
        ],
        "pi_configured": has_pi,
        "git": git_status,
    }

    if compat:
        result["compatibility"] = {
            "status_mapping": {
                "planning": "plan",
                "in_progress": "implement",
                "completed": "complete",
                "paused": "in_progress (downgraded)",
                "archived": "complete (downgraded)",
            },
        }

    if json_output:
        return result

    # Human-readable output
    print(f"\n  ── DiJiang Status ──\n")
    print(f"  {'项目:':<15} {name}")
    if active:
        print(f"  {'当前任务:':<15} {active}")
        for t in tasks:
            if t["name"] == active:
                print(f"  {'状态:':<15} {t['status']}")
    else:
        print(f"  {'当前任务:':<15} (none)")
    print(f"  任务 ({len(tasks)}):")
    for t in tasks:
        marker = "*" if t["name"] == active else " "
        status = t["status"]
        title = t["title"]
        print(f"    {marker} {t['name']:<45} {status:<12}")
    if has_pi:
        print(f"  Pi:              ✓ configured")
    print(f"  Git:             {git_status.split(chr(10))[0] if git_status else '?'}")

    if compat:
        print(f"  ── Compatibility Diagnostics ──")
        print(f"  Status mapping (DiJiang → Trellis):")
        for dij, tre in result["compatibility"]["status_mapping"].items():
            print(f"    {dij:<20} → {tre}")

    print()
    return result


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description="DiJiang status — project state query")
    parser.add_argument("--compat", action="store_true", help="Show compatibility diagnostics")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    args = parser.parse_args()

    result = cmd_status(compat=args.compat, json_output=args.json)
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
