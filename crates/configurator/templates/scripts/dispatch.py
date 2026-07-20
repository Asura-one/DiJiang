#!/usr/bin/env python3
"""
DiJiang dispatch — request classification + skill routing.

Usage:
    python3 .dijiang/scripts/dispatch.py <prompt>             # Text output
    python3 .dijiang/scripts/dispatch.py <prompt> --json     # JSON output

Outputs a dispatch block compatible with <dijiang-dispatch> XML format.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from common.paths import get_repo_root, get_tasks_dir
from common.tasks import load_task
from common.io import read_json


# ── Dispatch routes ──────────────────────────────────────────────────────

DispatchRoute = dict


def strip_embedded_context(prompt: str) -> str:
    """Strip <skill>...</skill> embedded context from prompt."""
    import re
    return re.sub(r'<skill[^>]*>.*?</skill>', '', prompt, flags=re.DOTALL)


def dispatch_route(prompt: str) -> DispatchRoute:
    """Classify a user prompt into a dispatch route using keyword matching."""
    visible = strip_embedded_context(prompt)
    lower = visible.lower()

    def has_any(words: list[str]) -> bool:
        return any(w in lower for w in words)

    has_specific_failure = has_any([
        "排查", "调试", "debug", "crash", "error", "fail", "报错",
        "崩溃", "无法启动", "不能运行", "失败", "复现", "日志", "stack", "trace",
    ])
    has_specific_impl = has_any([
        "字段", "接口", "按钮", "页面", "文件", "函数", "方法", "模块", "配置",
        "校验", "样式", "布局", "api", "cli", "command", "config", "button", "field",
    ])
    has_vague_bug = has_any([
        "修 bug", "修bug", "fix bug", "fix bugs", "修复 bug", "修复bug",
        "有个 bug", "有 bug", "bug 这些", "bug这些",
    ])
    has_vague_feature = has_any([
        "做个", "做一个", "加个", "加一个", "新增个", "新增一个",
        "实现个", "实现一个", "优化", "改进", "提升", "体验",
    ]) and not has_specific_impl
    has_hunt = has_specific_failure or (has_any(["bug"]) and not has_vague_bug)

    if has_hunt:
        return {
            "task_type": "排查调试",
            "primary_intent": "排查根因",
            "skill": "dj-hunt",
            "recommended_path": "dj-hunt → dj-implement → dj-check",
            "status": "in_progress",
            "intent": "debug",
            "complexity": "complex",
        }
    if has_vague_feature:
        return {
            "task_type": "调研对齐",
            "primary_intent": "需求澄清",
            "skill": "dj-grill",
            "recommended_path": "dj-grill → dj-output/dj-implement",
            "status": "planning",
            "intent": "align",
            "complexity": "complex",
        }
    if has_any(["审计", "安全", "扫描", "体检", "audit", "security"]):
        return {
            "task_type": "审计代码",
            "primary_intent": "代码审计",
            "skill": "dj-audit",
            "recommended_path": "dj-audit → dj-implement → dj-check",
            "status": "in_progress",
            "intent": "check",
            "complexity": "complex",
        }
    if has_any(["调研", "research", "资料", "技术方案对比"]):
        return {
            "task_type": "技术调研",
            "primary_intent": "调研收集信息",
            "skill": "dj-research",
            "recommended_path": "dj-research → dj-output/dj-implement",
            "status": "planning",
            "intent": "research",
            "complexity": "complex",
        }
    if has_any(["方案", "对比", "url", "网页", "compare"]):
        return {
            "task_type": "调研对齐",
            "primary_intent": "调研并对齐",
            "skill": "dj-grill",
            "recommended_path": "dj-grill → dj-output/dj-tdd",
            "status": "planning",
            "intent": "align",
            "complexity": "complex",
        }
    if has_any(["文档", "prd", "设计文档", "润色", "document", "write"]):
        return {
            "task_type": "写文档",
            "primary_intent": "文档产出",
            "skill": "dj-output",
            "recommended_path": "dj-output",
            "status": "planning",
            "intent": "document",
            "complexity": "complex",
        }
    if has_any(["脚本", "工具", "自动化", "script", "tool"]):
        return {
            "task_type": "脚本工具",
            "primary_intent": "脚本或工具实现",
            "skill": "dj-script",
            "recommended_path": "dj-script → dj-check",
            "status": "in_progress",
            "intent": "implement",
            "complexity": "complex",
        }
    if has_any(["ui", "页面", "样式", "布局", "组件", "design", "style"]):
        return {
            "task_type": "设计 UI",
            "primary_intent": "UI 设计实现",
            "skill": "dj-design",
            "recommended_path": "dj-design → dj-implement → dj-check",
            "status": "in_progress",
            "intent": "implement",
            "complexity": "complex",
        }
    if has_any(["测试", "tdd", "test"]):
        return {
            "task_type": "测试开发",
            "primary_intent": "测试驱动开发",
            "skill": "dj-tdd",
            "recommended_path": "dj-tdd → dj-check",
            "status": "in_progress",
            "intent": "implement",
            "complexity": "complex",
        }
    if has_any(["实现", "修复", "重构", "新增", "修改", "改",
                "implement", "fix", "refactor", "add"]):
        return {
            "task_type": "代码开发",
            "primary_intent": "实现变更",
            "skill": "dj-implement",
            "recommended_path": "dj-implement → dj-check",
            "status": "in_progress",
            "intent": "implement",
            "complexity": "complex",
        }

    # Default: unclear → align
    return {
        "task_type": "调研对齐",
        "primary_intent": "需求澄清",
        "skill": "dj-grill",
        "recommended_path": "dj-grill → dj-output/dj-implement",
        "status": "planning",
        "intent": "unknown",
        "complexity": "complex",
    }


def dispatch_route_for_active_task(task_data: dict) -> DispatchRoute:
    """Determine the dispatch route based on active task status."""
    status = task_data.get("status", "planning")
    meta = task_data.get("meta", {})

    route_for_status = {
        "planning": {
            "task_type": "调研对齐",
            "primary_intent": "需求澄清",
            "skill": "dj-grill",
            "recommended_path": "dj-grill → dj-output/dj-implement",
            "status": "planning",
            "intent": "align",
            "complexity": "complex",
        },
        "in_progress": _dispatch_from_meta(meta) or {
            "task_type": "代码开发",
            "primary_intent": "继续实现",
            "skill": "dj-implement",
            "recommended_path": "dj-implement → dj-check",
            "status": "in_progress",
            "intent": "implement",
            "complexity": "complex",
        },
        "completed": {
            "task_type": "收尾归档",
            "primary_intent": "完成工作",
            "skill": "dijiang-finish-work",
            "recommended_path": "dijiang-finish-work",
            "status": "completed",
            "intent": "finish",
            "complexity": "complex",
        },
        "paused": {
            "task_type": "恢复上下文",
            "primary_intent": "继续暂停任务",
            "skill": "dijiang-continue",
            "recommended_path": "dijiang-continue",
            "status": "paused",
            "intent": "resume",
            "complexity": "complex",
        },
        "archived": {
            "task_type": "恢复上下文",
            "primary_intent": "重新激活归档任务",
            "skill": "dijiang-start",
            "recommended_path": "dijiang-start",
            "status": "archived",
            "intent": "resume",
            "complexity": "complex",
        },
    }
    return route_for_status.get(status, route_for_status["planning"])


def _dispatch_from_meta(meta: dict | None) -> DispatchRoute | None:
    """If task has a stored dispatch skill, use it."""
    if not meta:
        return None
    dispatch = meta.get("dispatch", {})
    skill = dispatch.get("skill")
    if skill:
        return dispatch_route_from_skill(skill)
    route = meta.get("route", {})
    skill = route.get("skill")
    if skill:
        return dispatch_route_from_skill(skill)
    return None


SKILL_ROUTES: dict[str, DispatchRoute] = {
    "dj-hunt": {
        "task_type": "排查调试", "primary_intent": "继续排查",
        "skill": "dj-hunt", "recommended_path": "dj-hunt → dj-implement → dj-check",
        "status": "in_progress", "intent": "debug", "complexity": "complex",
    },
    "dj-implement": {
        "task_type": "代码开发", "primary_intent": "继续实现",
        "skill": "dj-implement", "recommended_path": "dj-implement → dj-check",
        "status": "in_progress", "intent": "implement", "complexity": "complex",
    },
    "dj-script": {
        "task_type": "脚本工具", "primary_intent": "继续实现脚本或工具",
        "skill": "dj-script", "recommended_path": "dj-script → dj-check",
        "status": "in_progress", "intent": "implement", "complexity": "complex",
    },
    "dj-tdd": {
        "task_type": "测试开发", "primary_intent": "继续 TDD",
        "skill": "dj-tdd", "recommended_path": "dj-tdd → dj-check",
        "status": "in_progress", "intent": "implement", "complexity": "complex",
    },
    "dj-check": {
        "task_type": "代码审查", "primary_intent": "质量检查",
        "skill": "dj-check", "recommended_path": "dj-check",
        "status": "in_progress", "intent": "check", "complexity": "complex",
    },
    "dj-output": {
        "task_type": "写文档", "primary_intent": "文档产出",
        "skill": "dj-output", "recommended_path": "dj-output",
        "status": "planning", "intent": "document", "complexity": "complex",
    },
    "dj-grill": {
        "task_type": "调研对齐", "primary_intent": "需求澄清",
        "skill": "dj-grill", "recommended_path": "dj-grill → dj-output/dj-implement",
        "status": "planning", "intent": "align", "complexity": "complex",
    },
    "dj-research": {
        "task_type": "调研对齐", "primary_intent": "需求澄清",
        "skill": "dj-grill", "recommended_path": "dj-grill → dj-output/dj-implement",
        "status": "planning", "intent": "align", "complexity": "complex",
    },
    "dijiang-finish-work": {
        "task_type": "收尾归档", "primary_intent": "完成工作",
        "skill": "dijiang-finish-work", "recommended_path": "dijiang-finish-work",
        "status": "completed", "intent": "finish", "complexity": "complex",
    },
    "dijiang-continue": {
        "task_type": "恢复上下文", "primary_intent": "继续暂停任务",
        "skill": "dijiang-continue", "recommended_path": "dijiang-continue",
        "status": "paused", "intent": "resume", "complexity": "complex",
    },
    "dijiang-start": {
        "task_type": "恢复上下文", "primary_intent": "重新激活归档任务",
        "skill": "dijiang-start", "recommended_path": "dijiang-start",
        "status": "archived", "intent": "resume", "complexity": "complex",
    },
}


def dispatch_route_from_skill(skill: str) -> DispatchRoute | None:
    """Resolve a skill name to its dispatch route."""
    return SKILL_ROUTES.get(skill)


def slug_from_prompt(prompt: str) -> str:
    """Generate a slug from a prompt string."""
    slug = ""
    last_dash = False
    for ch in prompt.lower():
        if ch.isalnum():
            slug += ch
            last_dash = False
        elif not last_dash and slug:
            slug += "-"
            last_dash = True
        if len(slug) >= 48:
            break
    slug = slug.strip("-")
    if not slug:
        from datetime import datetime, timezone
        slug = f"task-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}"
    return slug


def title_from_prompt(prompt: str) -> str:
    """Extract a title from a prompt string."""
    compact = " ".join(prompt.split())
    title = compact[:80]
    return title if title.strip() else "Untitled DiJiang Task"


def render_dispatch_context(route: DispatchRoute) -> str:
    """Render a dispatch context block (XML format)."""
    return (
        f"<dijiang-dispatch>\n"
        f"任务：{route.get('task_type', '?')}\n"
        f"标题：{route.get('primary_intent', '?')}\n"
        f"任务类型：{route.get('task_type', '?')}\n"
        f"主要意图：{route.get('primary_intent', '?')}\n"
        f"路线：{route.get('skill', '?')}\n"
        f"推荐路径：{route.get('recommended_path', '?')}\n"
        f"</dijiang-dispatch>"
    )


# ── Main ─────────────────────────────────────────────────────────────────

def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description="DiJiang dispatch — route user prompts to skills")
    parser.add_argument("prompt", nargs="?", default="", help="User prompt to classify")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of XML")
    parser.add_argument("--active-task", help="Task name to check (default: read from active session)")
    args = parser.parse_args()

    prompt = args.prompt
    if not prompt and not sys.stdin.isatty():
        prompt = sys.stdin.read().strip()

    if not prompt:
        print("Usage: python3 dispatch.py <prompt> [--json]", file=sys.stderr)
        return 1

    # Try to get active task
    root = get_repo_root()
    tasks_dir = get_tasks_dir(root)

    active_task = args.active_task
    if not active_task:
        session_file = root / ".dijiang" / ".runtime" / "sessions" / "pi.json"
        if session_file.exists():
            try:
                session = json.loads(session_file.read_text())
                active_task = session.get("current_task")
            except (json.JSONDecodeError, OSError):
                pass

    if active_task:
        task_data = load_task(active_task, root)
        if task_data:
            route = dispatch_route_for_active_task(task_data._raw if hasattr(task_data, '_raw') else task_data)
        else:
            route = dispatch_route(prompt)
    else:
        route = dispatch_route(prompt)

    if args.json:
        print(json.dumps(route, ensure_ascii=False, indent=2))
    else:
        print(render_dispatch_context(route))

    return 0


if __name__ == "__main__":
    sys.exit(main())
