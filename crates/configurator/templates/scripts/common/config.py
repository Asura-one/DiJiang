#!/usr/bin/env python3
"""
DiJiang TOML configuration reader.

Reads ``.dijiang/config.toml`` with sensible defaults.
Uses stdlib ``tomllib`` (Python 3.11+).
"""

from __future__ import annotations

import sys
from pathlib import Path

from .paths import get_config_path, get_repo_root


# =============================================================================
# Default values
# =============================================================================

DEFAULT_PLATFORMS = ["pi"]
DEFAULT_TASKS_DIR = ".dijiang/tasks"
DEFAULT_SPEC_DIR = ".dijiang/spec"
DEFAULT_WORKSPACE_DIR = ".dijiang/workspace"


# =============================================================================
# Config model
# =============================================================================

class DiJiangConfig:
    """Typed access to ``.dijiang/config.toml`` with defaults."""

    def __init__(self, raw: dict) -> None:
        self._raw = raw
        self._project = raw.get("project", {})
        self._session = raw.get("session", {})

    # -- Top-level keys --

    @property
    def platforms(self) -> list[str]:
        p = self._raw.get("platforms", DEFAULT_PLATFORMS)
        if isinstance(p, list):
            return [str(x) for x in p]
        return list(DEFAULT_PLATFORMS)

    @property
    def tasks_dir(self) -> str:
        return str(self._raw.get("tasks_dir", DEFAULT_TASKS_DIR))

    @property
    def spec_dir(self) -> str:
        return str(self._raw.get("spec_dir", DEFAULT_SPEC_DIR))

    @property
    def workspace_dir(self) -> str:
        return str(self._raw.get("workspace_dir", DEFAULT_WORKSPACE_DIR))

    @property
    def dijiang_version(self) -> str:
        return str(self._raw.get("dijiang_version", ""))

    # -- [project] --

    @property
    def project_name(self) -> str:
        return str(self._project.get("name", ""))

    @property
    def developer(self) -> str:
        return str(self._project.get("developer", ""))

    @property
    def project_version(self) -> str:
        return str(self._project.get("version", ""))

    # -- [session] --

    @property
    def session_auto_commit(self) -> bool:
        """Whether to auto-stage and auto-commit task/journal changes."""
        raw = self._session.get("auto_commit", True)
        if isinstance(raw, bool):
            return raw
        return str(raw).strip().lower() in ("true", "yes", "1", "on")


# =============================================================================
# Loader
# =============================================================================

def _try_tomllib() -> object:
    """Import ``tomllib`` (stdlib 3.11+) or fail gracefully."""
    try:
        import tomllib
        return tomllib
    except ImportError:
        return None


def load_config(repo_root: Path | None = None) -> DiJiangConfig:
    """Load ``.dijiang/config.toml``.

    Returns a ``DiJiangConfig`` object with defaults when the file
    is missing or unreadable.
    """
    path = get_config_path(repo_root)
    try:
        raw_bytes = path.read_bytes()
    except (OSError, IOError):
        return DiJiangConfig({})

    toml_mod = _try_tomllib()
    if toml_mod is None:
        print("Warning: tomllib not available (Python <3.11). Config may be incomplete.", file=sys.stderr)
        return DiJiangConfig({})

    try:
        parsed = toml_mod.loads(path.read_text(encoding="utf-8"))
        return DiJiangConfig(parsed)
    except Exception as exc:
        print(f"Warning: failed to parse {path}: {exc}", file=sys.stderr)
        return DiJiangConfig({})


def get_developer(repo_root: Path | None = None) -> str:
    """Return developer name from config, or empty string."""
    return load_config(repo_root).developer
