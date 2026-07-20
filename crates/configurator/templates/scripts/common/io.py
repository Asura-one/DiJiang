#!/usr/bin/env python3
"""
I/O utilities for reading/writing JSON task files.

All functions operate relative to the repo root found by ``paths.get_repo_root()``.
"""

from __future__ import annotations

import json
import os
from pathlib import Path


def read_json(path: Path) -> dict | list | None:
    """Read a JSON file and return its content.

    Returns ``None`` if the file does not exist or is not valid JSON.
    """
    if not path.exists():
        return None
    try:
        raw = path.read_text(encoding="utf-8")
        return json.loads(raw)
    except (json.JSONDecodeError, OSError):
        return None


def write_json(path: Path, data: object, *, pretty: bool = True) -> bool:
    """Write *data* to *path* as JSON, creating parent directories.

    Returns ``True`` on success.
    """
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        kwargs = {"indent": 2, "ensure_ascii": False} if pretty else {}
        path.write_text(json.dumps(data, **kwargs) + "\n", encoding="utf-8")
        return True
    except OSError:
        return False


def ensure_dir(path: Path) -> Path:
    """Create *path* (and parents) if it doesn't exist, then return it."""
    path.mkdir(parents=True, exist_ok=True)
    return path


def find_marker_in_parents(name: str, start: Path | None = None) -> Path | None:
    """Walk up from *start* (default: cwd) looking for a file or directory *name*.

    Returns the first ancestor containing *name*, or ``None``.
    """
    current = (start or Path.cwd()).resolve()
    while current != current.parent:
        if (current / name).exists():
            return current
        current = current.parent
    return None


def walk_markdown_files(directory: Path) -> list[Path]:
    """Recursively list all ``.md`` files under *directory*, sorted."""
    if not directory.is_dir():
        return []
    return sorted(directory.rglob("*.md"))
