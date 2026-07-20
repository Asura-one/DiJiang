#!/usr/bin/env python3
"""
verify_skills.py — Waza-style 高级技能验证

Checks:
  1. Frontmatter 完整性（required fields, 有效解析）
  2. 引用图谱（SKILL → spec, spec → spec 交叉引用全部存在）
  3. Outcome Contract 结构完整性
  4. Hard Rules 格式一致性
  5. Gotchas 表格结构一致性
  6. 索引完整性（guides/index.md ↔ 实际文件双向一致）
  7. 分组统计报告

Usage:
  python3 .dijiang/spec/scripts/verify_skills.py          # 全量检查
  python3 .dijiang/spec/scripts/verify_skills.py --json   # JSON 输出
  python3 .dijiang/spec/scripts/verify_skills.py --skill dj-hunt  # 单个 skill

Exit code: 0 if all checks pass, 1 if any fail, 2 on internal error.
"""

import os
import re
import sys
import json
import subprocess
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path(
    os.environ.get("DIJIANG_ROOT")
    or subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], stderr=subprocess.DEVNULL
    ).strip().decode()
)

SKILL_DIR = PROJECT_ROOT / ".pi" / "skills"
SPEC_DIR = PROJECT_ROOT / ".dijiang" / "spec"
GUIDES_DIR = SPEC_DIR / "guides"
INDEX_FILE = GUIDES_DIR / "index.md"

REQUIRED_FRONTMATTER = {"name", "description"}
REQUIRED_SECTIONS = {"Outcome Contract", "Hard Rules", "Gotchas"}
OUTCOME_FIELDS = {"Outcome", "Done when", "Evidence", "Output"}

# ── Utilities ──────────────────────────────────────────────────────────────

def parse_frontmatter(text):
    """Parse YAML-like frontmatter from markdown text without PyYAML.
    
    Supports:
      - key: value
      - key: >
          multi-line description
      - key: >
        description with continuation
      - key: "quoted value"
    """
    fm = {}
    m = re.match(r'^---\s*\n(.*?)\n---', text, re.DOTALL)
    if not m:
        return None
    raw = m.group(1)
    lines = raw.split("\n")
    key = None
    buf = []
    in_block = False
    
    for li, line in enumerate(lines):
        # Multi-line continuation (indented)
        if in_block:
            if line.startswith("  ") or line.startswith("\t") or line.strip() == "":
                if line.strip():
                    buf.append(line.strip())
                continue
            else:
                fm[key] = " ".join(buf) if buf else ""
                buf = []
                in_block = False
        
        m_line = re.match(r'^(\w[\w-]*)\s*:\s*(.*)', line)
        if m_line:
            key = m_line.group(1)
            val = m_line.group(2).strip()
            if val == ">" or val == "|-":
                in_block = True
                buf = []
            elif val == "":
                in_block = True
                buf = []
            else:
                # Strip quotes if present
                val = re.sub(r'^"(.*)"$', r'\1', val)
                fm[key] = val
        elif key and not in_block:
            # Continuation (no colon on this line)
            stripped = line.strip()
            if stripped and not line.startswith(" "):
                # New section start without colon? skip
                pass
    
    # Flush any remaining multi-line buffer
    if in_block and buf:
        fm[key] = " ".join(buf)
    
    return fm


def extract_sections(text):
    """Extract named sections from markdown text by heading."""
    sections = {}
    current = None
    current_lines = []
    
    for line in text.split("\n"):
        h = re.match(r'^(#{2,4})\s+(.+)$', line)
        if h:
            if current:
                sections[current] = "\n".join(current_lines).strip()
            current = h.group(2).strip()
            current_lines = []
        elif current:
            current_lines.append(line)
    
    if current:
        sections[current] = "\n".join(current_lines).strip()
    
    return sections


def extract_markdown_links(text):
    """Extract all [text](path) links from markdown text."""
    return re.findall(r'\[([^\]]+)\]\(([^)]+)\)', text)


def extract_spec_refs(text):
    """Extract all `.dijiang/spec/...` backtick references."""
    return re.findall(r'`(\.dijiang/spec/[^`]+)`', text)


def rpad(s, width):
    """Right-pad string to width."""
    return str(s).ljust(width)


# ── Checkers ───────────────────────────────────────────────────────────────

class Checker:
    def __init__(self, name):
        self.name = name
        self.passed = []
        self.failed = []
        self.warnings = []
    
    def ok(self, msg):
        self.passed.append(msg)
    
    def fail(self, msg):
        self.failed.append(msg)
    
    def warn(self, msg):
        self.warnings.append(msg)
    
    def status(self):
        if not self.failed and not self.warnings:
            return "PASS"
        if self.failed:
            return "FAIL"
        return "WARN"


def check_frontmatter(skill_name, text):
    """Verification 1: Frontmatter completeness."""
    c = Checker(f"{skill_name}/frontmatter")
    fm = parse_frontmatter(text)
    
    if fm is None:
        c.fail("No frontmatter found (missing --- delimiters)")
        return c
    
    for field in REQUIRED_FRONTMATTER:
        if field in fm:
            val = fm[field].strip()
            if len(val) < 3:
                c.warn(f"Field '{field}' is too short: '{val}'")
        else:
            c.fail(f"Missing required field: '{field}'")
    
    if "name" in fm:
        expected = f"dj-{skill_name.split('/')[0].replace('dj-', '')}"
        # Just check it starts with dj-
        if not fm["name"].startswith("dj-"):
            c.warn(f"name should start with 'dj-', got '{fm['name']}'")
    
    if not c.failed:
        c.ok(f"Frontmatter valid ({len(fm)} fields)")
    
    return c


def check_sections(skill_name, text):
    """Verification 2: Required sections exist."""
    c = Checker(f"{skill_name}/sections")
    sections = extract_sections(text)
    
    for section in REQUIRED_SECTIONS:
        if section in sections:
            content = sections[section].strip()
            if len(content) < 10:
                c.warn(f"Section '{section}' is nearly empty")
        else:
            c.fail(f"Missing required section: '{section}'")
    
    if not c.failed:
        c.ok(f"All {len(REQUIRED_SECTIONS)} required sections present")
    
    return c


def check_outcome_contract(skill_name, text):
    """Verification 3: Outcome Contract table has all 4 fields."""
    c = Checker(f"{skill_name}/outcome-contract")
    sections = extract_sections(text)
    
    contract = sections.get("Outcome Contract", "")
    if not contract:
        c.fail("Outcome Contract section is empty or missing")
        return c
    
    # Find table rows
    table_rows = re.findall(r'\|\s*\*?\*?([^*|]+)\*?\*?\s*\|\s*(.+?)\s*\|', contract)
    found_fields = {row[0].strip() for row in table_rows if row[0].strip() in OUTCOME_FIELDS}
    
    for field in OUTCOME_FIELDS:
        if field not in found_fields:
            c.fail(f"Outcome Contract missing field: '{field}'")
    
    if found_fields:
        c.ok(f"Outcome Contract has {len(found_fields)}/4 fields: {', '.join(sorted(found_fields))}")
    
    return c


def check_hard_rules(skill_name, text):
    """Verification 4: Hard Rules section structure."""
    c = Checker(f"{skill_name}/hard-rules")
    sections = extract_sections(text)
    
    hr = sections.get("Hard Rules", "")
    if not hr:
        c.fail("Hard Rules section is empty")
        return c
    
    lines = [l.strip() for l in hr.split("\n") if l.strip()]
    
    # Should have numbered or bulleted items
    rule_count = 0
    for line in lines:
        if re.match(r'^(\d+[\.\)]|[-*])\s', line):
            rule_count += 1
    
    if rule_count == 0:
        c.warn("No numbered/bulleted rules found in Hard Rules")
    elif rule_count < 3:
        c.warn(f"Only {rule_count} rules — consider adding more")
    else:
        c.ok(f"{rule_count} rules defined")
    
    return c


def check_gotchas(skill_name, text):
    """Verification 5: Gotchas table structure."""
    c = Checker(f"{skill_name}/gotchas")
    sections = extract_sections(text)
    
    gotchas = sections.get("Gotchas", "")
    if not gotchas:
        c.fail("Gotchas section is empty")
        return c
    
    table_rows = re.findall(r'\|(.+?)\|(.+?)\|(.+?)\|', gotchas)
    
    if len(table_rows) < 2:
        c.warn("Gotchas table has no data rows")
        return c
    
    # Header should include Gotcha / Consequence / Prevention (or Chinese equivalents)
    headers = [h.strip().lower() for h in table_rows[0]]
    expected_headers = [{"gotcha", "后果", "what happened"}, {"consequence", "后果", "rule"}, {"prevention", "预防", "prevent"}]
    
    data_rows = table_rows[2:]  # Skip header + separator
    if data_rows:
        c.ok(f"{len(data_rows)} gotcha entries")
    else:
        c.warn("Gotchas table has header but no data")
    
    return c


def check_references(skill_name, text):
    """Verification 6: All `.dijiang/spec/` references point to real files."""
    c = Checker(f"{skill_name}/references")
    refs = extract_spec_refs(text)
    
    if not refs:
        c.ok("No spec references to check")
        return c
    
    missing = []
    for ref in refs:
        resolved = PROJECT_ROOT / ref
        if not resolved.exists():
            missing.append(ref)
    
    if missing:
        for ref in missing:
            c.fail(f"Reference not found: {ref}")
    else:
        c.ok(f"All {len(refs)} references valid")
    
    return c


def check_index_consistency():
    """Verification 7: Index ↔ filesystem bidirectional check."""
    c = Checker("index")
    
    if not INDEX_FILE.exists():
        c.fail(f"Index file not found: {INDEX_FILE}")
        return c
    
    index_text = INDEX_FILE.read_text()
    
    # Extract all links from index
    index_links = extract_markdown_links(index_text)
    
    # Map of relative paths referenced in index
    index_refs = {}
    for link_text, link_path in index_links:
        if link_path.startswith("./"):
            rel_path = link_path[2:]
            index_refs[rel_path] = link_text
    
    # Check each index ref exists
    for rel_path, link_text in index_refs.items():
        full = GUIDES_DIR / rel_path
        if not full.exists():
            c.fail(f"Index links to '{rel_path}' but file not found")
    
    # Check each guide file has index entry
    for f in sorted(GUIDES_DIR.glob("*.md")):
        if f.name == "index.md":
            continue
        name_no_ext = f.stem
        # Check if this file's name appears in any index link
        found = False
        for link_text, link_path in index_links:
            if name_no_ext in link_path or name_no_ext in link_text:
                found = True
                break
        if not found:
            c.fail(f"'{f.name}' has no entry in index.md")
    
    # Count valid linked entries
    valid = sum(1 for r in index_refs if (GUIDES_DIR / r).exists())
    if valid:
        c.ok(f"{valid} index entries verified")
    
    return c


def check_reference_graph():
    """Verification 8: Build and validate cross-reference dependency graph."""
    c = Checker("reference-graph")
    
    # Collect all .md files in spec tree
    spec_files = {}
    for f in sorted(SPEC_DIR.rglob("*.md")):
        rel = f.relative_to(PROJECT_ROOT)
        spec_files[str(rel)] = f
    
    # For each spec file, find references to other spec files
    graph = defaultdict(list)  # source → [targets]
    broken = []
    for rel_path, abs_path in spec_files.items():
        text = abs_path.read_text()
        refs = extract_spec_refs(text)
        for ref in refs:
            graph[rel_path].append(ref)
            full = PROJECT_ROOT / ref
            if not full.exists():
                broken.append((rel_path, ref))
    
    if broken:
        for src, tgt in broken:
            c.fail(f"Broken ref: {src} → {tgt}")
    else:
        total_refs = sum(len(v) for v in graph.values())
        total_files = len(spec_files)
        c.ok(f"Reference graph: {total_files} files, {total_refs} edges, 0 broken")
    
    return c


# ── Report ─────────────────────────────────────────────────────────────────

class Report:
    def __init__(self):
        self.checkers = []
    
    def add(self, checker):
        self.checkers.append(checker)
    
    def print_summary(self, format="text"):
        results = defaultdict(lambda: {"pass": 0, "fail": 0, "warn": 0})
        details = []
        
        for c in self.checkers:
            results[c.status()]["pass" if c.status() == "PASS" else "fail" if c.status() == "FAIL" else "warn"] += 1
            for msg in c.failed:
                details.append({"checker": c.name, "status": "FAIL", "message": msg})
            for msg in c.warnings:
                details.append({"checker": c.name, "status": "WARN", "message": msg})
        
        if format == "json":
            output = {
                "summary": {
                    "pass": results.get("PASS", {}).get("pass", 0) + results.get("PASS", {}).get("pass", 0),
                    "fail": sum(1 for c in self.checkers if c.status() == "FAIL"),
                    "warn": sum(1 for c in self.checkers if c.status() == "WARN"),
                },
                "details": details
            }
            print(json.dumps(output, indent=2, ensure_ascii=False))
            return
        
        # Table output
        print(f"\n{'Checker':<50} {'Status':<8} {'Pass':<5} {'Fail':<5} {'Warn':<5}")
        print("-" * 75)
        
        total_pass = total_fail = total_warn = 0
        for c in self.checkers:
            p = len(c.passed)
            f = len(c.failed)
            w = len(c.warnings)
            total_pass += p
            total_fail += f
            total_warn += w
            
            status = c.status()
            print(f"{c.name:<50} {status:<8} {p:<5} {f:<5} {w:<5}")
        
        print("-" * 75)
        print(f"{'TOTAL':<50} {'':<8} {total_pass:<5} {total_fail:<5} {total_warn:<5}")
        print()
        
        # Details for failures
        if any(c.failed for c in self.checkers):
            print("--- Failures ---")
            for c in self.checkers:
                for msg in c.failed:
                    print(f"  FAIL  {c.name:<45} {msg}")
        
        if any(c.warnings for c in self.checkers):
            print("--- Warnings ---")
            for c in self.checkers:
                for msg in c.warnings:
                    print(f"  WARN  {c.name:<45} {msg}")
    
    def success(self):
        return all(c.status() != "FAIL" for c in self.checkers)


# ── Main ───────────────────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Waza-style skill verification")
    parser.add_argument("--json", action="store_true", help="JSON output")
    parser.add_argument("--skill", "-s", type=str, help="Check only one skill (name or path)")
    args = parser.parse_args()
    
    report = Report()
    
    # 1. Index consistency (global, not per-skill)
    report.add(check_index_consistency())
    
    # 2. Reference graph (global)
    report.add(check_reference_graph())
    
    # 3. Per-skill checks
    if args.skill:
        skill_paths = [PROJECT_ROOT / ".pi" / "skills" / args.skill / "SKILL.md"]
    else:
        skill_paths = sorted(SKILL_DIR.glob("dj-*/SKILL.md"))
    
    for skill_file in skill_paths:
        if not skill_file.exists():
            print(f"Warning: {skill_file} not found", file=sys.stderr)
            continue
        
        text = skill_file.read_text()
        skill_name = skill_file.parent.name
        
        report.add(check_frontmatter(skill_name, text))
        report.add(check_sections(skill_name, text))
        report.add(check_outcome_contract(skill_name, text))
        report.add(check_hard_rules(skill_name, text))
        report.add(check_gotchas(skill_name, text))
        report.add(check_references(skill_name, text))
    
    report.print_summary(format="json" if args.json else "text")
    
    return 0 if report.success() else 1


if __name__ == "__main__":
    sys.exit(main())
