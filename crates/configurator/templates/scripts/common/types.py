#!/usr/bin/env python3
"""
Type definitions for DiJiang task data.

Mirrors the JSON schema used in ``task.json`` files.
"""

from __future__ import annotations

from typing import NotRequired, TypedDict


class TaskData(TypedDict, total=False):
    """Full task data as stored in ``task.json``.

    All fields are optional to support partial reads/writes.
    The TypedDict is structural - extra keys in the JSON are preserved.
    """

    id: str
    name: str
    title: str
    description: str
    status: str  # planning | in_progress | completed | archived | paused

    # Classification
    devType: str | None
    scope: str | None
    package: str | None
    priority: str  # P0 | P1 | P2 | P3

    # People
    creator: str
    assignee: str

    # Timestamps
    createdAt: str
    completedAt: str | None
    startedAt: str | None

    # Git metadata
    branch: str | None
    baseBranch: str | None
    worktreePath: str | None
    commit: str | None
    prUrl: str | None

    # Hierarchy
    subtasks: list[str]
    children: list[str]
    parent: str | None

    # Notes
    relatedFiles: list[str]
    notes: str
    meta: dict | None


VALID_STATUSES = frozenset({
    "planning",
    "in_progress",
    "completed",
    "archived",
    "paused",
})
