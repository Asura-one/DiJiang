#!/usr/bin/env python3
"""
Developer journal entry — record a session note into the workspace journal.

Usage:
    python3 .dijiang/scripts/add_session.py "Title here" --summary "What happened"
    python3 .dijiang/scripts/add_session.py "Refactor auth" --summary "Moved JWT to middleware"
        --commit abc1234

Piped input is appended as detail body:
    echo "line1\nline2" | python3 .dijiang/scripts/add_session.py "Title" --summary "..."

Writes to: ``.dijiang/workspace/<developer>/journal-<date>.md``

Detects platform from environment variables:
    DIJIANG_CONTEXT_ID, PI_SESSION_ID, CLAUDE_CODE, CURSOR_TICKET, etc.
"""

from __future__ import annotations

import argparse
import sys
from datetime import date, datetime
from pathlib import Path

from common.paths import get_workspace_dir, get_repo_root
from common.config import get_developer, load_config
from common.io import ensure_dir
from common.session import SessionIdentity, get_platform, get_context_id


def get_journal_file(root: Path, developer: str = "") -> Path:
    """Return path to today's journal file."""
    if not developer:
        developer = get_developer(root) or "anonymous"
    today = date.today().isoformat()
    return get_workspace_dir(root) / developer / f"journal-{today}.md"


def append_journal(
    root: Path,
    title: str,
    summary: str = "",
    commit: str = "",
    body: str = "",
    platform: str = "",
) -> bool:
    """Append an entry to today's journal file."""
    dev = get_developer(root) or "anonymous"
    journal_path = get_journal_file(root, dev)
    ensure_dir(journal_path.parent)

    timestamp = datetime.now().strftime("%H:%M:%S")
    today = date.today().isoformat()

    # Detect session identity
    session = SessionIdentity.detect()
    platform_tag = platform or session.platform
    session_tag = session.context_id or session.session_id
    session_info = f"[{platform_tag}]" if session_tag else f"[{platform_tag}]"

    entry_parts = [
        f"\n## {timestamp} {session_info} — {title}",
    ]
    if commit:
        entry_parts.append(f"  commit: {commit[:12]}")
    if summary:
        entry_parts.append(f"\n{summary}")
    if body:
        entry_parts.append(f"\n{body.strip()}")

    entry = "\n".join(entry_parts) + "\n"

    try:
        with open(journal_path, "a", encoding="utf-8") as f:
            f.write(entry)
        return True
    except OSError as e:
        print(f"Error writing journal: {e}", file=sys.stderr)
        return False


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Record a session journal entry",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Piped input is appended as detail body.\n"
            "Examples:\n"
            '  %(prog)s "Refactor auth" --summary "Moved JWT middleware"\n'
            '  echo "see details.md" | %(prog)s "Title" --summary "Work"'
        ),
    )
    parser.add_argument("title", help="Entry title")
    parser.add_argument("--summary", "-s", default="", help="Brief summary")
    parser.add_argument("--commit", "-c", default="", help="Commit hash")
    parser.add_argument("--platform", "-p", default="",
                        help="Override platform tag")
    args = parser.parse_args()

    # Read piped input if available
    body = ""
    if not sys.stdin.isatty():
        body = sys.stdin.read()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    ok = append_journal(
        root,
        title=args.title,
        summary=args.summary,
        commit=args.commit,
        body=body,
        platform=args.platform,
    )

    if ok:
        journal_file = get_journal_file(root)
        session = SessionIdentity.detect()
        print(f"Journal entry written to {journal_file}")
        print(f"  Platform: {session.display_name}")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
