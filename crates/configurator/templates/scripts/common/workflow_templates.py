#!/usr/bin/env python3
"""
Workflow template management for DiJiang.

Supports listing, switching, and initializing workflow templates.
Templates are stored in ``.dijiang/workflow-templates/``.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from .paths import get_repo_root, get_tasks_dir


TEMPLATES_DIR = "workflow-templates"
WORKFLOW_FILE = "workflow.md"


def get_templates_dir(root: Path | None = None) -> Path:
    """Return path to the workflow templates directory."""
    return get_repo_root(root) / ".dijiang" / TEMPLATES_DIR


def list_templates(root: Path | None = None) -> list[tuple[str, str]]:
    """List available workflow templates as ``(name, description)`` pairs.

    Scans ``.dijiang/workflow-templates/*.md`` and extracts the first
    heading as description.
    """
    tmpl_dir = get_templates_dir(root)
    if not tmpl_dir.is_dir():
        return []

    results: list[tuple[str, str]] = []
    for f in sorted(tmpl_dir.glob("*.md")):
        name = f.stem
        text = f.read_text(encoding="utf-8")
        # First line after # heading
        desc = ""
        for line in text.splitlines():
            if line.startswith("# "):
                continue
            if line.strip():
                desc = line.strip().strip("#").strip()
                if len(desc) > 120:
                    desc = desc[:117] + "..."
                break
        results.append((name, desc))
    return results


def get_template_path(name: str, root: Path | None = None) -> Path | None:
    """Return the path to a template by name, or None if not found."""
    tmpl_dir = get_templates_dir(root)
    path = tmpl_dir / f"{name}.md"
    return path if path.is_file() else None


def apply_template(
    name: str,
    target: str = "workflow.md",
    root: Path | None = None,
) -> int:
    """Copy a workflow template to ``.dijiang/<target>``.

    Args:
        name: Template name (without ``.md``).
        target: Output filename under ``.dijiang/`` (default: ``workflow.md``).
        root: Optional repo root override.

    Returns:
        0 on success, 1 on error.
    """
    src = get_template_path(name, root)
    if src is None:
        print(f"Error: template '{name}' not found.")
        return 1

    dest = get_repo_root(root) / ".dijiang" / target
    try:
        shutil.copy2(src, dest)
        print(f"Applied template '{name}' → {dest}")
        return 0
    except OSError as e:
        print(f"Error: {e}")
        return 1
