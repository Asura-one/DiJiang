#!/usr/bin/env python3
"""
Task data access layer.

Provides helpers to load, list, and iterate tasks from the filesystem.
"""

from __future__ import annotations

import os
from pathlib import Path

from .paths import get_archive_dir, get_task_dir, get_task_json, get_tasks_dir
from .io import read_json, write_json
from .types import TaskData, VALID_STATUSES


def load_task(task_id: str, root: Path | None = None) -> TaskData | None:
    """Load a task's ``task.json``.

    Returns ``None`` if the file doesn't exist or is invalid.
    """
    data = read_json(get_task_json(task_id, root))
    if isinstance(data, dict):
        return TaskData(data)  # type: ignore[valid-type]
    return None


def write_task(task_id: str, data: TaskData, root: Path | None = None) -> bool:
    """Write a TaskData dict to the task's ``task.json``."""
    return write_json(get_task_json(task_id, root), data)


def iter_active_tasks(root: Path | None = None) -> list[tuple[str, TaskData]]:
    """Iterate over all non-archived tasks (excludes ``archive/``).

    Yields ``(task_id, task_data)`` pairs.
    """
    tasks_dir = get_tasks_dir(root)
    archive_dir = get_archive_dir(root)
    results: list[tuple[str, TaskData]] = []

    if not tasks_dir.is_dir():
        return results

    for entry in sorted(tasks_dir.iterdir()):
        if not entry.is_dir():
            continue
        # Skip the archive directory itself
        if entry.resolve() == archive_dir.resolve():
            continue

        task_file = entry / "task.json"
        if not task_file.exists():
            continue

        data = read_json(task_file)
        if isinstance(data, dict):
            results.append((entry.name, TaskData(data)))  # type: ignore[valid-type]

    return results


def iter_all_tasks(root: Path | None = None) -> list[tuple[str, TaskData]]:
    """Iterate over ALL tasks, including those in ``archive/``."""
    results = iter_active_tasks(root)
    archive_dir = get_archive_dir(root)
    if archive_dir.is_dir():
        for entry in sorted(archive_dir.iterdir()):
            if not entry.is_dir():
                continue
            task_file = entry / "task.json"
            if not task_file.exists():
                continue
            data = read_json(task_file)
            if isinstance(data, dict):
                results.append((entry.name, TaskData(data)))  # type: ignore[valid-type]
    return results


def find_task_by_status(
    status: str, root: Path | None = None
) -> list[tuple[str, TaskData]]:
    """Find tasks matching a given status."""
    return [
        (tid, td) for tid, td in iter_all_tasks(root)
        if td.get("status") == status
    ]


def is_valid_status(status: str) -> bool:
    """Check if *status* is a recognised workflow state."""
    return status in VALID_STATUSES


def slugify(name: str) -> str:
    """Convert *name* to a filesystem-safe task ID slug.

    Preserves dashes, lowercases, and collapses whitespace.
    """
    import re
    s = name.strip().lower()
    s = re.sub(r"[^a-z0-9\s-]", "", s)
    s = re.sub(r"\s+", "-", s)
    s = re.sub(r"-+", "-", s)
    return s.strip("-")


def task_exists(task_id: str, root: Path | None = None) -> bool:
    """Check if a task directory with *task_id* exists (active or archived)."""
    return get_task_dir(task_id, root).is_dir()
