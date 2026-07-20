#!/usr/bin/env python3
"""
Task JSONL context management for DiJiang.

Provides helpers to add, validate, and list context entries stored as
JSONL files inside task directories.

Inspired by Trellis's ``task_context.py``.
"""

from __future__ import annotations

import json
from pathlib import Path

from .paths import get_repo_root


# ── Seed / example row ────────────────────────────────────────────────

SEED_ROW = {
    "_example": {
        "file": "relative/path/to/file.md",
        "reason": "Why this file is relevant to this task",
        "type": "file",
    },
    "_note": "Replace this seed with real entries using `task.py add-context`",
}


# ── Context entry types ────────────────────────────────────────────────

CONTEXT_FILE_NAMES = ["implement.jsonl", "check.jsonl", "context.jsonl"]


# ── Helpers ─────────────────────────────────────────────────────────────

def get_context_dir(task_id: str, root: Path | None = None) -> Path:
    """Return path to the task's directory (where context JSONL lives)."""
    from .paths import get_task_dir
    return get_task_dir(task_id, root)


def seed_context(task_id: str, root: Path | None = None) -> int:
    """Seed a task directory with an empty ``context.jsonl``.

    Creates the file with a self-describing example row if it doesn't exist.
    Returns 0 on success, 1 on failure.
    """
    task_dir = get_context_dir(task_id, root)
    if not task_dir.is_dir():
        print(f"Error: task directory not found: {task_dir}")
        return 1

    context_file = task_dir / "context.jsonl"
    if context_file.exists():
        return 0  # already exists

    try:
        context_file.parent.mkdir(parents=True, exist_ok=True)
        context_file.write_text(
            json.dumps(SEED_ROW, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        return 0
    except OSError as e:
        print(f"Error: failed to seed context: {e}")
        return 1


def add_context_entry(
    task_id: str,
    jsonl_name: str,
    path: str,
    reason: str = "",
    root: Path | None = None,
) -> int:
    """Append an entry to a JSONL context file for a task.

    Args:
        task_id: The task directory name under ``.dijiang/tasks/``.
        jsonl_name: JSONL filename (e.g. ``implement.jsonl``). Shorthand
                    (without ``.jsonl``) is expanded.
        path: Repo-relative file/directory path being referenced.
        reason: Why this context is relevant.
        root: Optional repo root override.

    Returns:
        0 on success, 1 on error.
    """
    repo_root = get_repo_root(root)
    task_dir = get_context_dir(task_id, root)

    if not task_dir.is_dir():
        print(f"Error: task directory not found: {task_dir}")
        return 1

    if not jsonl_name.endswith(".jsonl"):
        jsonl_name += ".jsonl"

    jsonl_file = task_dir / jsonl_name
    full_path = repo_root / path

    entry_type = "file"
    if full_path.is_dir():
        entry_type = "directory"
        if not path.endswith("/"):
            path += "/"
    elif not full_path.exists():
        print(f"Error: path not found (relative to repo root): {path}")
        return 1

    # Dedup: skip if exact path already recorded
    if jsonl_file.is_file():
        content = jsonl_file.read_text(encoding="utf-8")
        if f'"{path}"' in content:
            print(f"Entry already exists for {path} (skipped)")
            return 0

    entry = {"file": path, "reason": reason}
    if entry_type == "directory":
        entry["type"] = "directory"

    try:
        with jsonl_file.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
        print(f"Added {entry_type}: {path}")
        return 0
    except OSError as e:
        print(f"Error: failed to write context entry: {e}")
        return 1


def validate_context_task(
    task_id: str,
    root: Path | None = None,
) -> int:
    """Validate all JSONL context files in a task directory.

    Returns:
        0 if all valid, 1 if any error found.
    """
    repo_root = get_repo_root(root)
    task_dir = get_context_dir(task_id, root)

    if not task_dir.is_dir():
        print(f"Error: task directory not found: {task_dir}")
        return 1

    total_errors = 0
    for jsonl_name in CONTEXT_FILE_NAMES:
        jsonl_file = task_dir / jsonl_name
        errors, entry_count = _validate_jsonl(jsonl_file, repo_root)
        total_errors += errors
        if errors == 0:
            if jsonl_file.exists():
                print(f"  {jsonl_name}: ok ({entry_count} entries)")
        else:
            print(f"  {jsonl_name}: {errors} error(s)")

    if total_errors == 0:
        return 0
    return 1


def list_context_task(task_id: str, root: Path | None = None) -> int:
    """List all JSONL context entries in a task directory."""
    task_dir = get_context_dir(task_id, root)
    if not task_dir.is_dir():
        print(f"Error: task directory not found: {task_dir}")
        return 1

    print(f"Context files for task '{task_dir.name}':")
    print()

    found_any = False
    for jsonl_name in CONTEXT_FILE_NAMES:
        jsonl_file = task_dir / jsonl_name
        if not jsonl_file.exists():
            continue
        found_any = True

        print(f"  [{jsonl_name}]")
        count = 0
        seed_only = True
        for line in jsonl_file.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                continue
            file_path = data.get("file")
            if not file_path:
                continue
            seed_only = False
            count += 1
            entry_type = data.get("type", "file")
            reason = data.get("reason", "-")
            type_tag = f"[{entry_type.upper()}] " if entry_type != "file" else ""
            print(f"    {count}. {type_tag}{file_path}")
            print(f"       → {reason}")

        if seed_only and jsonl_file.exists():
            print("    (no curated entries yet — seed row only)")
        print()

    if not found_any:
        print("  (no JSONL context files)")
    return 0


def _validate_jsonl(jsonl_file: Path, repo_root: Path) -> tuple[int, int]:
    """Validate a single JSONL file. Returns ``(error_count, entry_count)``."""
    errors = 0
    real_entries = 0

    if not jsonl_file.is_file():
        return 0, 0

    for line_num, line in enumerate(jsonl_file.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            print(f"    {jsonl_file.name}:{line_num}: Invalid JSON")
            errors += 1
            continue

        file_path = data.get("file")
        entry_type = data.get("type", "file")

        if not file_path:
            continue  # seed/comment row

        real_entries += 1
        full_path = repo_root / file_path
        if entry_type == "directory":
            if not full_path.is_dir():
                print(f"    {jsonl_file.name}:{line_num}: Directory not found: {file_path}")
                errors += 1
        elif not full_path.is_file():
            print(f"    {jsonl_file.name}:{line_num}: File not found: {file_path}")
            errors += 1

    return errors, real_entries
