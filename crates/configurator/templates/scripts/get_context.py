#!/usr/bin/env python3
"""
Context summary for AI agent injection.

Outputs a concise snapshot of the DiJiang project state: git status, active
task, spec index, and workflow phase. Intended for hook scripts and manual
context loading.

Usage:
    python3 .dijiang/scripts/get_context.py                  # Full summary
    python3 .dijiang/scripts/get_context.py --mode phase     # Only workflow phase
    python3 .dijiang/scripts/get_context.py --mode task      # Only active task
    python3 .dijiang/scripts/get_context.py --mode git       # Only git status
    python3 .dijiang/scripts/get_context.py --json           # JSON output
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

from common.paths import (
    DIR_DIJIANG,
    get_repo_root,
    get_workspace_dir,
    get_tasks_dir,
    get_spec_dir,
)
from common.tasks import iter_active_tasks
from common.config import load_config
from common.io import walk_markdown_files
from common.session import SessionIdentity


# =============================================================================
# Git helpers
# =============================================================================

def git_run(*args: str, cwd: Path | None = None) -> str:
    """Run a git command and return stdout. Returns empty string on error."""
    try:
        return subprocess.check_output(
            ["git"] + list(args),
            cwd=cwd,
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""


def get_git_summary(root: Path) -> dict:
    """Return git branch, dirty state, and recent commits."""
    branch = git_run("rev-parse", "--abbrev-ref", "HEAD", cwd=root)
    dirty = git_run("status", "--porcelain", cwd=root)
    dirty_files = [l.strip() for l in dirty.split("\n") if l.strip()] if dirty else []

    return {
        "branch": branch or "unknown",
        "is_dirty": len(dirty_files) > 0,
        "dirty_count": len(dirty_files),
        "dirty_files": dirty_files[:20],  # cap at 20
    }


# =============================================================================
# Task helpers
# =============================================================================

def get_active_task_summary(root: Path) -> dict | None:
    """Summarise the current active task."""
    tasks = iter_active_tasks(root)
    # Prefer in_progress, then planning
    in_progress = [t for t in tasks if t[1].get("status") == "in_progress"]
    planning = [t for t in tasks if t[1].get("status") == "planning"]

    target = (in_progress or planning or [None])[0]
    if target is None:
        return None

    tid, td = target
    return {
        "id": tid,
        "title": td.get("title", tid),
        "status": td.get("status", "?"),
        "priority": td.get("priority", ""),
        "package": td.get("package", ""),
        "scope": td.get("scope", ""),
        "branch": td.get("branch", ""),
    }


# =============================================================================
# Spec helpers
# =============================================================================

def get_spec_summary(root: Path) -> dict:
    """Count spec files per category."""
    spec_dir = get_spec_dir(root)
    if not spec_dir.is_dir():
        return {"categories": {}, "total": 0}

    categories = {}
    for md_file in walk_markdown_files(spec_dir):
        rel = md_file.relative_to(spec_dir)
        parts = rel.parts
        if len(parts) >= 2:
            cat = parts[0]
        else:
            cat = "root"
        categories.setdefault(cat, 0)
        categories[cat] += 1

    total = sum(categories.values())
    return {"categories": categories, "total": total}


# =============================================================================
# Workflow phase from workflow.md
# =============================================================================

def get_workflow_phase(root: Path, task_status: str | None = None) -> str:
    """Return the workflow phase based on active task status.

    Maps the task status to a workflow phase:
      None        → "none"
      planning    → "planning"
      in_progress → "in_progress"
      completed   → "completed"
      archived    → "archived"
      paused      → "paused"
      other       → "none"
    """
    if task_status is None:
        return "none"
    mapping = {
        "planning": "planning",
        "in_progress": "in_progress",
        "completed": "completed",
        "archived": "archived",
        "paused": "paused",
    }
    return mapping.get(task_status, "none")


# =============================================================================
# Config summary
# =============================================================================

def get_config_summary(root: Path) -> dict:
    """Return project name and developer from config."""
    cfg = load_config(root)
    return {
        "project": cfg.project_name or "DiJiang",
        "developer": cfg.developer or "",
        "platforms": cfg.platforms,
        "session": SessionIdentity.detect().to_dict(),
    }


# =============================================================================
# Main
# =============================================================================

def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description="DiJiang context summary")
    parser.add_argument(
        "--mode", "-m",
        choices=["full", "phase", "task", "git", "spec", "config"],
        default="full",
        help="What to summarise (default: full)",
    )
    parser.add_argument("--json", "-j", action="store_true",
                        help="Output as JSON")
    args = parser.parse_args()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    output: dict = {}

    if args.mode in ("full", "git"):
        output["git"] = get_git_summary(root)
    if args.mode in ("full", "task"):
        task = get_active_task_summary(root)
        if task:
            output["task"] = task
    if args.mode in ("full", "spec"):
        output["spec"] = get_spec_summary(root)
    if args.mode in ("full", "phase"):
        task_data = get_active_task_summary(root)
        task_status = task_data.get("status") if task_data else None
        output["workflow_phase"] = get_workflow_phase(root, task_status)
    if args.mode in ("full", "config"):
        output["config"] = get_config_summary(root)

    if not output:
        print("No output generated.")
        return 0

    if args.json:
        print(json.dumps(output, indent=2, ensure_ascii=False))
    else:
        for section, data in output.items():
            if isinstance(data, dict):
                print(f"\n── {section} ──")
                for k, v in data.items():
                    if isinstance(v, list) and len(v) > 5:
                        print(f"  {k}: {len(v)} items")
                        for item in v[:5]:
                            print(f"    - {item}")
                        if len(v) > 5:
                            print(f"    ... and {len(v) - 5} more")
                    elif isinstance(v, dict):
                        print(f"  {k}:")
                        for sk, sv in v.items():
                            print(f"    {sk}: {sv}")
                    else:
                        print(f"  {k}: {v}")
            else:
                print(f"{section}: {data}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
