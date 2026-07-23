#!/usr/bin/env python3
"""
DiJiang skills — list available `dj-*` skills.

Usage:
    python3 .dijiang/scripts/skills.py                          # List skills
    python3 .dijiang/scripts/skills.py --json                   # JSON output
    python3 .dijiang/scripts/skills.py --sync                   # Sync skills to .pi/skills/

Lists the registered `dj-*` skills from `.pi/skills/` directory or from
the template registry.

With --sync, copies skill templates from `crates/configurator/templates/skills/`
to `.pi/skills/`.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from common.paths import get_repo_root


KNOWN_SKILLS: list[dict[str, str]] = [
    {"name": "dj-output", "summary": "产出或同步 PRD、design、implement 等任务文档"},
    {"name": "dj-health", "summary": "综合代码库健康检查：构建、测试、Git、依赖、lint、agent 配置"},
    {"name": "dj-debt", "summary": "技术债评估与追踪：多源聚合 ponytail/TODO/依赖/死代码债务"},
    {"name": "dj-channel", "summary": "多 agent 协作通道：生成、监控和管理 AI agent 通道"},
    {"name": "dj-reason", "summary": "推理增强、系统透镜和复杂判断校准"},
    {"name": "dj-research", "summary": "技术调研与信息收集"},
    {"name": "dj-spec-bootstrap", "summary": "扫描 crates 目录并为每个 crate 生成初始 spec 文件"},
    {"name": "dj-session-insight", "summary": "跨会话记忆检索：通过 dijiaang mem recall 检索历史对话、findings 和 learnings"},
    {"name": "dj-meta", "summary": "DiJiang 架构自省、技能创建指南和系统理解"},
    {"name": "dj-implement", "summary": "代码实现与变更推进"},
    {"name": "dj-check", "summary": "质量审计与回归审查"},
    {"name": "dj-hunt", "summary": "Bug 排查与根因诊断"},
    {"name": "dj-grill", "summary": "需求对齐、范围澄清、问题收敛"},
    {"name": "dj-audit", "summary": "全仓代码审计：安全、性能、合规"},
    {"name": "dj-script", "summary": "脚本和工具开发"},
    {"name": "dj-tdd", "summary": "测试驱动开发"},
    {"name": "dj-design", "summary": "UI 设计实现"},
    {"name": "dj-codebase-design", "summary": "代码库架构设计"},
    {"name": "dj-domain-modeling", "summary": "领域建模"},
    {"name": "dj-git-guardrails", "summary": "Git 操作安全护栏"},
    {"name": "dj-handoff", "summary": "会话交接与上下文传递"},
    {"name": "dj-karpathy", "summary": "长篇代码讨论"},
    {"name": "dj-pattern", "summary": "模式研究与识别"},
    {"name": "dj-ponytail", "summary": "最小聚焦改动"},
    {"name": "dj-prototype", "summary": "快速原型开发"},
    {"name": "dj-remix", "summary": "网站/App 复刻与站点再造"},
    {"name": "dj-split", "summary": "任务分解与拆分"},
    {"name": "dj-write", "summary": "写作润色与文档撰写"},
    {"name": "dj-dispatch", "summary": "请求分类与 skill 路由"},
    {"name": "dj-absorb", "summary": "吸收和整合外部知识"},
]

# ── Helpers ──────────────────────────────────────────────────────────────


def list_skills_from_disk(root: Path) -> list[dict[str, str]]:
    """Read available skills from `.pi/skills/` directory."""
    pi_skills = root / ".pi" / "skills"
    if not pi_skills.is_dir():
        return KNOWN_SKILLS

    skills: list[dict[str, str]] = []
    for entry in sorted(pi_skills.iterdir()):
        if entry.is_dir() and entry.name.startswith("dj-"):
            skill_md = entry / "SKILL.md"
            summary = ""
            if skill_md.is_file():
                for line in skill_md.read_text().splitlines():
                    if line.startswith("summary:"):
                        summary = line[len("summary:"):].strip()
                        break
            skills.append({"name": entry.name, "summary": summary})

    if not skills:
        return KNOWN_SKILLS
    return skills


def sync_skills(root: Path) -> int:
    """Sync skills to `.pi/skills/` by copying from template source."""
    template_dir = root / "crates" / "configurator" / "templates" / "skills"
    if not template_dir.is_dir():
        print("Error: template source not found (expected at crates/configurator/templates/skills/)", file=sys.stderr)
        return 1

    pi_skills = root / ".pi" / "skills"
    pi_skills.mkdir(parents=True, exist_ok=True)

    count = 0
    for entry in sorted(template_dir.iterdir()):
        if entry.is_dir() and entry.name.startswith("dj-"):
            src_skill = entry / "SKILL.md"
            if src_skill.is_file():
                dest = pi_skills / entry.name / "SKILL.md"
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(src_skill.read_text(encoding="utf-8"), encoding="utf-8")
                count += 1
                print(f"  Synced {entry.name}")

    print(f"  Synced {count} dj-* skills to .pi/skills/")
    return 0


# ── Main ─────────────────────────────────────────────────────────────────


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description="DiJiang skills — list available dj-* skills")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    parser.add_argument("--sync", action="store_true", help="Sync skills to .pi/skills/")
    args = parser.parse_args()

    if args.sync:
        root = get_repo_root()
        return sync_skills(root)

    root = get_repo_root()
    skills = list_skills_from_disk(root)

    if args.json:
        print(json.dumps(skills, ensure_ascii=False, indent=2))
    else:
        print(f"  {len(skills)} dj-* skills available:")
        for s in skills:
            summary = s.get("summary", "")
            if summary:
                print(f"    {s['name']:<30} {summary}")
            else:
                print(f"    {s['name']}")
        print()
        print("  Use `python3 .dijiang/scripts/skills.py --sync` to write skills to current project.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
