#!/usr/bin/env python3
"""
Session identity detection for DiJiang.

Detects the current AI platform and session ID from environment variables,
supporting multi-platform identity resolution.

Inspired by Trellis's ``active_task.py`` session parsing.
"""

from __future__ import annotations

import os
import re
from pathlib import Path


# ── Platform detection ──────────────────────────────────────────────────

# Ordered by specificity — first match wins
PLATFORM_DETECTORS: list[tuple[str, str]] = [
    # Explicit DiJiang context ID (canonical)
    ("dijiang", "DIJIANG_PLATFORM"),
    # Pi agent harness
    ("pi", "PI_PLATFORM"),
    # Claude Code
    ("claude", "CLAUDE_CODE"),
    # Cursor (probe via shell ticket)
    ("cursor", "CURSOR_TICKET"),
    # Codex CLI
    ("codex", "CODEX_CLI"),
    # Anthropic API / Vertex AI
    ("anthropic", "ANTHROPIC_API_KEY"),
    # OpenAI / Codex
    ("openai", "OPENAI_API_KEY"),
    # Generic Terminal (no known platform marker)
    ("terminal", "TERM"),
]


# ── Session identity data ───────────────────────────────────────────────

PLATFORM_SESSION_VARS = {
    "dijiang": "DIJIANG_CONTEXT_ID",
    "pi": "PI_SESSION_ID",
    "claude": "CLAUDE_CODE_SESSION",
    "cursor": "CURSOR_TICKET",
    "codex": "CODEX_CLI_SESSION",
}

# Patterns for extracting session IDs from known formats
SESSION_EXTRACTORS: dict[str, list[str]] = {
    "cursor": [
        r"cursor-ticket-([a-f0-9-]+)",
    ],
    "pi": [
        r"pi-([a-f0-9-]+)",
    ],
}


class SessionIdentity:
    """Detected session identity with platform and session ID."""

    def __init__(
        self,
        platform: str,
        session_id: str = "",
        context_id: str = "",
        developer: str = "",
    ) -> None:
        self.platform = platform
        self.session_id = session_id
        self.context_id = context_id
        self.developer = developer

    @property
    def display_name(self) -> str:
        if self.context_id:
            return f"{self.platform}:{self.context_id[:12]}"
        if self.session_id:
            return f"{self.platform}:{self.session_id[:12]}"
        return self.platform

    @classmethod
    def detect(cls) -> SessionIdentity:
        """Detect the current session identity from the environment.

        Returns:
            A ``SessionIdentity`` with best-guess platform and session ID.
            Falls back to ``"terminal"`` platform if no AI platform is detected.
        """
        # 1. Check explicit DIJIANG_CONTEXT_ID first
        context_id = os.environ.get("DIJIANG_CONTEXT_ID", "").strip()
        if context_id:
            # Try to parse platform from prefix
            platform = _detect_platform_from_context_id(context_id)
            return cls(
                platform=platform,
                context_id=context_id,
                session_id=context_id,
            )

        # 2. Try platform-specific env vars
        for platform, env_var in PLATFORM_DETECTORS:
            value = os.environ.get(env_var, "").strip()
            if value:
                session_id = _extract_session_id(platform, value)
                return cls(
                    platform=platform,
                    session_id=session_id,
                )

        # 3. Fallback: no known platform
        hostname = os.uname().nodename if hasattr(os, "uname") else "unknown"
        return cls(platform="terminal", session_id=hostname)

    def to_dict(self) -> dict:
        return {
            "platform": self.platform,
            "session_id": self.session_id,
            "context_id": self.context_id,
            "developer": self.developer,
            "display_name": self.display_name,
        }


# ── Helpers ─────────────────────────────────────────────────────────────

def _detect_platform_from_context_id(context_id: str) -> str:
    """Guess the platform from a ``DIJIANG_CONTEXT_ID`` value.

    Supports both bare platform names (``pi``, ``claude``) and
    prefixed IDs (``pi-abc123``, ``claude-xyz``).
    """
    known_platforms = {"pi", "claude", "cursor", "codex", "dijiang"}
    if context_id in known_platforms:
        return context_id
    prefixes = {
        "pi-": "pi",
        "claude-": "claude",
        "cursor-": "cursor",
        "codex-": "codex",
    }
    for prefix, platform in prefixes.items():
        if context_id.startswith(prefix):
            return platform
    return "unknown"


def _extract_session_id(platform: str, raw_value: str) -> str:
    """Extract a human-readable session ID from a raw environment value."""
    extractors = SESSION_EXTRACTORS.get(platform, [])
    for pattern in extractors:
        m = re.search(pattern, raw_value)
        if m:
            return m.group(1)
    # Fallback: first 16 chars as session token
    return raw_value[:24]


# ── Convenience ─────────────────────────────────────────────────────────

def get_platform() -> str:
    """Return the detected platform name (short string)."""
    return SessionIdentity.detect().platform


def get_context_id() -> str:
    """Return the ``DIJIANG_CONTEXT_ID`` if set, else empty string."""
    return os.environ.get("DIJIANG_CONTEXT_ID", "").strip()


def is_ai_platform() -> bool:
    """Check if an AI platform is detected (vs bare terminal)."""
    return SessionIdentity.detect().platform not in ("terminal", "unknown")
