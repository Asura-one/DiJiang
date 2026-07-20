#!/usr/bin/env python3
"""
Path utilities for DiJiang Python scripts.

Provides repository root discovery from `.dijiang/` marker directory
and standard path constructors for tasks, specs, workspace, and scripts.
All paths are relative to the repo root.
"""

from __future__ import annotations

from pathlib import Path


# =============================================================================
# Directory/File name constants
# =============================================================================

DIR_DIJIANG = ".dijiang"
DIR_WORKSPACE = "workspace"
DIR_TASKS = "tasks"
DIR_SPEC = "spec"
DIR_SCRIPTS = "scripts"
DIR_REFERENCES = "references"

FILE_TASK_JSON = "task.json"
FILE_CONFIG = "config.toml"


# =============================================================================
# Repository root discovery
# =============================================================================

def get_repo_root(start: Path | None = None) -> Path:
    """Walk up from *start* (default: cwd) until ``.dijiang/`` is found.

    Returns:
        The first ancestor (or *start* itself) that contains ``.dijiang/``.

    Raises:
        SystemExit(1): if no ``.dijiang/`` directory is found all the way up
                       to the filesystem root.
    """
    current = (start or Path.cwd()).resolve()
    while current != current.parent:
        if (current / DIR_DIJIANG).is_dir():
            return current
        current = current.parent
    print("Error: not inside a DiJiang project (no .dijiang/ found)", file=__import__("sys").stderr)
    __import__("sys").exit(1)


def require_repo_root(start: Path | None = None) -> Path:
    """Like ``get_repo_root`` but guaranteed non-None by exiting on failure."""
    return get_repo_root(start)


# =============================================================================
# Shortcuts (all relative to repo root)
# =============================================================================

def get_tasks_dir(root: Path | None = None) -> Path:
    """Return path to the tasks directory."""
    return get_repo_root(root) / DIR_DIJIANG / DIR_TASKS


def get_workspace_dir(root: Path | None = None) -> Path:
    """Return path to the workspace directory."""
    return get_repo_root(root) / DIR_DIJIANG / DIR_WORKSPACE


def get_spec_dir(root: Path | None = None) -> Path:
    """Return path to the spec directory."""
    return get_repo_root(root) / DIR_DIJIANG / DIR_SPEC


def get_scripts_dir(root: Path | None = None) -> Path:
    """Return path to the scripts directory."""
    return get_repo_root(root) / DIR_DIJIANG / DIR_SCRIPTS


def get_config_path(root: Path | None = None) -> Path:
    """Return path to ``.dijiang/config.toml``."""
    return get_repo_root(root) / DIR_DIJIANG / FILE_CONFIG


def get_task_dir(task_id: str, root: Path | None = None) -> Path:
    """Return path to a specific task directory by its id/slug."""
    return get_tasks_dir(root) / task_id


def get_task_json(task_id: str, root: Path | None = None) -> Path:
    """Return path to a task's ``task.json``."""
    return get_task_dir(task_id, root) / FILE_TASK_JSON


def get_archive_dir(root: Path | None = None) -> Path:
    """Return path to the task archive directory inside .dijiang/tasks/."""
    return get_tasks_dir(root) / "archive"
