#!/usr/bin/env python3
"""
Standalone workflow-state breadcrumb for DiJiang hooks.

Replaces ``dijiang workflow-state --json`` by reading all state from
disk and git directly. Does not require the ``dijiang`` binary.

Outputs a JSON envelope with hookEventName + additionalContext
(<dijiang-workflow-state> block) — the same format the platform
hooks expect.

Usage:
    python3 .dijiang/scripts/workflow_state.py
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path


def find_root(start: Path) -> Path | None:
    """Walk up from ``start`` to find the repository root (containing .dijiang/)."""
    cur = start.resolve()
    while cur != cur.parent:
        if (cur / ".dijiang").is_dir():
            return cur
        cur = cur.parent
    return None


def detect_platform() -> str:
    """Detect the AI coding platform from environment variables."""
    env_map: list[tuple[str, str]] = [
        ("CODEIX_VS_CODE_VERSION", "codex"),
        ("CLAUDE_PROJECT_DIR", "claude"),
        ("CURSOR_PROJECT_DIR", "cursor"),
        ("GEMINI_PROJECT_DIR", "gemini"),
        ("COPILOT_PROJECT_DIR", "copilot"),
        ("KIRO_PROJECT_DIR", "kiro"),
        ("QODER_PROJECT_DIR", "qoder"),
        ("CODEBUDDY_PROJECT_DIR", "codebuddy"),
        ("FACTORY_PROJECT_DIR", "droid"),
        ("TRAE_PROJECT_DIR", "trae"),
    ]
    for env_name, platform in env_map:
        if os.environ.get(env_name):
            return platform
    return "unknown"


def detect_session_id() -> str:
    """Detect the AI session ID from environment variables."""
    for var in [
        "DIJIANG_CONTEXT_ID",
        "CODEX_SESSION_ID",
        "CLAUDE_SESSION_ID",
        "CODEX_THREAD_ID",
        "CURSOR_SESSION_ID",
        "CURSOR_CONVERSATION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ]:
        val = os.environ.get(var)
        if val:
            return val
    return "unknown"


def read_active_task(root: Path) -> dict | None:
    """Read the active task from session file + task.json.

    Session identity is driven by ``DIJIANG_CONTEXT_ID`` (e.g. ``pi``).
    Falls back to platform name or "global".
    """
    runtime_dir = root / ".dijiang" / ".runtime" / "sessions"
    if not runtime_dir.is_dir():
        return None

    source = os.environ.get("DIJIANG_CONTEXT_ID", "global")
    session_file = runtime_dir / f"{source}.json"

    if not session_file.is_file():
        platform = detect_platform()
        platform_file = runtime_dir / f"{platform}.json"
        if platform_file.is_file():
            session_file = platform_file
        else:
            return None

    try:
        session_data = json.loads(session_file.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None

    task_name = session_data.get("current_task")
    if not task_name:
        return None

    task_json = root / ".dijiang" / "tasks" / task_name / "task.json"
    if not task_json.is_file():
        return {
            "id": task_name,
            "title": task_name,
            "status": "?",
            "task_path": str(root / ".dijiang" / "tasks" / task_name),
        }

    try:
        task_data = json.loads(task_json.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {
            "id": task_name,
            "title": task_name,
            "status": "?",
            "task_path": str(root / ".dijiang" / "tasks" / task_name),
        }

    return {
        "id": task_data.get("id", task_name),
        "title": task_data.get("title", task_name),
        "status": task_data.get("status", "?"),
        "task_path": str(root / ".dijiang" / "tasks" / task_name),
    }


def get_git_state(root: Path) -> dict:
    """Return short git state (branch + dirty file count)."""
    branch = "unknown"
    try:
        branch = (
            subprocess.check_output(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                cwd=root,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            .strip()
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass

    dirty_count = 0
    try:
        dirty = (
            subprocess.check_output(
                ["git", "status", "--porcelain"],
                cwd=root,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            .strip()
        )
        dirty_count = len([l for l in dirty.split("\n") if l.strip()]) if dirty else 0
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass

    return {"branch": branch, "dirty_count": dirty_count}


# Tag parser adapted from Trellis — matches [workflow-state:STATUS]⋯[/workflow-state:STATUS]
_TAG_RE = re.compile(
    r"\[workflow-state:([A-Za-z0-9_-]+)\]\s*\n(.*?)\n\s*\[/workflow-state:\1\]",
    re.DOTALL,
)


def load_workflow_tags(root: Path) -> dict[str, str]:
    """Parse ``workflow.md`` for ``[workflow-state:STATUS]`` blocks."""
    workflow = root / ".dijiang" / "workflow.md"
    if not workflow.is_file():
        return {}
    try:
        content = workflow.read_text(encoding="utf-8")
    except OSError:
        return {}

    result: dict[str, str] = {}
    for match in _TAG_RE.finditer(content):
        status = match.group(1)
        body = match.group(2).strip()
        if body:
            result[status] = body
    return result


def build_context(
    task: dict | None,
    git: dict,
    tags: dict[str, str],
    platform: str,
    session_id: str,
) -> str:
    """Build the ``<dijiang-workflow-state>⋯</dijiang-workflow-state>`` block."""
    lines: list[str] = ["<dijiang-workflow-state>"]
    lines.append(f"平台: {platform}")
    lines.append(f"会话: {session_id}")

    if task:
        lines.append(f"活跃任务: {task['id']}")
        lines.append(f"标题: {task['title']}")
        lines.append(f"状态: {task['status']}")
        lines.append(f"任务路径: {task['task_path']}")

        tag_body = tags.get(task["status"])
        if tag_body:
            lines.append(f"Workflow 标签 [{task['status']}]:")
            lines.append(tag_body)
        else:
            lines.append(f"Workflow 标签 [{task['status']}]: （无）")
    else:
        lines.append("活跃任务: none")

    lines.append(
        f"Git: branch={git['branch']}, "
        f"dirty={ 'yes (' + str(git['dirty_count']) + ' files)' if git['dirty_count'] > 0 else 'no'}"
    )

    if task:
        lines.append("加载上下文：读取 task.json；如果存在，也读取 prd.md/design.md/implement.md/check 产物。")

    lines.append("</dijiang-workflow-state>")
    return "\n".join(lines)


def main() -> int:
    hook_event = "UserPromptSubmit"

    # Read optional stdin payload for hook event name override
    try:
        stdin_data = sys.stdin.read()
        if stdin_data.strip():
            payload = json.loads(stdin_data)
            if isinstance(payload, dict):
                hook_event = payload.get("hookEventName", hook_event)
    except (OSError, json.JSONDecodeError):
        pass

    # Allow hook platforms to set CWD via env
    cwd_str = os.environ.get("DIJIANG_PROJECT_DIR") or os.getcwd()
    root = find_root(Path(cwd_str))
    if root is None:
        return 0  # Not a DiJiang project — silent no-op

    platform = detect_platform()
    session_id = detect_session_id()
    task = read_active_task(root)
    git = get_git_state(root)
    tags = load_workflow_tags(root)
    context = build_context(task, git, tags, platform, session_id)

    output = {"hookEventName": hook_event, "additionalContext": context}
    print(json.dumps(output))
    return 0


if __name__ == "__main__":
    sys.exit(main())
