#!/usr/bin/env python3
"""
Cross-session memory retrieval for DiJiang.

Reads platform session logs (``.dijiang/workspace/<dev>/sessions/*.jsonl``)
and provides search/filter capabilities by task, date range, and keyword.
"""

from __future__ import annotations

import json
from datetime import date, timedelta
from pathlib import Path

from .paths import get_repo_root, get_workspace_dir

SESSIONS_DIR_NAME = "sessions"


# ── Session log entry ──────────────────────────────────────────────────


def get_session_logs_dir(root: Path | None = None) -> Path:
    """Return the sessions directory under the workspace.

    Scans all developer subdirectories for a ``sessions/`` folder.
    """
    workspace = get_workspace_dir(root)
    if not workspace.is_dir():
        return workspace / SESSIONS_DIR_NAME

    # Collect all sessions directories under workspace/<dev>/sessions/
    session_dirs: list[Path] = []
    for dev_dir in sorted(workspace.iterdir()):
        if dev_dir.is_dir() and not dev_dir.name.startswith("."):
            sess_dir = dev_dir / SESSIONS_DIR_NAME
            if sess_dir.is_dir():
                session_dirs.append(sess_dir)

    return session_dirs[0] if session_dirs else workspace / SESSIONS_DIR_NAME


def iter_logs(root: Path | None = None) -> list[dict]:
    """Iterate all session log entries from all JSONL files.

    Returns a list of parsed JSON dicts, newest first.
    """
    sess_dir = get_session_logs_dir(root)
    if not sess_dir.is_dir():
        return []

    entries: list[dict] = []
    for jsonl_file in sorted(sess_dir.glob("*.jsonl")):
        for line in jsonl_file.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
                entry["_source_file"] = jsonl_file.name
                entries.append(entry)
            except json.JSONDecodeError:
                continue

    # Sort newest first by closed_at or inserted order
    entries.sort(
        key=lambda e: e.get("closed_at", e.get("event", "")),
        reverse=True,
    )
    return entries


# ── Search / filter ───────────────────────────────────────────────────

def search_by_task(task_name: str, root: Path | None = None, limit: int = 10) -> list[dict]:
    """Find session logs related to a task by name.

    Performs case-insensitive substring match against the ``task`` field.
    """
    task_lower = task_name.lower()
    results: list[dict] = []
    for entry in iter_logs(root):
        task = entry.get("task", "")
        if task_lower in task.lower():
            results.append(entry)
            if len(results) >= limit:
                break
    return results


def search_by_keyword(keyword: str, root: Path | None = None, limit: int = 10) -> list[dict]:
    """Find session logs containing a keyword in summary or verification.

    Performs case-insensitive substring match.
    """
    kw_lower = keyword.lower()
    results: list[dict] = []
    for entry in iter_logs(root):
        summary = entry.get("summary", "")
        verification = entry.get("verification", "")
        if kw_lower in summary.lower() or kw_lower in verification.lower():
            results.append(entry)
            if len(results) >= limit:
                break
    return results


def search_by_date(
    days: int = 7,
    root: Path | None = None,
    limit: int = 20,
) -> list[dict]:
    """Find session logs from the last *days* days."""
    cutoff = date.today() - timedelta(days=days)
    results: list[dict] = []
    for entry in iter_logs(root):
        closed_at = entry.get("closed_at", "")
        if closed_at:
            entry_date = closed_at[:10]
            if entry_date >= cutoff.isoformat():
                results.append(entry)
                if len(results) >= limit:
                    break
    return results


def search_by_platform(
    platform: str,
    root: Path | None = None,
    limit: int = 10,
) -> list[dict]:
    """Find session logs by platform (source or session_key)."""
    plat_lower = platform.lower()
    results: list[dict] = []
    for entry in iter_logs(root):
        source = entry.get("source", "").lower()
        session_key = entry.get("session_key", "").lower()
        if plat_lower in source or plat_lower in session_key:
            results.append(entry)
            if len(results) >= limit:
                break
    return results


def search_by_event_type(
    event_type: str,
    root: Path | None = None,
    limit: int = 10,
) -> list[dict]:
    """Find session logs by event type (e.g. ``session_closed``, ``finding``)."""
    results: list[dict] = []
    for entry in iter_logs(root):
        if entry.get("event") == event_type:
            results.append(entry)
            if len(results) >= limit:
                break
    return results


# ── Display ────────────────────────────────────────────────────────────

def format_entry(entry: dict, index: int = 0) -> str:
    """Format a session log entry as human-readable text."""
    lines: list[str] = []
    prefix = f"[{index}] " if index else ""

    event = entry.get("event", "?")
    source = entry.get("source", "?")
    task = entry.get("task", "?")
    summary = entry.get("summary", "")
    closed_at = entry.get("closed_at", "")

    lines.append(f"{prefix}{event} [{source}] task={task}")
    if closed_at:
        lines.append(f"   at: {closed_at[:19]}")
    if summary:
        lines.append(f"   {summary[:120]}")
    return "\n".join(lines)


def format_detailed(entry: dict) -> str:
    """Format a session log entry with all available fields."""
    lines: list[str] = []
    for key, value in sorted(entry.items()):
        if key.startswith("_"):
            continue
        if isinstance(value, str) and len(value) > 200:
            value = value[:197] + "..."
        lines.append(f"  {key}: {value}")
    return "\n".join(lines)


# ── CLI entry point ────────────────────────────────────────────────────

def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(
        description="Cross-session memory retrieval",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Examples:\n"
            "  memory.py search keyword 'dispatch'\n"
            "  memory.py search task 'Trellis 吸收分析'\n"
            "  memory.py search date --days 14\n"
            "  memory.py search platform pi\n"
            "  memory.py show 3\n"
        ),
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # search
    search_p = sub.add_parser("search")
    search_p.add_argument("field", choices=["keyword", "task", "date", "platform", "event"],
                          help="Search field")
    search_p.add_argument("value", nargs="?", default="",
                          help="Search value (not needed for date)")
    search_p.add_argument("--days", type=int, default=7,
                          help="Days back (for date search)")
    search_p.add_argument("--limit", type=int, default=10,
                          help="Max results")

    # show
    show_p = sub.add_parser("show")
    show_p.add_argument("index", type=int, help="Entry index to show in detail")

    args = parser.parse_args()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    if args.command == "search":
        if args.field == "keyword":
            if not args.value:
                print("Error: keyword search requires a value")
                return 1
            results = search_by_keyword(args.value, root, limit=args.limit)
        elif args.field == "task":
            if not args.value:
                print("Error: task search requires a value")
                return 1
            results = search_by_task(args.value, root, limit=args.limit)
        elif args.field == "date":
            results = search_by_date(days=args.days, root=root, limit=args.limit)
        elif args.field == "platform":
            if not args.value:
                print("Error: platform search requires a value")
                return 1
            results = search_by_platform(args.value, root, limit=args.limit)
        elif args.field == "event":
            if not args.value:
                print("Error: event search requires a value")
                return 1
            results = search_by_event_type(args.value, root, limit=args.limit)
        else:
            results = []

        if not results:
            print(f"No sessions found for '{args.field}:{args.value}'.")
            return 0

        print(f"Found {len(results)} session(s):\n")
        for i, entry in enumerate(results):
            print(format_entry(entry, index=i + 1))
            print()

    elif args.command == "show":
        entries = iter_logs(root)
        if args.index < 1 or args.index > len(entries):
            print(f"Error: index {args.index} out of range (1–{len(entries)})")
            return 1
        entry = entries[args.index - 1]
        print(format_detailed(entry))

    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
