#!/usr/bin/env python3
"""
DiJiang task helper — CLI for task lifecycle operations.

Usage:
    python3 .dijiang/scripts/task.py current                  # Show active task
    python3 .dijiang/scripts/task.py list                      # List all tasks
    python3 .dijiang/scripts/task.py active                    # List non-archived tasks
    python3 .dijiang/scripts/task.py show <id>                 # Show task details
    python3 .dijiang/scripts/task.py find <status>             # Find tasks by status

    # JSONL context (P1)
    python3 .dijiang/scripts/task.py add-context <id> <name> <path> [reason]
    python3 .dijiang/scripts/task.py list-context <id>
    python3 .dijiang/scripts/task.py validate-context <id>
    python3 .dijiang/scripts/task.py seed-context <id>

    # Task hierarchy (P1)
    python3 .dijiang/scripts/task.py add-subtask <parent-id> <child-id>
    python3 .dijiang/scripts/task.py tree <id>
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

from common.paths import get_repo_root, get_task_dir, get_task_json, get_tasks_dir
from common.tasks import (
    load_task,
    write_task,
    iter_active_tasks,
    iter_all_tasks,
    find_task_by_status,
    is_valid_status,
    slugify,
    task_exists,
)
from common.types import TaskData, VALID_STATUSES
from common.io import read_json, write_json


# ── Display helpers ──────────────────────────────────────────────────────

def fmt_task(tid: str, td: TaskData, *, indent: int = 0) -> str:
    pfx = " " * indent
    status = td.get("status", "?")
    title = td.get("title", td.get("name", tid))
    priority = td.get("priority", "")
    pkg = td.get("package", "")
    parts = [f"{pfx}{tid}  ({status})"]
    if title != tid:
        parts.append(f"{pfx}  title:    {title}")
    if priority:
        parts.append(f"{pfx}  priority: {priority}")
    if pkg:
        parts.append(f"{pfx}  package:  {pkg}")
    return "\n".join(parts)


def fmt_status_badge(status: str) -> str:
    badges = {
        "planning": "🟡 planning",
        "in_progress": "🟢 in_progress",
        "completed": "🔵 completed",
        "archived": "⚪ archived",
        "paused": "🟠 paused",
    }
    return badges.get(status, status)


def show_hierarchy(td: TaskData, root: Path) -> str:
    """Build a human-readable hierarchy line from task data."""
    parts = []
    parent = td.get("parent")
    if parent:
        parts.append(f"  Parent:   {parent}")
    children = td.get("children", [])
    if children:
        parts.append(f"  Children: {', '.join(children)}")
    subtasks = td.get("subtasks", [])
    if subtasks:
        parts.append(f"  Subtasks: {', '.join(subtasks)}")
    return "\n".join(parts)


# ── Commands ─────────────────────────────────────────────────────────────────

def cmd_current(root: Path) -> int:
    """Show active task."""
    active = iter_active_tasks(root)
    in_progress = [t for t in active if t[1].get("status") == "in_progress"]
    planning = [t for t in active if t[1].get("status") == "planning"]

    target = in_progress or planning or active
    if not target:
        print("No active task.")
        return 0

    tid, td = target[0]
    print(f"Active task: {tid}")
    print(f"  Status:   {fmt_status_badge(td.get('status', '?'))}")
    print(f"  Title:    {td.get('title', tid)}")
    print(f"  Priority: {td.get('priority', '-')}")
    if td.get("package"):
        print(f"  Package:  {td.get('package')}")
    if td.get("branch"):
        print(f"  Branch:   {td.get('branch')}")

    hier = show_hierarchy(td, root)
    if hier.strip():
        print(hier)

    if td.get("description"):
        print(f"  Desc:     {td.get('description')}")

    if len(target) > 1:
        print(f"\nOther active ({len(target) - 1}):")
        for t in target[1:]:
            print(f"  {t[0]}  ({t[1].get('status', '?')})")
    return 0


def cmd_list(root: Path) -> int:
    """List all tasks grouped by status."""
    all_tasks = iter_all_tasks(root)
    groups = {}
    for tid, td in all_tasks:
        s = td.get("status", "?")
        groups.setdefault(s, []).append((tid, td))

    status_order = ["in_progress", "planning", "completed", "archived", "paused"]
    for status in status_order:
        items = groups.pop(status, [])
        if items:
            print(f"\n[{fmt_status_badge(status)}]  ({len(items)})")
            for tid, td in items:
                children = td.get("children", [])
                suffix = f" [{', '.join(children)}]" if children else ""
                print(f"  {tid}{suffix}")

    for status, items in sorted(groups.items()):
        print(f"\n[{status}]  ({len(items)})")
        for tid, _ in items:
            print(f"  {tid}")
    return 0


def cmd_active(root: Path) -> int:
    """List non-archived tasks."""
    tasks = iter_active_tasks(root)
    if not tasks:
        print("No active tasks.")
        return 0
    print(f"Active tasks ({len(tasks)}):")
    for tid, td in tasks:
        print(f"  {tid}  ({td.get('status', '?')})  {td.get('title', '')}")
    return 0


def cmd_show(root: Path, task_id: str) -> int:
    """Show full details for a task."""
    td = load_task(task_id, root)
    if td is None:
        print(f"Task '{task_id}' not found.")
        return 1
    # Include hierarchy info inline
    hier = show_hierarchy(td, root)
    data = dict(td)  # make a mutable copy
    print(json.dumps(data, indent=2, ensure_ascii=False))
    return 0


def cmd_find(root: Path, status: str) -> int:
    """Find tasks by status."""
    status = status.strip().lower()
    if not is_valid_status(status):
        valid = ", ".join(sorted(VALID_STATUSES))
        print(f"Invalid status '{status}'. Valid: {valid}")
        return 1
    tasks = find_task_by_status(status, root)
    if not tasks:
        print(f"No tasks with status '{status}'.")
        return 0
    print(f"Tasks with status '{status}' ({len(tasks)}):")
    for tid, td in tasks:
        print(f"  {tid}  {td.get('title', '')}")
    return 0


# ── Context commands (P1) ────────────────────────────────────────────────

def cmd_add_context(root: Path, args: list[str]) -> int:
    """add-context <task-id> <jsonl-name> <path> [reason]"""
    if len(args) < 3:
        print("Usage: task.py add-context <task-id> <jsonl-name> <path> [reason]")
        return 1
    task_id = args[0]
    jsonl_name = args[1]
    path = args[2]
    reason = " ".join(args[3:]) if len(args) > 3 else ""

    if not task_exists(task_id, root):
        print(f"Task '{task_id}' not found.")
        return 1

    from common.context import add_context_entry
    return add_context_entry(task_id, jsonl_name, path, reason, root)


def cmd_list_context(root: Path, args: list[str]) -> int:
    """list-context <task-id>"""
    if len(args) < 1:
        print("Usage: task.py list-context <task-id>")
        return 1
    task_id = args[0]
    if not task_exists(task_id, root):
        print(f"Task '{task_id}' not found.")
        return 1
    from common.context import list_context_task
    return list_context_task(task_id, root)


def cmd_validate_context(root: Path, args: list[str]) -> int:
    """validate-context <task-id>"""
    if len(args) < 1:
        print("Usage: task.py validate-context <task-id>")
        return 1
    task_id = args[0]
    if not task_exists(task_id, root):
        print(f"Task '{task_id}' not found.")
        return 1
    from common.context import validate_context_task
    return validate_context_task(task_id, root)


def cmd_seed_context(root: Path, args: list[str]) -> int:
    """seed-context <task-id>"""
    if len(args) < 1:
        print("Usage: task.py seed-context <task-id>")
        return 1
    task_id = args[0]
    if not task_exists(task_id, root):
        print(f"Task '{task_id}' not found.")
        return 1
    from common.context import seed_context
    return seed_context(task_id, root)


# ── Hierarchy commands (P1) ──────────────────────────────────────────────

def cmd_add_subtask(root: Path, args: list[str]) -> int:
    """add-subtask <parent-id> <child-id>"""
    if len(args) < 2:
        print("Usage: task.py add-subtask <parent-id> <child-id>")
        return 1
    parent_id = args[0]
    child_id = args[1]

    if not task_exists(parent_id, root):
        print(f"Parent task '{parent_id}' not found.")
        return 1
    if not task_exists(child_id, root):
        print(f"Child task '{child_id}' not found.")
        return 1
    if parent_id == child_id:
        print("Error: a task cannot be its own parent.")
        return 1

    # Update parent
    parent_td = load_task(parent_id, root)
    if parent_td is None:
        print(f"Error: could not load task '{parent_id}'.")
        return 1
    children = parent_td.get("children", [])
    if child_id not in children:
        children.append(child_id)
        parent_td["children"] = children
        write_task(parent_id, parent_td, root)

    # Update child
    child_td = load_task(child_id, root)
    if child_td is None:
        print(f"Error: could not load task '{child_id}'.")
        return 1
    child_td["parent"] = parent_id
    write_task(child_id, child_td, root)

    print(f"Added subtask: {child_id} → {parent_id}")
    return 0


def cmd_tree(root: Path, args: list[str]) -> int:
    """tree <task-id> — show task subtree."""
    if len(args) < 1:
        print("Usage: task.py tree <task-id>")
        return 1
    task_id = args[0]
    td = load_task(task_id, root)
    if td is None:
        print(f"Task '{task_id}' not found.")
        return 1

    def _print_tree(node_id: str, depth: int = 0) -> None:
        prefix = "  " * depth + "└─ " if depth > 0 else ""
        node_td = load_task(node_id, root)
        status = node_td.get("status", "?") if node_td else "?"
        print(f"{prefix}{node_id}  ({status})")
        if node_td:
            for child_id in node_td.get("children", []):
                _print_tree(child_id, depth + 1)

    _print_tree(task_id)
    return 0


# ── Main ─────────────────────────────────────────────────────────────────────

def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(
        description="DiJiang task helper",
        usage="%(prog)s <command> [options]",
    )
    parser.add_argument("command", nargs="?", default="current")
    parser.add_argument("args", nargs="*", default=[],
                        help="Arguments for the command")
    args = parser.parse_args()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    cmd = args.command
    extra = args.args

    # Simple commands (no extra args needed or positional-only)
    if cmd == "current":
        return cmd_current(root)
    elif cmd == "list":
        return cmd_list(root)
    elif cmd == "active":
        return cmd_active(root)

    # Commands with single positional required arg
    elif cmd == "show":
        if not extra:
            print("Usage: task.py show <task-id>")
            return 1
        return cmd_show(root, extra[0])
    elif cmd == "find":
        if not extra:
            print("Usage: task.py find <status>")
            return 1
        return cmd_find(root, extra[0])

    # P1: Context commands
    elif cmd == "add-context":
        return cmd_add_context(root, extra)
    elif cmd == "list-context":
        return cmd_list_context(root, extra)
    elif cmd == "validate-context":
        return cmd_validate_context(root, extra)
    elif cmd == "seed-context":
        return cmd_seed_context(root, extra)

    # P1: Hierarchy commands
    elif cmd == "add-subtask":
        return cmd_add_subtask(root, extra)
    elif cmd == "tree":
        return cmd_tree(root, extra)

    else:
        print(f"Unknown command: {cmd}")
        print("Commands: current, list, active, show, find")
        print("          add-context, list-context, validate-context, seed-context")
        print("          add-subtask, tree")
        return 1


if __name__ == "__main__":
    sys.exit(main())
